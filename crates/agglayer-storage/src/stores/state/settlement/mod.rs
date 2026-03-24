use agglayer_types::{
    SettlementAttempt, SettlementAttemptResult, SettlementJob, SettlementJobResult,
};
use rocksdb::WriteBatch;
use ulid::Ulid;

use super::StateStore;
use crate::{
    columns::{
        settlement_attempt_per_wallet::SettlementAttemptPerWalletColumn,
        settlement_attempt_results::SettlementAttemptResultsColumn,
        settlement_attempts::SettlementAttemptsColumn,
        settlement_job_results::SettlementJobResultsColumn, settlement_jobs::SettlementJobsColumn,
    },
    error::Error,
    stores::{SettlementReader, SettlementWriter},
    types::{
        generated::agglayer::storage::v0,
        settlement::{attempt::Key as SettlementAttemptKey, attempt_per_wallet},
    },
};

impl StateStore {
    fn with_settlement_write_lock<T>(
        &self,
        settlement_job_id: &Ulid,
        callback: impl FnOnce() -> Result<T, Error>,
    ) -> Result<T, Error> {
        let key_lock = {
            let mut settlement_write_locks = self.settlement_write_locks.lock().map_err(|_| {
                Error::Unexpected("Settlement write lock map is poisoned".to_string())
            })?;

            settlement_write_locks
                .entry(*settlement_job_id)
                .or_insert_with(|| std::sync::Arc::new(std::sync::Mutex::new(())))
                .clone()
        };

        let _settlement_write_lock = key_lock.lock().map_err(|_| {
            Error::Unexpected(format!(
                "Settlement write lock is poisoned for job {settlement_job_id}",
            ))
        })?;

        callback()
    }

    fn encode_settlement_job(settlement_job: &SettlementJob) -> Result<v0::SettlementJob, Error> {
        v0::SettlementJob::try_from(settlement_job).map_err(|error| {
            Error::Unexpected(format!(
                "Failed to encode settlement job for storage: {error}"
            ))
        })
    }

    fn decode_settlement_job(settlement_job: v0::SettlementJob) -> Result<SettlementJob, Error> {
        SettlementJob::try_from(settlement_job).map_err(|error| {
            Error::Unexpected(format!(
                "Failed to decode settlement job from storage: {error}"
            ))
        })
    }
}

impl SettlementReader for StateStore {
    fn get_settlement_job(&self, settlement_job_id: &Ulid) -> Result<Option<SettlementJob>, Error> {
        self.db
            .get::<SettlementJobsColumn>(settlement_job_id)?
            .map(Self::decode_settlement_job)
            .transpose()
    }

    fn get_settlement_job_result(
        &self,
        settlement_job_id: &Ulid,
    ) -> Result<Option<SettlementJobResult>, Error> {
        self.db
            .get::<SettlementJobResultsColumn>(settlement_job_id)?
            .map(|result| {
                SettlementJobResult::try_from(result).map_err(|error| {
                    Error::Unexpected(format!(
                        "Failed to decode settlement result from storage: {error}"
                    ))
                })
            })
            .transpose()
    }
}

impl SettlementWriter for StateStore {
    fn insert_settlement_job(
        &self,
        settlement_job_id: &Ulid,
        settlement_job: &SettlementJob,
    ) -> Result<(), Error> {
        let settlement_job = Self::encode_settlement_job(settlement_job)?;

        self.with_settlement_write_lock(settlement_job_id, || {
            if self
                .db
                .get::<SettlementJobsColumn>(settlement_job_id)?
                .is_some()
            {
                return Err(Error::UnprocessedAction(format!(
                    "Settlement job already exists for id {settlement_job_id}"
                )));
            }

            Ok(self
                .db
                .put::<SettlementJobsColumn>(settlement_job_id, &settlement_job)?)
        })
    }

