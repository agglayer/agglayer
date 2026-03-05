use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, OnceLock},
    time::{Duration, SystemTime},
};

use agglayer_storage::{
    error::Error as StorageError,
    stores::{SettlementReader, SettlementWriter},
    types::generated::agglayer::storage::v0,
};
use agglayer_types::{
    Address, ClientError, ClientErrorType, ContractCallOutcome, ContractCallResult, Digest, Nonce,
    SettlementAttempt, SettlementAttemptNumber, SettlementJob, SettlementJobResult,
    SettlementTxHash,
};
use alloy::consensus::{EthereumTxEnvelope, TxEip4844Variant};
use tokio::sync::mpsc;
use tracing::warn;
use ulid::Ulid;

type TxEnvelope = EthereumTxEnvelope<TxEip4844Variant>;

pub enum StoredSettlementJob<S> {
    Pending(SettlementTask<S>),
    Completed(SettlementJob, SettlementJobResult),
}

pub enum TaskAdminCommand {
    Abort,
    ReloadAndRestart,
}

type AttemptsByNonce =
    BTreeMap<(Address, Nonce), BTreeMap<SettlementAttemptNumber, SettlementAttempt>>;

pub(crate) fn derive_terminal_result_from_storage_records(
    attempts: &[(u64, v0::SettlementAttempt)],
    attempt_results: &[(u64, v0::TxResult)],
) -> eyre::Result<Option<SettlementJobResult>> {
    let mut attempt_sequence_numbers = BTreeSet::new();
    for (attempt_sequence_number, _) in attempts {
        attempt_sequence_numbers.insert(*attempt_sequence_number);
    }

    let mut parsed_results = BTreeMap::new();
    for (attempt_sequence_number, result) in attempt_results {
        let parsed_result = SettlementJobResult::try_from(result.clone())?;
        if !attempt_sequence_numbers.contains(attempt_sequence_number) {
            return Err(eyre::eyre!(
                "Settlement storage is inconsistent for job: result for attempt {} exists but the \
                 corresponding attempt does not",
                attempt_sequence_number,
            ));
        }
        parsed_results.insert(*attempt_sequence_number, parsed_result);
    }

    for result in parsed_results.values() {
        if let SettlementJobResult::ContractCall(contract_call_result) = result {
            if contract_call_result.outcome == ContractCallOutcome::Success {
                return Ok(Some(result.clone()));
            }
        }
    }

    let all_attempts_have_results = !attempts.is_empty()
        && attempts.iter().all(|(attempt_sequence_number, _)| {
            parsed_results.contains_key(attempt_sequence_number)
        });

    if !all_attempts_have_results {
        return Ok(None);
    }

    let earliest_revert = parsed_results
        .values()
        .filter_map(|result| match result {
            SettlementJobResult::ContractCall(contract_call_result)
                if contract_call_result.outcome == ContractCallOutcome::Revert =>
            {
                Some(contract_call_result)
            }
            _ => None,
        })
        .min_by_key(|result| result.block_number)
        .cloned();

    Ok(earliest_revert.map(SettlementJobResult::ContractCall))
}

pub struct SettlementTask<S> {
    id: Ulid,
    job: SettlementJob,
    store: Arc<S>,
    admin_commands: mpsc::Receiver<TaskAdminCommand>,
    attempts: AttemptsByNonce,
}

static ID_GENERATOR: OnceLock<std::sync::Mutex<ulid::Generator>> = OnceLock::new();

