#[derive(Debug, thiserror::Error)]
pub enum NotifierError {
    #[error("unable to build notifier")]
    UnableToBuildNotifier,
}