    fn insert_settlement_job_result(
        &self,
        settlement_job_id: &Ulid,
        tx_result: &SettlementJobResult,
    ) -> Result<(), Error> {
        let tx_result: v0::TxResult = tx_result.into();

        self.with_settlement_write_lock(settlement_job_id, || {
            if self
                .db
                .get::<SettlementJobResultsColumn>(settlement_job_id)?
                .is_some()
            {
                return Err(Error::UnprocessedAction(format!(
                    "Settlement job result already exists for id {settlement_job_id}"
                )));
            }

            if self
                .db
                .get::<SettlementJobsColumn>(settlement_job_id)?
                .is_none()
            {
                return Err(Error::UnprocessedAction(format!(
                    "Settlement job does not exist for id {settlement_job_id}"
                )));
            }

            Ok(self
                .db
                .put::<SettlementJobResultsColumn>(settlement_job_id, &tx_result)?)
        })
    }

    fn insert_settlement_attempt(
        &self,
        settlement_job_id: &Ulid,
        attempt_sequence_number: u64,
        settlement_attempt: &SettlementAttempt,
    ) -> Result<(), Error> {
        let sender_wallet = settlement_attempt.sender_wallet;
        let nonce = settlement_attempt.nonce.0;
        let proto_settlement_attempt: v0::SettlementAttempt = settlement_attempt.into();

        self.with_settlement_write_lock(settlement_job_id, || -> Result<(), Error> {
            let job_exists = self
                .db
                .get::<SettlementJobsColumn>(settlement_job_id)?
                .is_some();

            if !job_exists {
                return Err(Error::UnprocessedAction(format!(
                    "Settlement job does not exist for id {settlement_job_id}"
                )));
            }

            let key = SettlementAttemptKey {
                settlement_job_id: *settlement_job_id,
                attempt_sequence_number,
            };

            let attempt_exists = self.db.get::<SettlementAttemptsColumn>(&key)?.is_some();
            if attempt_exists {
                return Err(Error::UnprocessedAction(format!(
                    "Settlement attempt already exists for job {settlement_job_id} and attempt \
                     sequence number {attempt_sequence_number}"
                )));
            }

            let address = sender_wallet.into_array();

            let attempt_per_wallet_key = attempt_per_wallet::Key {
                address,
                nonce,
                settlement_job_id: *settlement_job_id,
                attempt_sequence_number,
            };
            let attempt_per_wallet_value = attempt_per_wallet::Value;

            let mut batch = WriteBatch::default();
            self.db.multi_insert_batch::<SettlementAttemptsColumn>(
                [(&key, &proto_settlement_attempt)],
                &mut batch,
            )?;
            self.db
                .multi_insert_batch::<SettlementAttemptPerWalletColumn>(
                    [(&attempt_per_wallet_key, &attempt_per_wallet_value)],
                    &mut batch,
                )?;

            self.db.write_batch(batch)?;

            Ok(())
        })
    }

    fn insert_settlement_attempt_result(
        &self,
        settlement_job_id: &Ulid,
        attempt_sequence_number: u64,
        tx_result: &SettlementAttemptResult,
    ) -> Result<(), Error> {
        let tx_result: v0::TxResult = tx_result.into();

        self.with_settlement_write_lock(settlement_job_id, || {
            let key = SettlementAttemptKey {
                settlement_job_id: *settlement_job_id,
                attempt_sequence_number,
            };

            if self
                .db
                .get::<SettlementAttemptResultsColumn>(&key)?
                .is_some()
            {
                return Err(Error::UnprocessedAction(format!(
                    "Settlement attempt result already exists for job {settlement_job_id} and \
                     attempt sequence number {attempt_sequence_number}"
                )));
            }

            if self.db.get::<SettlementAttemptsColumn>(&key)?.is_none() {
                return Err(Error::UnprocessedAction(format!(
                    "Settlement attempt does not exist for job {settlement_job_id} and attempt \
                     sequence number {attempt_sequence_number}"
                )));
            }

            Ok(self
                .db
                .put::<SettlementAttemptResultsColumn>(&key, &tx_result)?)
        })
    }
}
