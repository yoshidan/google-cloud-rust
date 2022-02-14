use tokio::sync::watch;

/// ```
/// use google_cloud_pubsub::cancel::CancellationToken;
/// let (mut token, cancel) = CancellationToken::new();
/// tokio::spawn(async move {
///     //wait for cancel
///     token.done().await;
/// });
/// // cancel
/// drop(cancel);
/// ```
#[derive(Clone)]
pub struct CancellationToken {
    cancel: watch::Receiver<bool>
}

impl CancellationToken {
    pub fn new() -> (Self, impl Drop)  {
        let (sender, receiver) = watch::channel::<bool>(false);
        (Self {
            cancel: receiver
        },  sender)
    }

    // return if sender closed or sender first published
    pub async fn done(&mut self) {
       self.cancel.changed().await;
    }
}