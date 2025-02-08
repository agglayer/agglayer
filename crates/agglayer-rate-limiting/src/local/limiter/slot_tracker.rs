/// Represents a single reserved rate limiting slot.
///
/// If this goes out of scope, it checks the tracked slots have been properly
/// released into the rate limiter state.
#[must_use = "Dropping slot without releasing it"]
#[derive(Debug)]
pub struct SlotTracker(usize);

impl SlotTracker {
    pub(super) const fn new() -> Self {
        Self(1)
    }

    #[must_use = "Slot has to be released from the limiter"]
    pub(super) fn release(mut self) -> usize {
        let num_slots = self.0;
        self.0 = 0;
        num_slots
    }

    pub fn take(&mut self) -> Self {
        let num_slots = self.0;
        self.0 = 0;
        Self(num_slots)
    }
}

impl Drop for SlotTracker {
    fn drop(&mut self) {
        agglayer_utils::log_assert_eq!(self.0, 0, "slots not released");
    }
}

#[cfg(test)]
mod tests {
    use super::SlotTracker;

    #[test]
    #[should_panic = "slots not released"]
    fn drop_without_release() {
        let _slot = SlotTracker::new();
    }

    #[test]
    fn release() {
        assert_eq!(SlotTracker::new().release(), 1);
    }

    #[test]
    fn take_and_release() {
        let mut slot0 = SlotTracker::new();
        let slot1 = slot0.take();
        assert_eq!(slot0.0, 0);
        assert_eq!(slot1.release(), 1);
    }
}
