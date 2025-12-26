/// Internal configuration for settlement tasks.
/// This is not exposed to users and is configured programmatically by the
/// service.
pub struct SettlementTaskConfig {}

pub struct SettlementTask {}

impl SettlementTask {
    pub fn new(_config: SettlementTaskConfig) -> Self {
        Self {}
    }
}
