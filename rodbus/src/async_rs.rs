/// Abstraction over oneshot channels
pub(crate) mod oneshot {
    pub(crate) type RecvError = tokio::sync::oneshot::error::RecvError;
    pub(crate) type Receiver<T> = tokio::sync::oneshot::Receiver<T>;
    pub(crate) type Sender<T> = tokio::sync::oneshot::Sender<T>;
    pub(crate) fn channel<T>() -> (Sender<T>, Receiver<T>) {
        tokio::sync::oneshot::channel()
    }
}

/// Abstraction over mpsc channels
pub(crate) mod mpsc {
    pub(crate) type RecvError = tokio::sync::mpsc::error::RecvError;
    pub(crate) type SendError<T> = tokio::sync::mpsc::error::SendError<T>;
    pub(crate) type Receiver<T> = tokio::sync::mpsc::Receiver<T>;
    pub(crate) type Sender<T> = tokio::sync::mpsc::Sender<T>;
    pub(crate) fn channel<T>(buffer: usize) -> (Sender<T>, Receiver<T>) {
        tokio::sync::mpsc::channel(buffer)
    }
}

/// Abstraction over important traits
pub(crate) mod traits {
    pub(crate) use tokio::io::AsyncRead;
    pub(crate) use tokio::io::AsyncReadExt;
    pub(crate) use tokio::io::AsyncWrite;
    pub(crate) use tokio::io::AsyncWriteExt;
}
