use std::time::Instant;

pub(super) struct LatestAttemptReadMetrics {
    started_at: Instant,
    success: bool,
    found: bool,
}

impl LatestAttemptReadMetrics {
    pub(super) fn start() -> Self {
        Self {
            started_at: Instant::now(),
            success: false,
            found: false,
        }
    }

    pub(super) fn mark_success(&mut self, found: bool) {
        self.success = true;
        self.found = found;
    }
}

impl Drop for LatestAttemptReadMetrics {
    fn drop(&mut self) {
        agglayer_telemetry::settlement_storage::record_latest_attempt_read(
            self.started_at.elapsed(),
            self.found,
            self.success,
        );
    }
}

pub(super) struct AttemptWriteMetrics {
    started_at: Instant,
    success: bool,
}

impl AttemptWriteMetrics {
    pub(super) fn start() -> Self {
        Self {
            started_at: Instant::now(),
            success: false,
        }
    }

    pub(super) fn mark_success(&mut self) {
        self.success = true;
    }
}

impl Drop for AttemptWriteMetrics {
    fn drop(&mut self) {
        agglayer_telemetry::settlement_storage::record_attempt_write(
            self.started_at.elapsed(),
            self.success,
        );
    }
}
