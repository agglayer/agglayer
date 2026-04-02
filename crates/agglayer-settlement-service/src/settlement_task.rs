use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, OnceLock},
    time::{Duration, SystemTime},
};

use agglayer_storage::stores::SettlementWriter;
use agglayer_types::{
    ClientError, ClientErrorType, ContractCallOutcome, ContractCallResult, Digest, Nonce,
    SettlementAttempt, SettlementAttemptNumber, SettlementAttemptResult, SettlementJob,
    SettlementJobResult, SettlementTxHash,
};
use alloy::{
    consensus::{EthereumTxEnvelope, TxEip4844Variant},
    primitives::Address,
};
use eyre::Context as _;
use tokio::sync::mpsc;
use tracing::warn;
use ulid::Ulid;

type TxEnvelope = EthereumTxEnvelope<TxEip4844Variant>;

pub enum StoredSettlementJob<Store>
where
    Store: SettlementWriter,
{
    Pending(SettlementTask<Store>),
    Completed(SettlementJob, SettlementJobResult),
}

pub enum TaskAdminCommand {
    Abort,
    ReloadAndRestart,
}

struct ActiveSettlementAttempt {
    attempt: SettlementAttempt,
    result: Option<SettlementAttemptResult>,
}

pub struct SettlementTask<Store>
where
    Store: SettlementWriter,
{
    id: Ulid,
    job: SettlementJob,
    admin_commands: mpsc::Receiver<TaskAdminCommand>,
    attempts:
        BTreeMap<(Address, Nonce), BTreeMap<SettlementAttemptNumber, ActiveSettlementAttempt>>,
    store: Arc<Store>,
}

static ID_GENERATOR: OnceLock<std::sync::Mutex<ulid::Generator>> = OnceLock::new();

