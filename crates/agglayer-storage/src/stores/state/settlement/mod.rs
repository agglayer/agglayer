use rocksdb::{Direction, ReadOptions};
use ulid::Ulid;

use super::StateStore;
use crate::{
    columns::{
        settlement_attempt_results::SettlementAttemptResultsColumn,
        settlement_attempts::SettlementAttemptsColumn, settlement_jobs::SettlementJobsColumn,
    },
    error::Error,
    stores::{SettlementReader, SettlementWriter},
    types::{
        generated::agglayer::storage::v0::{SettlementAttempt, SettlementJob, TxResult},
        settlement::attempt::Key as SettlementAttemptKey,
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
        Ok(self
            .db
            .iter_with_direction::<SettlementAttemptsColumn>(
                ReadOptions::default(),
                Direction::Reverse,
            )?
            .find_map(|entry| match entry {
                Ok((key, _)) if key.settlement_job_id == *settlement_job_id => {
                    Some(Ok(key.attempt_sequence_number))
                }
                Ok(_) => None,
                Err(err) => Some(Err(err)),
            })
            .transpose()?)
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

        Ok(self
            .db
            .put::<SettlementAttemptsColumn>(&key, settlement_attempt)?)
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

        Ok(self
            .db
            .put::<SettlementAttemptResultsColumn>(&key, tx_result)?)
    }
}
