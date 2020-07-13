
pub type Runtime = tokio::runtime::Runtime;

/// Abstraction over oneshot channels
pub mod oneshot {
    pub type RecvError = tokio::sync::oneshot::error::RecvError;
    pub type Receiver<T> = tokio::sync::oneshot::Receiver<T>;
    pub type Sender<T> = tokio::sync::oneshot::Sender<T>;
    pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
        tokio::sync::oneshot::channel()
    }
}

/// Abstraction over mpsc channels
pub mod mpsc {
    pub use tokio::sync::mpsc::error::RecvError;
    pub use tokio::sync::mpsc::error::SendError;
    pub type Receiver<T> = tokio::sync::mpsc::Receiver<T>;
    pub type Sender<T> = tokio::sync::mpsc::Sender<T>;
    pub fn channel<T>(buffer: usize) -> (Sender<T>, Receiver<T>) {
        tokio::sync::mpsc::channel(buffer)
    }
}

pub mod mutex {
    pub type Mutex<T> = tokio::sync::Mutex<T>;
}

/// Abstraction over important traits
pub mod traits {
    pub use tokio::io::AsyncRead;
    pub use tokio::io::AsyncReadExt;
    pub use tokio::io::AsyncWrite;
    pub use tokio::io::AsyncWriteExt;
}

pub mod task {
    pub type JoinError = tokio::task::JoinError;
    pub type JoinHandle<T> = tokio::task::JoinHandle<T>;

    pub fn spawn<T>(task: T) -> JoinHandle<T::Output> where
        T: std::future::Future + Send + 'static,
        T::Output: Send + 'static,
    {
        tokio::spawn(task)
    }
}

pub mod time {
    pub async fn timeout_at<T>(deadline: std::time::Instant, future: T) -> Option<T::Output> where T: std::future::Future {
        tokio::time::timeout_at(deadline.into(), future).await.ok()
    }
}

pub mod net {
    pub type TcpStream = tokio::net::TcpStream;
    pub type TcpListener = tokio::net::TcpListener;
}

