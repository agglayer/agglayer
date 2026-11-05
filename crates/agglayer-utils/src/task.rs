//! Task-spawning helpers.

use tokio::task::JoinHandle;

/// `tokio::task::spawn_blocking`, with the caller's tracing spans carried
/// into the blocking closure.
///
/// Blocking tasks start with an empty span stack, so events emitted inside
/// them are not attributed to the operation that spawned them and lose every
/// `#[instrument]` field. This wrapper enters the span active at the call
/// site for the whole run of the closure.
pub fn spawn_blocking_in_current_span<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let span = tracing::Span::current();
    tokio::task::spawn_blocking(move || span.in_scope(f))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn blocking_closure_runs_inside_the_caller_span() {
        // Global rather than thread-local: the blocking closure resolves
        // `Span::current()` through its own thread's dispatcher.
        tracing::subscriber::set_global_default(tracing_subscriber::registry())
            .expect("no other test in this binary may set the global subscriber");

        let span = tracing::info_span!("caller");
        assert!(span.id().is_some(), "the subscriber must enable the span");

        let handle =
            span.in_scope(|| spawn_blocking_in_current_span(|| tracing::Span::current().id()));

        assert_eq!(handle.await.expect("blocking task panicked"), span.id());
    }
}
