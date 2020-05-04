use tokio::net::TcpListener;

use crate::server::handler::{ServerHandler, ServerHandlerMap};
use crate::shutdown::TaskHandle;
use crate::tcp::server::ServerTask;
use tracing_futures::Instrument;

/// server handling
pub mod handler;
pub(crate) mod request;
pub(crate) mod response;
pub(crate) mod task;
pub(crate) mod validator;

/// Spawns a TCP server task onto the runtime. This method can only
/// be called from within the runtime context. Use [`create_tcp_server_task`]
/// and then spawn it manually if using outside the Tokio runtime.
///
/// Each incoming connection will spawn a new task to handle it.
///
/// * `max_sessions` - Maximum number of concurrent sessions
/// * `listener` - A bound TCP listener used to accept connections
/// * `handlers` - A map of handlers keyed by a unit id
///
/// [`create_tcp_server_task`]: fn.create_tcp_server_task.html
pub fn spawn_tcp_server_task<T: ServerHandler>(
    max_sessions: usize,
    listener: TcpListener,
    handlers: ServerHandlerMap<T>,
) -> TaskHandle {
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    let handle = tokio::spawn(create_tcp_server_task(rx, max_sessions, listener, handlers));
    TaskHandle::new(tx, handle)
}

/// Creates a TCP server task that can then be spawned onto the runtime manually.
/// Most users will prefer [`spawn_tcp_server_task`] unless they are using the library from
/// outside the Tokio runtime and need to spawn it using a Runtime handle instead of the
/// `tokio::spawn` function.
///
/// Each incoming connection will spawn a new task to handle it.
///
/// * `max_sessions` - Maximum number of concurrent sessions
/// * `listener` - A bound TCP listener used to accept connections
/// * `handlers` - A map of handlers keyed by a unit id
///
/// [`spawn_tcp_server_task`]: fn.spawn_tcp_server_task.html
pub async fn create_tcp_server_task<T: ServerHandler>(
    rx: tokio::sync::mpsc::Receiver<()>,
    max_sessions: usize,
    listener: TcpListener,
    handlers: ServerHandlerMap<T>,
) {
    let endpoint = listener.local_addr().unwrap();
    ServerTask::new(max_sessions, listener, handlers)
        .run(rx)
        .instrument(tracing::span!(
            tracing::Level::INFO,
            "server",
            "{}",
            endpoint
        ))
        .await
}
