use rocksdb::{Direction, ReadOptions, WriteBatch};
use ulid::Ulid;

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

    fn get_latest_settlement_attempt_sequence_number(
        &self,
        settlement_job_id: &Ulid,
    ) -> Result<Option<u64>, Error> {
        let mut iter = self.db.iter_with_direction::<SettlementAttemptsColumn>(
            ReadOptions::default(),
            Direction::Reverse,
        )?;
        iter.seek_for_prev(&SettlementAttemptKey {
            settlement_job_id: *settlement_job_id,
            attempt_sequence_number: u64::MAX,
        })?;

        Ok(match iter.next().transpose()? {
            Some((key, _)) if key.settlement_job_id == *settlement_job_id => {
                Some(key.attempt_sequence_number)
            }
            _ => None,
        })
    }
}

impl SettlementWriter for StateStore {
    fn insert_settlement_job(
        &self,
        settlement_job_id: &Ulid,
        settlement_job: &SettlementJob,
    ) -> Result<(), Error> {
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
        if self
            .db
            .get::<SettlementJobsColumn>(settlement_job_id)?
            .is_none()
        {
            return Err(Error::UnprocessedAction(format!(
                "Settlement job does not exist for id {settlement_job_id}"
            )));
        }

        let key = SettlementAttemptKey {
            settlement_job_id: *settlement_job_id,
            attempt_sequence_number,
        };

        if self.db.get::<SettlementAttemptsColumn>(&key)?.is_some() {
            return Err(Error::UnprocessedAction(format!(
                "Settlement attempt already exists for job {settlement_job_id} and attempt \
                 sequence number {attempt_sequence_number}"
            )));
        }

        let sender_wallet = settlement_attempt.sender_wallet.as_ref().ok_or_else(|| {
            Error::UnprocessedAction(
                "Settlement attempt cannot be stored without sender_wallet".to_string(),
            )
        })?;
        let nonce = settlement_attempt.nonce.as_ref().ok_or_else(|| {
            Error::UnprocessedAction(
                "Settlement attempt cannot be stored without nonce".to_string(),
            )
        })?;
        let address: [u8; 20] = sender_wallet.address.as_ref().try_into().map_err(|_| {
            Error::UnprocessedAction(
                "Settlement attempt sender_wallet must be 20 bytes".to_string(),
            )
        })?;

        let attempt_per_wallet_key = attempt_per_wallet::Key {
            address,
            nonce: nonce.nonce,
            settlement_job_id: *settlement_job_id,
            attempt_sequence_number,
        };
        let attempt_per_wallet_value = attempt_per_wallet::Value;

        let mut batch = WriteBatch::default();
        self.db.multi_insert_batch::<SettlementAttemptsColumn>(
            [(&key, settlement_attempt)],
            &mut batch,
        )?;
        self.db
            .multi_insert_batch::<SettlementAttemptPerWalletColumn>(
                [(&attempt_per_wallet_key, &attempt_per_wallet_value)],
                &mut batch,
            )?;

        Ok(self.db.write_batch(batch)?)
    }

    fn insert_settlement_attempt_result(
        &self,
        settlement_job_id: &Ulid,
        attempt_sequence_number: u64,
        tx_result: &TxResult,
    ) -> Result<(), Error> {
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
