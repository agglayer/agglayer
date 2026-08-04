use agglayer_types::{
    Address, CertificateId, ClientError, ClientErrorType, Nonce, SettlementAttempt,
    SettlementAttemptResult, SettlementJob, SettlementJobId, SettlementJobResult,
};
use rocksdb::{Direction, WriteBatch};
use tracing::warn;

use super::StateStore;
use crate::{
    columns::{
        certificate_id_per_settlement_job_id::CertificateIdPerSettlementJobIdColumn,
        settlement_attempt_per_wallet::SettlementAttemptPerWalletColumn,
        settlement_attempt_results::SettlementAttemptResultsColumn,
        settlement_attempts::SettlementAttemptsColumn,
        settlement_job_id_per_certificate_id::SettlementJobIdPerCertificateIdColumn,
        settlement_job_results::SettlementJobResultsColumn, settlement_jobs::SettlementJobsColumn,
    },
    error::Error,
    stores::{EditEvenIfCompleted, SettlementReader, SettlementWriter},
    types::{
        generated::agglayer::storage::v0,
        settlement::{attempt::Key as SettlementAttemptKey, attempt_per_wallet},
    },
};

impl StateStore {
    fn with_settlement_write_lock<T>(
        &self,
        settlement_job_id: &SettlementJobId,
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

    /// Writes a settlement attempt and its per-wallet index entry in one
    /// batch, refusing to overwrite an existing attempt. The caller must hold
    /// the job's settlement write lock and have checked its other
    /// preconditions.
    fn write_settlement_attempt_locked(
        &self,
        settlement_job_id: &SettlementJobId,
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

        let proto_settlement_attempt: v0::SettlementAttempt = settlement_attempt.into();

        let attempt_per_wallet_key = attempt_per_wallet::Key {
            address: settlement_attempt.sender_wallet.into_array(),
            nonce: settlement_attempt.nonce.0,
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
    }

    /// Fails unless the settlement attempt at `key` exists. The caller must
    /// hold the job's settlement write lock.
    fn check_settlement_attempt_exists_locked(
        &self,
        key: &SettlementAttemptKey,
    ) -> Result<(), Error> {
        if self.db.get::<SettlementAttemptsColumn>(key)?.is_none() {
            return Err(Error::SettlementAttemptNotFound {
                job: key.settlement_job_id,
                attempt: key.attempt_sequence_number,
            });
        }

        Ok(())
    }

    /// Fails unless `settlement_job_id` exists and is open to admin attempt
    /// edits: it has no terminal result yet, or `edit_even_if_completed`
    /// forces the edit through. The caller must hold the job's settlement
    /// write lock.
    fn check_settlement_job_is_editable_locked(
        &self,
        settlement_job_id: &SettlementJobId,
        edit_even_if_completed: EditEvenIfCompleted,
    ) -> Result<(), Error> {
        if self
            .db
            .get::<SettlementJobsColumn>(settlement_job_id)?
            .is_none()
        {
            return Err(Error::SettlementJobNotFound(*settlement_job_id));
        }

        if edit_even_if_completed == EditEvenIfCompleted::No
            && self
                .db
                .get::<SettlementJobResultsColumn>(settlement_job_id)?
                .is_some()
        {
            return Err(Error::SettlementJobAlreadyCompleted(*settlement_job_id));
        }

        Ok(())
    }
}

impl SettlementReader for StateStore {
    fn list_settlement_job_ids(&self) -> Result<Vec<SettlementJobId>, Error> {
        Ok(self
            .db
            .keys::<SettlementJobsColumn>()?
            .collect::<Result<Vec<_>, _>>()?)
    }

    fn get_settlement_job(
        &self,
        settlement_job_id: &SettlementJobId,
    ) -> Result<Option<SettlementJob>, Error> {
        Ok(self
            .db
            .get::<SettlementJobsColumn>(settlement_job_id)?
            .map(SettlementJob::try_from)
            .transpose()?)
    }

    fn get_settlement_job_result(
        &self,
        settlement_job_id: &SettlementJobId,
    ) -> Result<Option<SettlementJobResult>, Error> {
        Ok(self
            .db
            .get::<SettlementJobResultsColumn>(settlement_job_id)?
            .map(SettlementJobResult::try_from)
            .transpose()?)
    }

    fn list_settlement_attempts(
        &self,
        settlement_job_id: &SettlementJobId,
    ) -> Result<Vec<(u64, SettlementAttempt)>, Error> {
        self.db
            .prefix_iterator::<SettlementAttemptsColumn, _>(settlement_job_id)?
            .map(|entry| -> Result<(u64, SettlementAttempt), Error> {
                let (key, attempt) = entry?;

                Ok((
                    key.attempt_sequence_number,
                    SettlementAttempt::try_from(attempt)?,
                ))
            })
            .collect::<Result<Vec<_>, _>>()
    }

    fn list_settlement_attempt_results(
        &self,
        settlement_job_id: &SettlementJobId,
    ) -> Result<Vec<(u64, SettlementAttemptResult)>, Error> {
        self.db
            .prefix_iterator::<SettlementAttemptResultsColumn, _>(settlement_job_id)?
            .map(|entry| -> Result<(u64, SettlementAttemptResult), Error> {
                let (key, result) = entry?;

                Ok((
                    key.attempt_sequence_number,
                    SettlementAttemptResult::try_from(result)?,
                ))
            })
            .collect::<Result<Vec<_>, _>>()
    }

    fn max_settlement_nonce_for_wallet(&self, wallet: Address) -> Result<Option<Nonce>, Error> {
        let prefix = wallet.into_array();
        Ok(self
            .db
            .prefix_iterator_with_direction::<SettlementAttemptPerWalletColumn, _>(
                &prefix,
                Direction::Reverse,
            )?
            .next()
            .transpose()?
            .map(|(key, _)| Nonce(key.nonce)))
    }
}

impl SettlementWriter for StateStore {
    fn insert_settlement_job(
        &self,
        settlement_job_id: &SettlementJobId,
        settlement_job: &SettlementJob,
    ) -> Result<(), Error> {
        let settlement_job: v0::SettlementJob = settlement_job.into();

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

    fn insert_settlement_job_with_certificate(
        &self,
        settlement_job_id: &SettlementJobId,
        settlement_job: &SettlementJob,
        certificate_id: &CertificateId,
    ) -> Result<(), Error> {
        let settlement_job: v0::SettlementJob = settlement_job.into();

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
            if self
                .db
                .get::<SettlementJobIdPerCertificateIdColumn>(certificate_id)?
                .is_some()
            {
                return Err(Error::UnprocessedAction(format!(
                    "Certificate {certificate_id} already has a settlement job id"
                )));
            }

            let mut batch = WriteBatch::default();
            self.db.multi_insert_batch::<SettlementJobsColumn>(
                [(settlement_job_id, &settlement_job)],
                &mut batch,
            )?;
            self.db
                .multi_insert_batch::<SettlementJobIdPerCertificateIdColumn>(
                    [(certificate_id, settlement_job_id)],
                    &mut batch,
                )?;
            self.db
                .multi_insert_batch::<CertificateIdPerSettlementJobIdColumn>(
                    [(settlement_job_id, certificate_id)],
                    &mut batch,
                )?;
            Ok(self.db.write_batch(batch)?)
        })
    }

    fn insert_settlement_job_result(
        &self,
        settlement_job_id: &SettlementJobId,
        tx_result: &SettlementJobResult,
    ) -> Result<(), Error> {
        let tx_result: v0::SettlementJobResult = tx_result.into();

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
        settlement_job_id: &SettlementJobId,
        attempt_sequence_number: u64,
        settlement_attempt: &SettlementAttempt,
    ) -> Result<(), Error> {
        self.with_settlement_write_lock(settlement_job_id, || -> Result<(), Error> {
            let job_exists = self
                .db
                .get::<SettlementJobsColumn>(settlement_job_id)?
                .is_some();

            if !job_exists {
                return Err(Error::SettlementJobNotFound(*settlement_job_id));
            }

            self.write_settlement_attempt_locked(
                settlement_job_id,
                attempt_sequence_number,
                settlement_attempt,
            )
        })
    }

    fn record_settlement_attempt_result(
        &self,
        settlement_job_id: &SettlementJobId,
        attempt_sequence_number: u64,
        tx_result: &SettlementAttemptResult,
    ) -> Result<(), Error> {
        let proto_tx_result: v0::SettlementAttemptResult = tx_result.into();

        self.with_settlement_write_lock(settlement_job_id, || {
            let key = SettlementAttemptKey {
                settlement_job_id: *settlement_job_id,
                attempt_sequence_number,
            };

            self.check_settlement_attempt_exists_locked(&key)?;

            if let Some(stored_result) = self
                .db
                .get::<SettlementAttemptResultsColumn>(&key)?
                .map(SettlementAttemptResult::try_from)
                .transpose()?
            {
                if stored_result == *tx_result {
                    return Ok(());
                }

                if !stored_result.can_be_replaced_by(tx_result) {
                    // An admin abandon assertion outranks any client-side
                    // note derived from pre-override state; dropping the
                    // weaker write (and reporting success) keeps a task that
                    // has not reloaded yet from wedging on a conflict it
                    // cannot resolve (cf. #1617). On-chain evidence still
                    // replaces the assertion through `can_be_replaced_by`.
                    if matches!(
                        &stored_result,
                        SettlementAttemptResult::ClientError(ClientError {
                            kind: ClientErrorType::AbandonedByAdmin,
                            ..
                        })
                    ) && matches!(tx_result, SettlementAttemptResult::ClientError(_))
                    {
                        warn!(
                            %settlement_job_id,
                            attempt_sequence_number,
                            ?tx_result,
                            "Kept admin-abandoned settlement attempt result, dropped weaker \
                             client-side write"
                        );
                        return Ok(());
                    }

                    return Err(Error::UnprocessedAction(format!(
                        "Cannot replace existing settlement attempt result {stored_result:?} with \
                         new settlement attempt result {tx_result:?} for job {settlement_job_id} \
                         and attempt sequence number {attempt_sequence_number}",
                    )));
                }
            }

            Ok(self
                .db
                .put::<SettlementAttemptResultsColumn>(&key, &proto_tx_result)?)
        })
    }

    fn admin_insert_settlement_attempt(
        &self,
        settlement_job_id: &SettlementJobId,
        settlement_attempt: &SettlementAttempt,
        edit_even_if_completed: EditEvenIfCompleted,
    ) -> Result<u64, Error> {
        self.with_settlement_write_lock(settlement_job_id, || -> Result<u64, Error> {
            self.check_settlement_job_is_editable_locked(
                settlement_job_id,
                edit_even_if_completed,
            )?;

            let attempt_sequence_number = match self
                .db
                .prefix_iterator_with_direction::<SettlementAttemptsColumn, _>(
                    settlement_job_id,
                    Direction::Reverse,
                )?
                .next()
                .transpose()?
            {
                None => 0,
                Some((key, _)) => key.attempt_sequence_number.checked_add(1).ok_or_else(|| {
                    Error::UnprocessedAction(format!(
                        "Settlement attempt sequence numbers are exhausted for job \
                         {settlement_job_id}"
                    ))
                })?,
            };

            self.write_settlement_attempt_locked(
                settlement_job_id,
                attempt_sequence_number,
                settlement_attempt,
            )?;

            Ok(attempt_sequence_number)
        })
    }

    fn admin_override_settlement_attempt_result(
        &self,
        settlement_job_id: &SettlementJobId,
        attempt_number: u64,
        tx_result: &SettlementAttemptResult,
        edit_even_if_completed: EditEvenIfCompleted,
    ) -> Result<(), Error> {
        self.with_settlement_write_lock(settlement_job_id, || {
            self.check_settlement_job_is_editable_locked(
                settlement_job_id,
                edit_even_if_completed,
            )?;

            let key = SettlementAttemptKey {
                settlement_job_id: *settlement_job_id,
                attempt_sequence_number: attempt_number,
            };

            self.check_settlement_attempt_exists_locked(&key)?;

            let proto_tx_result: v0::SettlementAttemptResult = tx_result.into();
            Ok(self
                .db
                .put::<SettlementAttemptResultsColumn>(&key, &proto_tx_result)?)
        })
    }

    fn admin_remove_settlement_attempt_result(
        &self,
        settlement_job_id: &SettlementJobId,
        attempt_number: u64,
        edit_even_if_completed: EditEvenIfCompleted,
    ) -> Result<(), Error> {
        self.with_settlement_write_lock(settlement_job_id, || {
            self.check_settlement_job_is_editable_locked(
                settlement_job_id,
                edit_even_if_completed,
            )?;

            let key = SettlementAttemptKey {
                settlement_job_id: *settlement_job_id,
                attempt_sequence_number: attempt_number,
            };

            self.check_settlement_attempt_exists_locked(&key)?;

            if self
                .db
                .get::<SettlementAttemptResultsColumn>(&key)?
                .is_none()
            {
                return Err(Error::SettlementAttemptResultNotRecorded {
                    job: *settlement_job_id,
                    attempt: attempt_number,
                });
            }

            Ok(self.db.delete::<SettlementAttemptResultsColumn>(&key)?)
        })
    }
}
