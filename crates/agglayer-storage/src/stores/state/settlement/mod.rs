use rocksdb::WriteBatch;
use ulid::Ulid;

use self::telemetry::AttemptWriteMetrics;
use super::StateStore;
use crate::{
    columns::{
        settlement_attempt_per_wallet::SettlementAttemptPerWalletColumn,
        settlement_attempt_results::SettlementAttemptResultsColumn,
        settlement_attempts::SettlementAttemptsColumn, settlement_jobs::SettlementJobsColumn,
    },
    error::Error,
    stores::{SettlementReader, SettlementWriter},
    types::{
        generated::agglayer::storage::v0::{SettlementAttempt, SettlementJob, TxResult},
        settlement::{attempt::Key as SettlementAttemptKey, attempt_per_wallet},
    },
};

mod telemetry;

impl StateStore {
    fn lock_settlement_writes(&self) -> Result<std::sync::MutexGuard<'_, ()>, Error> {
        self.settlement_write_lock
            .lock()
            .map_err(|_| Error::Unexpected("Settlement write lock is poisoned".to_string()))
    }
}

impl SettlementReader for StateStore {
    fn get_settlement_job(&self, settlement_job_id: &Ulid) -> Result<Option<SettlementJob>, Error> {
        Ok(self.db.get::<SettlementJobsColumn>(settlement_job_id)?)
    }

    fn get_settlement_attempt(
        &self,
        settlement_job_id: &Ulid,
        attempt_sequence_number: u64,
    ) -> Result<Option<SettlementAttempt>, Error> {
        let key = SettlementAttemptKey {
            settlement_job_id: *settlement_job_id,
            attempt_sequence_number,
        };

        Ok(self.db.get::<SettlementAttemptsColumn>(&key)?)
    }

    fn get_settlement_attempt_result(
        &self,
        settlement_job_id: &Ulid,
        attempt_sequence_number: u64,
    ) -> Result<Option<TxResult>, Error> {
        let key = SettlementAttemptKey {
            settlement_job_id: *settlement_job_id,
            attempt_sequence_number,
        };

        Ok(self.db.get::<SettlementAttemptResultsColumn>(&key)?)
    }
}

impl SettlementWriter for StateStore {
    fn insert_settlement_job(
        &self,
        settlement_job_id: &Ulid,
        settlement_job: &SettlementJob,
    ) -> Result<(), Error> {
        let _settlement_write_lock = self.lock_settlement_writes()?;

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
            .put::<SettlementJobsColumn>(settlement_job_id, settlement_job)?)
    }

    fn insert_settlement_attempt(
        &self,
        settlement_job_id: &Ulid,
        attempt_sequence_number: u64,
        settlement_attempt: &SettlementAttempt,
    ) -> Result<(), Error> {
        let _settlement_write_lock = self.lock_settlement_writes()?;
        let mut metrics = AttemptWriteMetrics::start();

        let job_exists = match self.db.get::<SettlementJobsColumn>(settlement_job_id) {
            Ok(value) => value.is_some(),
            Err(err) => return Err(err.into()),
        };
        if !job_exists {
            return Err(Error::UnprocessedAction(format!(
                "Settlement job does not exist for id {settlement_job_id}"
            )));
        }

        let key = SettlementAttemptKey {
            settlement_job_id: *settlement_job_id,
            attempt_sequence_number,
        };

        let attempt_exists = match self.db.get::<SettlementAttemptsColumn>(&key) {
            Ok(value) => value.is_some(),
            Err(err) => return Err(err.into()),
        };
        if attempt_exists {
            return Err(Error::UnprocessedAction(format!(
                "Settlement attempt already exists for job {settlement_job_id} and attempt \
                 sequence number {attempt_sequence_number}"
            )));
        }

        let sender_wallet = match settlement_attempt.sender_wallet.as_ref() {
            Some(sender_wallet) => sender_wallet,
            None => {
                return Err(Error::UnprocessedAction(
                    "Settlement attempt cannot be stored without sender_wallet".to_string(),
                ));
            }
        };
        let nonce = match settlement_attempt.nonce.as_ref() {
            Some(nonce) => nonce,
            None => {
                return Err(Error::UnprocessedAction(
                    "Settlement attempt cannot be stored without nonce".to_string(),
                ));
            }
        };
        let address: [u8; 20] = match sender_wallet.address.as_ref().try_into() {
            Ok(address) => address,
            Err(_) => {
                return Err(Error::UnprocessedAction(
                    "Settlement attempt sender_wallet must be 20 bytes".to_string(),
                ));
            }
        };

        let attempt_per_wallet_key = attempt_per_wallet::Key {
            address,
            nonce: nonce.nonce,
            settlement_job_id: *settlement_job_id,
            attempt_sequence_number,
        };
        let attempt_per_wallet_value = attempt_per_wallet::Value;

        let mut batch = WriteBatch::default();
        if let Err(err) = self.db.multi_insert_batch::<SettlementAttemptsColumn>(
            [(&key, settlement_attempt)],
            &mut batch,
        ) {
            return Err(err.into());
        }
        if let Err(err) = self
            .db
            .multi_insert_batch::<SettlementAttemptPerWalletColumn>(
                [(&attempt_per_wallet_key, &attempt_per_wallet_value)],
                &mut batch,
            )
        {
            return Err(err.into());
        }
        if let Err(err) = self.db.write_batch(batch) {
            return Err(err.into());
        }

        metrics.mark_success();
        Ok(())
    }

    fn insert_settlement_attempt_result(
        &self,
        settlement_job_id: &Ulid,
        attempt_sequence_number: u64,
        tx_result: &TxResult,
    ) -> Result<(), Error> {
        let _settlement_write_lock = self.lock_settlement_writes()?;

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
                "Settlement attempt result already exists for job {settlement_job_id} and attempt \
                 sequence number {attempt_sequence_number}"
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
            .put::<SettlementAttemptResultsColumn>(&key, tx_result)?)
    }
}
