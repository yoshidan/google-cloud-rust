use tokio::sync::watch;

#[derive(Clone)]
pub struct CancellationToken {
    cancel: watch::Receiver<bool>
}

impl CancellationToken {
    pub fn new() -> (Self, impl FnOnce() + Send + Sync + 'static)  {
        let (mut sender, receiver) = watch::channel::<bool>(false);
        (Self {
            cancel: receiver
        }, move || {
            sender.send(true);
        })
    }

    // return if sender closed or sender first published
    pub async fn done(&mut self) {
       self.cancel.changed().await;
    }
}