impl<Store> SettlementTask<Store>
where
    Store: SettlementWriter,
{
    pub async fn create(
        job: SettlementJob,
        admin_commands: mpsc::Receiver<TaskAdminCommand>,
        store: Arc<Store>,
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
            admin_commands,
            attempts: BTreeMap::new(),
            store,
        };
        this.save_settlement_job_to_db().await?;
        Ok((id, this))
    }

    pub async fn load(
        id: Ulid,
        admin_commands: mpsc::Receiver<TaskAdminCommand>,
        store: Arc<Store>,
    ) -> eyre::Result<StoredSettlementJob<Store>> {
        let (job, result) = Self::load_settlement_job_from_db(id, &store).await?;
        if let Some(result) = result {
            Ok(StoredSettlementJob::Completed(job, result))
        } else {
            let mut this = SettlementTask {
                id,
                job,
                admin_commands,
                attempts: BTreeMap::new(),
                store,
            };
            this.load_settlement_attempts_from_db().await?;
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
                    .find(|(_, attempt)| attempt.attempt.hash == tx_hash)
                    .map(|(attempt_number, _)| *attempt_number)
            })
    }

    fn attempt_by_number(
        &self,
        attempt_number: SettlementAttemptNumber,
    ) -> Option<&ActiveSettlementAttempt> {
        self.attempts
            .values()
            .find_map(|attempts_for_nonce| attempts_for_nonce.get(&attempt_number))
    }

    fn attempt_by_number_mut(
        &mut self,
        attempt_number: SettlementAttemptNumber,
    ) -> Option<&mut ActiveSettlementAttempt> {
        self.attempts
            .values_mut()
            .find_map(|attempts_for_nonce| attempts_for_nonce.get_mut(&attempt_number))
    }

    fn attempt_numbers_for_nonce(
        &self,
        wallet: Address,
        nonce: Nonce,
    ) -> Vec<SettlementAttemptNumber> {
        self.attempts
            .get(&(wallet, nonce))
            .map(|attempts_for_nonce| attempts_for_nonce.keys().copied().collect())
            .unwrap_or_default()
    }

    fn settlement_already_completed_result(tx_hash: SettlementTxHash) -> SettlementAttemptResult {
        SettlementAttemptResult::ClientError(ClientError {
            kind: ClientErrorType::Unknown,
            message: format!("Settlement job already completed successfully by tx {tx_hash}"),
        })
    }

    fn persist_attempt_result_if_unset(
        &mut self,
        attempt_number: SettlementAttemptNumber,
        result: SettlementAttemptResult,
    ) -> eyre::Result<()> {
        let existing = self
            .attempt_by_number(attempt_number)
            .ok_or_else(|| {
                eyre::eyre!(
                    "Unknown settlement attempt {attempt_number} for job {}",
                    self.id
                )
            })?
            .result
            .clone();

        match existing {
            Some(existing) if existing == result => return Ok(()),
            Some(existing) => {
                return Err(eyre::eyre!(
                    "Settlement attempt {attempt_number} for job {} already has a different result: {existing:?}",
                    self.id,
                ));
            }
            None => {}
        }

        self.store
            .insert_settlement_attempt_result(&self.id, attempt_number.0, &result)
            .wrap_err_with(|| {
                format!(
                    "Failed to persist result for settlement job {} attempt {attempt_number}",
                    self.id,
                )
            })?;

        self.attempt_by_number_mut(attempt_number)
            .ok_or_else(|| {
                eyre::eyre!(
                    "Unknown settlement attempt {attempt_number} for job {}",
                    self.id
                )
            })?
            .result = Some(result);

        Ok(())
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
        self.attempts
            .keys()
            .map(|(wallet, nonce)| self.next_attempt_deadline_for_nonce(*wallet, *nonce))
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

    async fn load_settlement_job_from_db(
        _id: Ulid,
        _store: &Store,
    ) -> eyre::Result<(SettlementJob, Option<SettlementJobResult>)> {
        // TODO: Load a settlement job's contents from L1, including its result if it is
        // completed.
        // XREF: https://github.com/agglayer/agglayer/issues/1381
        todo!()
    }

    async fn load_settlement_attempts_from_db(&mut self) -> eyre::Result<()> {
        // TODO: Load all the settlement attempts related to self into self
        // XREF: https://github.com/agglayer/agglayer/issues/1312
        todo!()
    }

    async fn save_attempt_to_db(
        &self,
        _wallet: Address,
        _nonce: Nonce,
        _attempt_number: SettlementAttemptNumber,
        _tx: SettlementTxHash,
    ) {
        // TODO: Save a new settlement attempt to db
        // XREF: https://github.com/agglayer/agglayer/issues/1320
        todo!()
    }

    async fn write_client_error_to_db(
        &mut self,
        attempt_number: SettlementAttemptNumber,
        result: ClientError,
    ) {
        if let Err(error) = self.persist_attempt_result_if_unset(
            attempt_number,
            SettlementAttemptResult::ClientError(result),
        ) {
            warn!(
                ?error,
                job_id = %self.id,
                %attempt_number,
                "Failed to persist settlement attempt client error"
            );
        }
    }

    async fn write_nonce_revert_to_db(
        &mut self,
        wallet: Address,
        nonce: Nonce,
        attempt_number: SettlementAttemptNumber,
        result: ContractCallResult,
    ) {
        let attempt_numbers = self.attempt_numbers_for_nonce(wallet, nonce);
        if attempt_numbers.is_empty() {
            warn!(
                job_id = %self.id,
                %wallet,
                %nonce,
                "No settlement attempts found for reverted nonce"
            );
            return;
        }

        for current_attempt_number in attempt_numbers {
            let attempt_result = if current_attempt_number == attempt_number {
                SettlementAttemptResult::ContractCall(result.clone())
            } else {
                SettlementAttemptResult::ClientError(ClientError::nonce_already_used(
                    wallet,
                    nonce,
                    result.tx_hash,
                ))
            };

            if let Err(error) =
                self.persist_attempt_result_if_unset(current_attempt_number, attempt_result)
            {
                warn!(
                    ?error,
                    job_id = %self.id,
                    %wallet,
                    %nonce,
                    %current_attempt_number,
                    "Failed to persist settlement nonce revert result"
                );
                return;
            }
        }
    }

    async fn write_nonce_used_externally_to_db(
        &mut self,
        wallet: Address,
        nonce: Nonce,
        tx_hash: SettlementTxHash,
    ) {
        let attempt_numbers = self.attempt_numbers_for_nonce(wallet, nonce);
        if attempt_numbers.is_empty() {
            warn!(
                job_id = %self.id,
                %wallet,
                %nonce,
                "No settlement attempts found for externally used nonce"
            );
            return;
        }

        for attempt_number in attempt_numbers {
            if let Err(error) = self.persist_attempt_result_if_unset(
                attempt_number,
                SettlementAttemptResult::ClientError(ClientError::nonce_already_used(
                    wallet, nonce, tx_hash,
                )),
            ) {
                warn!(
                    ?error,
                    job_id = %self.id,
                    %wallet,
                    %nonce,
                    %attempt_number,
                    "Failed to persist externally used nonce result"
                );
                return;
            }
        }
    }

    async fn write_job_successful_to_db(
        &mut self,
        wallet: Address,
        nonce: Nonce,
        attempt_number: SettlementAttemptNumber,
        tx_result: ContractCallResult,
    ) {
        let same_nonce_attempt_numbers = self.attempt_numbers_for_nonce(wallet, nonce);
        if same_nonce_attempt_numbers.is_empty() {
            warn!(
                job_id = %self.id,
                %wallet,
                %nonce,
                "No settlement attempts found for successful nonce"
            );
            return;
        }

        let other_attempt_numbers = self
            .attempts
            .iter()
            .filter(|((stored_wallet, stored_nonce), _)| {
                *stored_wallet != wallet || *stored_nonce != nonce
            })
            .flat_map(|(_, attempts_for_nonce)| attempts_for_nonce.keys().copied())
            .collect::<Vec<_>>();

        for current_attempt_number in same_nonce_attempt_numbers {
            let attempt_result = if current_attempt_number == attempt_number {
                SettlementAttemptResult::ContractCall(tx_result.clone())
            } else {
                SettlementAttemptResult::ClientError(ClientError::nonce_already_used(
                    wallet,
                    nonce,
                    tx_result.tx_hash,
                ))
            };

            if let Err(error) =
                self.persist_attempt_result_if_unset(current_attempt_number, attempt_result)
            {
                warn!(
                    ?error,
                    job_id = %self.id,
                    %wallet,
                    %nonce,
                    %current_attempt_number,
                    "Failed to persist successful settlement attempt result"
                );
                return;
            }
        }

        for current_attempt_number in other_attempt_numbers {
            if let Err(error) = self.persist_attempt_result_if_unset(
                current_attempt_number,
                Self::settlement_already_completed_result(tx_result.tx_hash),
            ) {
                warn!(
                    ?error,
                    job_id = %self.id,
                    %current_attempt_number,
                    "Failed to persist cross-nonce settlement completion result"
                );
                return;
            }
        }

        if let Err(error) = self
            .store
            .insert_settlement_job_result(&self.id, &SettlementJobResult::ContractCall(tx_result))
            .wrap_err_with(|| format!("Failed to persist settlement job result for {}", self.id))
        {
            warn!(
                ?error,
                job_id = %self.id,
                "Failed to persist successful settlement job result"
            );
        }
    }

    async fn write_job_revert_to_db(&mut self, result: &ContractCallResult) {
        let pending_attempts = self
            .attempts
            .iter()
            .flat_map(|(_, attempts_for_nonce)| attempts_for_nonce.iter())
            .filter_map(|(attempt_number, attempt)| {
                attempt.result.is_none().then_some(*attempt_number)
            })
            .collect::<Vec<_>>();

        if !pending_attempts.is_empty() {
            warn!(
                job_id = %self.id,
                ?pending_attempts,
                "Persisting reverted settlement job while some attempts are still pending"
            );
        }

        if let Err(error) = self
            .store
            .insert_settlement_job_result(
                &self.id,
                &SettlementJobResult::ContractCall(result.clone()),
            )
            .wrap_err_with(|| format!("Failed to persist settlement job result for {}", self.id))
        {
            warn!(
                ?error,
                job_id = %self.id,
                "Failed to persist reverted settlement job result"
            );
        }
    }

    async fn save_settlement_job_to_db(&self) -> eyre::Result<()> {
        // TODO: Save the settlement job contents to L1
        // XREF: https://github.com/agglayer/agglayer/issues/1381
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, HashMap},
        sync::{Arc, Mutex},
    };

    use agglayer_storage::{error::Error, stores::SettlementWriter};
    use agglayer_types::U256;

    use super::*;

    #[derive(Default)]
    struct RecordingStore {
        attempt_results: Mutex<HashMap<(Ulid, u64), SettlementAttemptResult>>,
        job_results: Mutex<HashMap<Ulid, SettlementJobResult>>,
    }

    impl SettlementWriter for RecordingStore {
        fn insert_settlement_job(
            &self,
            _settlement_job_id: &Ulid,
            _settlement_job: &SettlementJob,
        ) -> Result<(), Error> {
            Ok(())
        }

        fn insert_settlement_job_result(
            &self,
            settlement_job_id: &Ulid,
            tx_result: &SettlementJobResult,
        ) -> Result<(), Error> {
            let mut job_results = self.job_results.lock().unwrap();
            if job_results
                .insert(*settlement_job_id, tx_result.clone())
                .is_some()
            {
                return Err(Error::UnprocessedAction(
                    "duplicate settlement job result".to_string(),
                ));
            }
            Ok(())
        }

        fn insert_settlement_attempt(
            &self,
            _settlement_job_id: &Ulid,
            _attempt_sequence_number: u64,
            _settlement_attempt: &SettlementAttempt,
        ) -> Result<(), Error> {
            Ok(())
        }

        fn insert_settlement_attempt_result(
            &self,
            settlement_job_id: &Ulid,
            attempt_sequence_number: u64,
            tx_result: &SettlementAttemptResult,
        ) -> Result<(), Error> {
            let mut attempt_results = self.attempt_results.lock().unwrap();
            if attempt_results
                .insert(
                    (*settlement_job_id, attempt_sequence_number),
                    tx_result.clone(),
                )
                .is_some()
            {
                return Err(Error::UnprocessedAction(
                    "duplicate settlement attempt result".to_string(),
                ));
            }
            Ok(())
        }
    }

    fn setup_store() -> Arc<RecordingStore> {
        Arc::new(RecordingStore::default())
    }

    fn mk_job(seed: u8) -> SettlementJob {
        SettlementJob {
            contract_address: Address::from([seed; 20]),
            calldata: vec![seed, seed.wrapping_add(1)].into(),
            eth_value: U256::from_be_bytes([seed; 32]),
            gas_limit: u128::from_be_bytes([seed; 16]),
            max_fee_per_gas_ceiling: u128::from_be_bytes([seed.wrapping_add(1); 16]),
            max_fee_per_gas_floor: u128::from_be_bytes([seed.wrapping_add(2); 16]),
            max_fee_per_gas_increase_percents: 10,
            max_priority_fee_per_gas_ceiling: u128::from_be_bytes([seed.wrapping_add(3); 16]),
            max_priority_fee_per_gas_floor: u128::from_be_bytes([seed.wrapping_add(4); 16]),
            max_priority_fee_per_gas_increase_percents: 20,
        }
    }

    fn mk_attempt(wallet: Address, nonce: Nonce, hash_seed: u8) -> SettlementAttempt {
        SettlementAttempt {
            sender_wallet: wallet,
            nonce,
            max_fee_per_gas: u128::from_be_bytes([hash_seed; 16]),
            max_priority_fee_per_gas: u128::from_be_bytes([hash_seed.wrapping_add(1); 16]),
            hash: SettlementTxHash::new(Digest::from([hash_seed; 32])),
            submission_time: SystemTime::UNIX_EPOCH + Duration::from_secs(hash_seed as u64),
        }
    }

    fn mk_contract_call_result(
        outcome: ContractCallOutcome,
        block_number: u64,
        tx_hash: SettlementTxHash,
    ) -> ContractCallResult {
        ContractCallResult {
            outcome,
            metadata: vec![block_number as u8].into(),
            block_hash: Digest::from([(block_number as u8).wrapping_add(1); 32]).into(),
            block_number,
            tx_hash,
        }
    }

    fn mk_task(
        id: Ulid,
        job: SettlementJob,
        store: Arc<RecordingStore>,
    ) -> SettlementTask<RecordingStore> {
        let (_admin_tx, admin_rx) = mpsc::channel(1);
        SettlementTask {
            id,
            job,
            admin_commands: admin_rx,
            attempts: BTreeMap::new(),
            store,
        }
    }

    #[tokio::test]
    async fn write_job_successful_persists_attempt_and_job_results() {
        let store = setup_store();
        let job_id = Ulid::from(202_u128);
        let job = mk_job(2);
        let wallet = Address::from([8; 20]);
        let nonce = Nonce(9);
        let other_wallet = Address::from([9; 20]);
        let other_nonce = Nonce(10);
        let first_attempt = mk_attempt(wallet, nonce, 21);
        let second_attempt = mk_attempt(wallet, nonce, 22);
        let other_attempt = mk_attempt(other_wallet, other_nonce, 23);
        let success_result =
            mk_contract_call_result(ContractCallOutcome::Success, 77, first_attempt.hash);
        let mut task = mk_task(job_id, job.clone(), store.clone());

        task.attempts.entry((wallet, nonce)).or_default().insert(
            SettlementAttemptNumber(1),
            ActiveSettlementAttempt {
                attempt: first_attempt.clone(),
                result: None,
            },
        );
        task.attempts.entry((wallet, nonce)).or_default().insert(
            SettlementAttemptNumber(2),
            ActiveSettlementAttempt {
                attempt: second_attempt.clone(),
                result: None,
            },
        );
        task.attempts
            .entry((other_wallet, other_nonce))
            .or_default()
            .insert(
                SettlementAttemptNumber(3),
                ActiveSettlementAttempt {
                    attempt: other_attempt.clone(),
                    result: None,
                },
            );

        task.write_job_successful_to_db(
            wallet,
            nonce,
            SettlementAttemptNumber(1),
            success_result.clone(),
        )
        .await;

        assert_eq!(
            task.attempts
                .get(&(wallet, nonce))
                .and_then(|attempts| attempts.get(&SettlementAttemptNumber(1)))
                .map(|attempt| attempt.result.clone()),
            Some(Some(SettlementAttemptResult::ContractCall(
                success_result.clone()
            )))
        );
        assert_eq!(
            task.attempts
                .get(&(wallet, nonce))
                .and_then(|attempts| attempts.get(&SettlementAttemptNumber(2)))
                .map(|attempt| attempt.result.clone()),
            Some(Some(SettlementAttemptResult::ClientError(
                ClientError::nonce_already_used(wallet, nonce, success_result.tx_hash)
            )))
        );

        match task
            .attempts
            .get(&(other_wallet, other_nonce))
            .and_then(|attempts| attempts.get(&SettlementAttemptNumber(3)))
            .map(|attempt| attempt.result.clone())
        {
            Some(Some(SettlementAttemptResult::ClientError(error))) => {
                assert_eq!(error.kind, ClientErrorType::Unknown);
                assert!(error.message.contains(&success_result.tx_hash.to_string()));
            }
            other => panic!("unexpected cross-nonce attempt result: {other:?}"),
        }

        assert_eq!(
            store
                .attempt_results
                .lock()
                .unwrap()
                .get(&(job_id, 1))
                .cloned(),
            Some(SettlementAttemptResult::ContractCall(
                success_result.clone()
            ))
        );
        assert_eq!(
            store
                .attempt_results
                .lock()
                .unwrap()
                .get(&(job_id, 2))
                .cloned(),
            Some(SettlementAttemptResult::ClientError(
                ClientError::nonce_already_used(wallet, nonce, success_result.tx_hash)
            ))
        );
        match store
            .attempt_results
            .lock()
            .unwrap()
            .get(&(job_id, 3))
            .cloned()
        {
            Some(SettlementAttemptResult::ClientError(error)) => {
                assert_eq!(error.kind, ClientErrorType::Unknown);
                assert!(error.message.contains(&success_result.tx_hash.to_string()));
            }
            other => panic!("unexpected persisted cross-nonce attempt result: {other:?}"),
        }

        assert_eq!(
            store.job_results.lock().unwrap().get(&job_id).cloned(),
            Some(SettlementJobResult::ContractCall(success_result))
        );
    }
}
