/// A handle to an async task that can be used to shut it down
pub struct TaskHandle {
    tx: runtime::mpsc::Sender<()>,
    handle: runtime::task::JoinHandle<()>,
}

impl TaskHandle {
    pub fn new(tx: runtime::mpsc::Sender<()>, handle: runtime::task::JoinHandle<()>) -> Self {
        TaskHandle { tx, handle }
    }

    pub async fn shutdown(self) -> Result<(), runtime::task::JoinError> {
        // the task is waiting on the other end of the mpsc, so dropping the sender will kill the task
        drop(self.tx);
        self.handle.await
    }
}
