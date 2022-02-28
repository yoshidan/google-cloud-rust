use tokio_util::sync::{CancellationToken as InternalCancellationToken};

#[derive(Clone)]
pub struct CancellationToken {
    inner: InternalCancellationToken,
}

impl Default for CancellationToken {
    fn default() -> CancellationToken {
        CancellationToken::new()
    }
}

/// CancellationToken wrapper for tokio_util::sync::CancellationToken for avoiding Dependency Hell
impl CancellationToken {
    /// Creates a new CancellationToken in the non-cancelled state.
    pub fn new() -> Self {
        Self {
            inner: InternalCancellationToken::new(),
        }
    }

    /// Creates a CancellationToken which will get cancelled whenever the current token gets cancelled.
    /// If the current token is already cancelled, the child token will get returned in cancelled state.
    pub fn child_token(&self) -> CancellationToken {
        Self {
            inner: self.inner.child_token(),
        }
    }

    /// Cancel the [`CancellationToken`] and all child tokens which had been
    /// derived from it.
    ///
    /// This will wake up all tasks which are waiting for cancellation.
    pub fn cancel(&self) {
        self.inner.cancel();
    }

    /// Returns `true` if the `CancellationToken` had been cancelled
    pub fn is_cancelled(&self) -> bool {
        self.inner.is_cancelled()
    }

    /// Returns a `Future` that gets fulfilled when cancellation is requested.
    pub async fn cancelled(&self) {
        self.inner.cancelled().await
    }
}
