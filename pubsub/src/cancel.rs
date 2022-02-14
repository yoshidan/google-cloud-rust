use tokio::sync::watch;

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

    pub async fn wait(&mut self) {
       self.cancel.changed().await;
    }
}