impl<S> SettlementTask<S>
where
    S: SettlementReader + SettlementWriter,
{
    pub async fn create(
        store: Arc<S>,
        job: SettlementJob,
        admin_commands: mpsc::Receiver<TaskAdminCommand>,
    ) -> eyre::Result<(Ulid, Self)> {
        let id = loop {
            if let Ok(id) = ID_GENERATOR
                .get_or_init(|| std::sync::Mutex::new(ulid::Generator::new()))
                .lock()
                .unwrap()
                .generate()
            {
                break id;
            }
            tokio::time::sleep(std::time::Duration::from_micros(100)).await;
        };
        let this = Self {
            id,
            job,
            store,
            admin_commands,
            attempts: BTreeMap::new(),
        };
        this.save_settlement_job_to_db().await?;
        Ok((id, this))
    }

    pub async fn load(
        store: Arc<S>,
        id: Ulid,
        admin_commands: mpsc::Receiver<TaskAdminCommand>,
    ) -> eyre::Result<StoredSettlementJob<S>> {
        let job_proto = store
            .get_settlement_job(&id)?
            .ok_or_else(|| eyre::eyre!("Settlement job {} not found in storage", id))?;
        let job = SettlementJob::try_from(job_proto)?;

        let attempts = store.list_settlement_attempts(&id)?;
        let attempt_results = store.list_settlement_attempt_results(&id)?;
        if let Some(result) =
            derive_terminal_result_from_storage_records(&attempts, &attempt_results)?
        {
            Ok(StoredSettlementJob::Completed(job, result))
        } else {
            let attempts = Self::attempts_from_storage_records(&id, attempts, attempt_results)?;

            let this = SettlementTask {
                id,
                job,
                store,
                admin_commands,
                attempts,
            };
            Ok(StoredSettlementJob::Pending(this))
        }
    }

    pub async fn run(&mut self) -> SettlementJobResult {
        'start: loop {
            // Process in a big loop. We'll come back here whenever a reorg is detected, and
            // after waiting when we're done with one cycle.

            // First, for each nonce we know of, identify whether it is done or whether we
            // need to submit more txes for it. For this, we'll keep a list of
            // nonces used externally and reverts (that are not finalized yet), as well as
            // helper markers.
            let mut nonces_used_externally = BTreeMap::new();
            let mut reverts = BTreeMap::new();
            let mut all_nonces_seen_on_l1 = true;
            let mut need_to_submit_attempt_with_new_nonce = true;
            'nonces: for (wallet, nonce) in self.all_used_nonces() {
                if let Some(tx_hash) = self.tx_hash_on_l1_for_nonce(wallet, nonce).await {
                    // If the nonce is used on L1, we won't need to submit any new tx related to it.
                    let Some(attempt_number) =
                        self.settlement_attempt_number_for(wallet, nonce, tx_hash)
                    else {
                        nonces_used_externally.insert((wallet, nonce), tx_hash);
                        continue 'nonces;
                    };
                    let Some(tx_result) = self.current_result_on_l1_for(tx_hash).await else {
                        continue 'start; // reorg
                    };
                    if tx_result.outcome != ContractCallOutcome::Success {
                        reverts.insert((wallet, nonce), (attempt_number, tx_hash, tx_result));
                        continue 'nonces;
                    }
                    let Some(settled_result) = self.wait_for_settlement_of(tx_hash).await else {
                        continue 'start; // reorg
                    };
                    if settled_result != tx_result {
                        continue 'start; // reorg
                    }
                    self.write_job_successful_to_db(
                        wallet,
                        nonce,
                        attempt_number,
                        tx_result.clone(),
                    )
                    .await;
                    return SettlementJobResult::ContractCall(tx_result);
                } else {
                    // If the nonce is not used on L1, we'll need to either wait more or submit a
                    // new attempt with the same nonce.
                    all_nonces_seen_on_l1 = false;
                    if !self.is_wallet_privkey_known(wallet) {
                        continue 'nonces; // we don't have access to the wallet
                                          // any longer, so it makes no sense to
                                          // check if we need to resubmit.
                    }
                    // This nonce is not included yet and we still know the privkey, so we won't
                    // need to submit an attempt with a new nonce, regardless of whether we
                    // resubmit.
                    need_to_submit_attempt_with_new_nonce = false;
                    if self.is_any_attempt_pending_for_nonce(wallet, nonce) {
                        // At least one attempt is not in-error yet, so we'll need to wait for the
                        // previous nonce to be included before processing it further.
                        if let Some(previous_nonce) = nonce.previous() {
                            if self
                                .tx_hash_on_l1_for_nonce(wallet, previous_nonce)
                                .await
                                .is_none()
                            {
                                continue 'nonces; // wait for previous nonce to
                                                  // be included
                            }
                        }
                    }
                    let deadline = self.next_attempt_deadline_for_nonce(wallet, nonce);
                    if deadline > SystemTime::now() {
                        continue 'nonces; // wait for deadline to be reached
                    }
                    let (attempt_number, tx) = self.build_next_attempt_with_nonce(wallet, nonce);
                    self.save_attempt_to_db_and_submit_to_l1(wallet, nonce, attempt_number, tx)
                        .await;
                }
            }
            if all_nonces_seen_on_l1 && !reverts.is_empty() {
                // All nonces were seen on L1, but we didn't get a successful settlement result
                // for any of them. Also, there was at least one revert.
                // We can wait for finalization without submitting a new attempt.
                let earliest_revert_result = reverts
                    .values()
                    .map(|(_, _, result)| result)
                    .min_by_key(|result| result.block_number)
                    .unwrap() // No panic: we checked `!reverts.is_empty()` just before.
                    .clone();
                for (wallet, nonce) in self.all_used_nonces() {
                    if let Some(tx_hash) = nonces_used_externally.remove(&(wallet, nonce)) {
                        if self.wait_for_settlement_of(tx_hash).await.is_none() {
                            continue 'start; // reorg
                        }
                        self.write_nonce_used_externally_to_db(wallet, nonce, tx_hash)
                            .await;
                    } else if let Some((attempt_number, tx_hash, result)) =
                        reverts.remove(&(wallet, nonce))
                    {
                        let Some(settled_result) = self.wait_for_settlement_of(tx_hash).await
                        else {
                            continue 'start; // reorg
                        };
                        if settled_result != result {
                            continue 'start; // reorg
                        }
                        self.write_nonce_revert_to_db(wallet, nonce, attempt_number, result)
                            .await;
                    } else {
                        // Invariant: If we finish the `'nonces` loop with `all_nonces_seen_on_l1`,
                        // all nonces must be one of success, revert or external use.
                        // Any success would have led to either an early return, or a loop back to
                        // `'start` if it did not settle properly.
                        // As such, we must have entered at least one of the two branches above for
                        // each nonce.
                        panic!(
                            "Settlement logic invariant broken: nonces seen on L1 must be either \
                             success, revert or external use"
                        );
                    }
                }
                self.write_job_revert_to_db(&earliest_revert_result).await;
                return SettlementJobResult::ContractCall(earliest_revert_result.clone());
            }
            // There was no successful attempt, and either at least one nonce was not yet
            // seen on L1 or there is no reverting attempt. So we need to wait
            // for more nonces to be seen on L1.
            if need_to_submit_attempt_with_new_nonce {
                // There was no attempt that was pending or that received a retry in the
                // `'nonces` loop above. This means that either all nonces were
                // used externally, or that we no longer have the required wallets to bump
                // pending nonces. So we need to submit a new attempt with a new
                // nonce.
                let (wallet, nonce, attempt_number, tx) = self.build_next_attempt_with_new_nonce();
                self.save_attempt_to_db_and_submit_to_l1(wallet, nonce, attempt_number, tx)
                    .await;
            }
            // We now are sure we did at least one step to make things move forward. Wait
            // for the next external event or for the next deadline.
            let timeout = self
                .next_overall_deadline()
                .expect("There is at least one attempt but no deadline")
                .duration_since(SystemTime::now())
                .unwrap_or_else(|_| Duration::from_secs(0));
            let _ = tokio::time::timeout(timeout, self.wait_for_any_nonce_on_l1()).await;
        }
    }

    async fn save_attempt_to_db_and_submit_to_l1(
        &mut self,
        wallet: Address,
        nonce: Nonce,
        attempt_number: SettlementAttemptNumber,
        tx: TxEnvelope,
    ) {
        let tx_hash = SettlementTxHash::from(Digest::from(*tx.tx_hash()));
        self.save_attempt_to_db(wallet, nonce, attempt_number, tx_hash)
            .await;
        if let Err(error) = self.submit_attempt_to_l1(tx).await {
            warn!(?error, "Failed to submit settlement attempt to L1");
            self.write_client_error_to_db(
                attempt_number,
                ClientError {
                    kind: ClientErrorType::Unknown,
                    message: format!("Failed to submit settlement attempt to L1: {error:?}"),
                },
            )
            .await;
        }
    }

    fn all_used_nonces(&self) -> BTreeSet<(Address, Nonce)> {
        self.attempts.keys().cloned().collect()
    }

    fn is_any_attempt_pending_for_nonce(&self, wallet: Address, nonce: Nonce) -> bool {
        self.attempts
            .get(&(wallet, nonce))
            .map(|attempts_for_nonce| {
                attempts_for_nonce
                    .values()
                    .any(|attempt| attempt.result.is_none())
            })
            .unwrap_or(false)
    }

    fn settlement_attempt_number_for(
        &self,
        wallet: Address,
        nonce: Nonce,
        tx_hash: SettlementTxHash,
    ) -> Option<SettlementAttemptNumber> {
        self.attempts
            .get(&(wallet, nonce))
            .and_then(|attempts_for_nonce| {
                attempts_for_nonce
                    .iter()
                    .find(|(_, attempt)| attempt.hash == tx_hash)
                    .map(|(attempt_number, _)| *attempt_number)
            })
    }

    fn is_wallet_privkey_known(&self, _wallet: Address) -> bool {
        // TODO: tie with the configuration
        todo!()
    }

    fn next_attempt_deadline_for_nonce(&self, _wallet: Address, _nonce: Nonce) -> SystemTime {
        // TODO: use already-available timeout config to define the next attempt
        // deadline, considering both RPC-level retry for ClientErrors and
        // non-inclusion-level retry for the others
        todo!()
    }

    fn next_overall_deadline(&self) -> Option<SystemTime> {
        let used_nonces = self.attempts.keys().cloned().collect::<Vec<_>>();
        used_nonces
            .into_iter()
            .map(|(wallet, nonce)| self.next_attempt_deadline_for_nonce(wallet, nonce))
            .min()
    }

    async fn wait_for_any_nonce_on_l1(&self) {
        // TODO: wait for any nonce from our known list to be included on L1 (not
        // settled, just included) Use retry_callback_until_success as needed
        // XREF: https://github.com/agglayer/agglayer/issues/1314
        todo!()
    }

    async fn tx_hash_on_l1_for_nonce(
        &self,
        _wallet: Address,
        _nonce: Nonce,
    ) -> Option<SettlementTxHash> {
        // TODO: delegate to the standalone `tx_hash_on_l1_for_nonce` function
        // once a provider field is added to SettlementTask.
        // Use retry_callback_until_success as needed.
        todo!()
    }

    async fn current_result_on_l1_for(
        &self,
        _tx_hash: SettlementTxHash,
    ) -> Option<ContractCallResult> {
        // TODO: return the result on L1 if the tx_hash is already included on L1, and
        // None otherwise Use retry_callback_until_success as needed
        // XREF: https://github.com/agglayer/agglayer/issues/1382
        todo!()
    }

    async fn wait_for_settlement_of(
        &self,
        _tx_hash: SettlementTxHash,
    ) -> Option<ContractCallResult> {
        // TODO: Wait for the settlement of tx_hash, and then return its result on L1.
        // If a reorg is detected during the waiting, return None.
        // XREF: https://github.com/agglayer/agglayer/issues/1316
        todo!()
    }

    fn build_next_attempt_with_nonce(
        &self,
        _wallet: Address,
        _nonce: Nonce,
    ) -> (SettlementAttemptNumber, TxEnvelope) {
        // TODO: Build the next attempt with correct gas and other params. Use https://docs.rs/alloy/latest/alloy/rpc/types/struct.TransactionRequest.html#method.build
        // XREF: https://github.com/agglayer/agglayer/issues/1319
        todo!()
    }

    fn build_next_attempt_with_new_nonce(
        &self,
    ) -> (Address, Nonce, SettlementAttemptNumber, TxEnvelope) {
        // TODO: Build the next attempt with correct gas and other params. Use https://docs.rs/alloy/latest/alloy/rpc/types/struct.TransactionRequest.html#method.build
        // XREF: https://github.com/agglayer/agglayer/issues/1318
        todo!()
    }

    async fn submit_attempt_to_l1(&self, _tx: TxEnvelope) -> eyre::Result<()> {
        // TODO: Submit attempt to L1. Use https://docs.rs/alloy/latest/alloy/providers/trait.Provider.html#method.send_tx_envelope
        // XREF: https://github.com/agglayer/agglayer/issues/1321
        todo!()
    }

    async fn save_settlement_job_to_db(&self) -> eyre::Result<()> {
        let settlement_job = v0::SettlementJob::try_from(&self.job)?;
        self.store
            .insert_settlement_job(&self.id, &settlement_job)?;
        Ok(())
    }

    async fn load_settlement_attempts_from_db(&mut self) -> eyre::Result<()> {
        let settlement_attempts = self.store.list_settlement_attempts(&self.id)?;
        let settlement_attempt_results = self.store.list_settlement_attempt_results(&self.id)?;

        self.attempts = Self::attempts_from_storage_records(
            &self.id,
            settlement_attempts,
            settlement_attempt_results,
        )?;
        Ok(())
    }

    fn attempts_from_storage_records(
        settlement_job_id: &Ulid,
        settlement_attempts: Vec<(u64, v0::SettlementAttempt)>,
        settlement_attempt_results: Vec<(u64, v0::TxResult)>,
    ) -> eyre::Result<AttemptsByNonce> {
        let mut parsed_attempt_results = BTreeMap::new();
        for (attempt_sequence_number, result) in settlement_attempt_results {
            parsed_attempt_results.insert(
                attempt_sequence_number,
                SettlementJobResult::try_from(result)?,
            );
        }

        let mut attempts = BTreeMap::new();
        for (attempt_sequence_number, settlement_attempt) in settlement_attempts {
            let mut attempt = SettlementAttempt::try_from(settlement_attempt)?;
            attempt.result = parsed_attempt_results.remove(&attempt_sequence_number);
            attempts
                .entry((attempt.sender_wallet, attempt.nonce))
                .or_insert_with(BTreeMap::new)
                .insert(SettlementAttemptNumber(attempt_sequence_number), attempt);
        }

        if !parsed_attempt_results.is_empty() {
            return Err(eyre::eyre!(
                "Settlement storage is inconsistent for job {}: found attempt results without \
                 corresponding attempts",
                settlement_job_id,
            ));
        }

        Ok(attempts)
    }

    async fn save_attempt_to_db(
        &mut self,
        wallet: Address,
        nonce: Nonce,
        attempt_number: SettlementAttemptNumber,
        tx: SettlementTxHash,
    ) {
        let settlement_attempt = SettlementAttempt {
            sender_wallet: wallet,
            nonce,
            max_fee_per_gas: 0,
            max_priority_fee_per_gas: 0,
            hash: tx,
            submission_time: SystemTime::now(),
            result: None,
        };
        let stored_attempt: v0::SettlementAttempt = (&settlement_attempt).into();

        if let Err(error) =
            self.store
                .insert_settlement_attempt(&self.id, attempt_number.0, &stored_attempt)
        {
            if !Self::is_already_exists_storage_error(&error) {
                panic!(
                    "Failed to save settlement attempt {} for job {}: {error}",
                    attempt_number.0, self.id,
                );
            }
        }

        self.attempts
            .entry((wallet, nonce))
            .or_default()
            .entry(attempt_number)
            .or_insert(settlement_attempt);
    }

    async fn write_client_error_to_db(
        &mut self,
        attempt_number: SettlementAttemptNumber,
        result: ClientError,
    ) {
        let attempt_result = SettlementJobResult::ClientError(result);
        self.persist_attempt_result_or_panic(
            attempt_number,
            &attempt_result,
            "write_client_error_to_db",
        );
        self.set_attempt_result_in_memory(attempt_number, attempt_result);
    }

    async fn write_nonce_revert_to_db(
        &mut self,
        wallet: Address,
        nonce: Nonce,
        attempt_number: SettlementAttemptNumber,
        result: ContractCallResult,
    ) {
        let attempt_numbers = self.attempt_numbers_for_nonce_snapshot(wallet, nonce);
        if attempt_numbers.is_empty() {
            panic!(
                "Cannot mark nonce revert for job {}: no attempts tracked for {wallet}/{nonce}",
                self.id,
            );
        }

        for current_attempt_number in attempt_numbers {
            let attempt_result = if current_attempt_number == attempt_number {
                SettlementJobResult::ContractCall(result.clone())
            } else {
                SettlementJobResult::ClientError(ClientError::nonce_already_used(
                    wallet,
                    nonce,
                    result.tx_hash,
                ))
            };

            self.persist_attempt_result_or_panic(
                current_attempt_number,
                &attempt_result,
                "write_nonce_revert_to_db",
            );
            self.set_attempt_result_in_memory(current_attempt_number, attempt_result);
        }
    }

    async fn write_nonce_used_externally_to_db(
        &mut self,
        wallet: Address,
        nonce: Nonce,
        tx_hash: SettlementTxHash,
    ) {
        let attempt_numbers = self.attempt_numbers_for_nonce_snapshot(wallet, nonce);
        if attempt_numbers.is_empty() {
            panic!(
                "Cannot mark external nonce usage for job {}: no attempts tracked for \
                 {wallet}/{nonce}",
                self.id,
            );
        }

        for attempt_number in attempt_numbers {
            let attempt_result = SettlementJobResult::ClientError(ClientError::nonce_already_used(
                wallet, nonce, tx_hash,
            ));
            self.persist_attempt_result_or_panic(
                attempt_number,
                &attempt_result,
                "write_nonce_used_externally_to_db",
            );
            self.set_attempt_result_in_memory(attempt_number, attempt_result);
        }
    }

    async fn write_job_successful_to_db(
        &mut self,
        wallet: Address,
        nonce: Nonce,
        attempt_number: SettlementAttemptNumber,
        tx_result: ContractCallResult,
    ) {
        let attempts = self.all_attempt_numbers_snapshot();
        if attempts.is_empty() {
            panic!(
                "Cannot mark settlement job {} as successful: no attempts are tracked",
                self.id,
            );
        }

        for (current_wallet, current_nonce, current_attempt_number) in attempts {
            let attempt_result = if current_wallet == wallet
                && current_nonce == nonce
                && current_attempt_number == attempt_number
            {
                SettlementJobResult::ContractCall(tx_result.clone())
            } else if current_wallet == wallet && current_nonce == nonce {
                SettlementJobResult::ClientError(ClientError::nonce_already_used(
                    wallet,
                    nonce,
                    tx_result.tx_hash,
                ))
            } else {
                SettlementJobResult::ClientError(ClientError {
                    kind: ClientErrorType::Unknown,
                    message: format!(
                        "Settlement succeeded on another nonce: settled tx is {}",
                        tx_result.tx_hash,
                    ),
                })
            };

            self.persist_attempt_result_or_panic(
                current_attempt_number,
                &attempt_result,
                "write_job_successful_to_db",
            );
            self.set_attempt_result_in_memory(current_attempt_number, attempt_result);
        }
    }

    async fn write_job_revert_to_db(&self, result: &ContractCallResult) {
        let pending_attempts = self
            .attempts
            .iter()
            .flat_map(|((wallet, nonce), attempts_for_nonce)| {
                attempts_for_nonce
                    .iter()
                    .filter_map(move |(attempt_number, attempt)| {
                        if attempt.result.is_none() {
                            Some(format!("{wallet}/{nonce}/{}", attempt_number.0))
                        } else {
                            None
                        }
                    })
            })
            .collect::<Vec<_>>();

        if !pending_attempts.is_empty() {
            panic!(
                "Cannot mark settlement job {} as reverted at tx {}: pending attempts remain ({})",
                self.id,
                result.tx_hash,
                pending_attempts.join(", "),
            );
        }
    }

    fn attempt_numbers_for_nonce_snapshot(
        &self,
        wallet: Address,
        nonce: Nonce,
    ) -> Vec<SettlementAttemptNumber> {
        self.attempts
            .get(&(wallet, nonce))
            .map(|attempts_for_nonce| attempts_for_nonce.keys().copied().collect())
            .unwrap_or_default()
    }

    fn all_attempt_numbers_snapshot(&self) -> Vec<(Address, Nonce, SettlementAttemptNumber)> {
        self.attempts
            .iter()
            .flat_map(|((wallet, nonce), attempts_for_nonce)| {
                attempts_for_nonce
                    .keys()
                    .map(|attempt_number| (*wallet, *nonce, *attempt_number))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn persist_attempt_result_or_panic(
        &self,
        attempt_number: SettlementAttemptNumber,
        result: &SettlementJobResult,
        operation: &'static str,
    ) {
        let stored_result: v0::TxResult = result.clone().into();
        match self.store.insert_settlement_attempt_result(
            &self.id,
            attempt_number.0,
            &stored_result,
        ) {
            Ok(()) => {}
            Err(error) if Self::is_already_exists_storage_error(&error) => {
                let persisted_result =
                    self.load_persisted_attempt_result_or_panic(attempt_number, operation);
                if &persisted_result != result {
                    panic!(
                        "{} attempted to overwrite settlement job {} attempt {} result; existing: \
                         {:?}, new: {:?}",
                        operation, self.id, attempt_number.0, persisted_result, result,
                    );
                }
            }
            Err(error) => {
                panic!(
                    "{} failed for settlement job {} attempt {}: {error}",
                    operation, self.id, attempt_number.0,
                );
            }
        }
    }

    fn load_persisted_attempt_result_or_panic(
        &self,
        attempt_number: SettlementAttemptNumber,
        operation: &'static str,
    ) -> SettlementJobResult {
        let stored_results = self
            .store
            .list_settlement_attempt_results(&self.id)
            .unwrap_or_else(|error| {
                panic!(
                    "{} failed to load existing attempt results for settlement job {}: {error}",
                    operation, self.id,
                )
            });

        let stored_result = stored_results
            .into_iter()
            .find_map(|(stored_attempt_number, stored_result)| {
                (stored_attempt_number == attempt_number.0).then_some(stored_result)
            })
            .unwrap_or_else(|| {
                panic!(
                    "{} observed an existing result for settlement job {} attempt {}, but no \
                     stored result could be loaded",
                    operation, self.id, attempt_number.0,
                )
            });

        SettlementJobResult::try_from(stored_result).unwrap_or_else(|error| {
            panic!(
                "{} failed to decode existing result for settlement job {} attempt {}: {error}",
                operation, self.id, attempt_number.0,
            )
        })
    }

    fn set_attempt_result_in_memory(
        &mut self,
        attempt_number: SettlementAttemptNumber,
        result: SettlementJobResult,
    ) {
        for attempts_for_nonce in self.attempts.values_mut() {
            if let Some(attempt) = attempts_for_nonce.get_mut(&attempt_number) {
                attempt.result = Some(result);
                return;
            }
        }

        panic!(
            "Failed to update in-memory settlement attempt {} for job {}: attempt was not found",
            attempt_number.0, self.id,
        );
    }

    fn is_already_exists_storage_error(error: &StorageError) -> bool {
        matches!(error, StorageError::UnprocessedAction(message) if message.contains("already exists"))
    }
}
