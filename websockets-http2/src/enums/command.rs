use tokio::sync::oneshot;

#[derive(Debug)]
pub enum Command {
    Increment,
    GetCount { respond_to: oneshot::Sender<u32> },
}
