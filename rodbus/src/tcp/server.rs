use std::collections::BTreeMap;
use std::sync::Arc;

use std::net::SocketAddr;

use runtime::net::TcpListener;
use runtime::mutex::Mutex;

use futures_util::future::FutureExt;

use crate::server::handler::{ServerHandler, ServerHandlerMap};

struct SessionTracker {
    max: usize,
    id: u64,
    sessions: BTreeMap<u64, runtime::mpsc::Sender<()>>,
}

type SessionTrackerWrapper = Arc<Mutex<Box<SessionTracker>>>;

impl SessionTracker {
    fn new(max: usize) -> SessionTracker {
        Self {
            max,
            id: 0,
            sessions: BTreeMap::new(),
        }
    }

    fn get_next_id(&mut self) -> u64 {
        let ret = self.id;
        self.id += 1;
        ret
    }

    pub(crate) fn wrapped(max: usize) -> SessionTrackerWrapper {
        Arc::new(Mutex::new(Box::new(Self::new(max))))
    }

    pub(crate) fn add(&mut self, sender: runtime::mpsc::Sender<()>) -> u64 {
        // TODO - this is so ugly. there's a nightly API on BTreeMap that has a remove_first
        if !self.sessions.is_empty() && self.sessions.len() >= self.max {
            let id = *self.sessions.keys().next().unwrap();
            log::warn!("exceeded max connections, closing oldest session: {}", id);
            // when the record drops, and there are no more senders,
            // the other end will stop the task
            self.sessions.remove(&id).unwrap();
        }

        let id = self.get_next_id();
        self.sessions.insert(id, sender);
        id
    }

    pub(crate) fn remove(&mut self, id: u64) {
        self.sessions.remove(&id);
    }
}

pub(crate) struct ServerTask<T: ServerHandler> {
    listener: TcpListener,
    handlers: ServerHandlerMap<T>,
    tracker: SessionTrackerWrapper,
    shutdown: runtime::mpsc::Receiver<()>,
}

impl<T> ServerTask<T>
where
    T: ServerHandler,
{
    pub(crate) fn new(
        max_sessions: usize,
        listener: TcpListener,
        handlers: ServerHandlerMap<T>,
        shutdown: runtime::mpsc::Receiver<()>,
    ) -> Self {
        Self {
            listener,
            handlers,
            tracker: SessionTracker::wrapped(max_sessions),
            shutdown
        }
    }

    pub(crate) async fn run(&mut self) {
        loop {
            match self.accept().await {
                Err(_) => {
                    return;
                }
                Ok((stream, addr)) =>  {
                    self.handle(stream, addr).await
                }
            }
        }
    }

    async fn accept(&mut self) -> Result<(runtime::net::TcpStream, SocketAddr), ()> {
        futures_util::select! {
             _ = self.shutdown.recv().fuse() => {
                 Err(())
             }
             res = self.listener.accept().fuse() => {
                 match res {
                     Ok(x) => Ok(x),
                     Err(_) => Err(())
                 }
             }
        }
    }

    async fn handle(&self, socket: runtime::net::TcpStream, addr: SocketAddr) {
        let handlers = self.handlers.clone();
        let tracker = self.tracker.clone();
        let (tx, rx) = runtime::mpsc::channel(1);

        let id = self.tracker.lock().await.add(tx);

        log::info!("accepted connection {} from: {}", id, addr);

        runtime::task::spawn(async move {
            crate::server::task::SessionTask::new(socket, handlers, rx)
                .run()
                .await
                .ok();
            log::info!("shutdown session: {}", id);
            tracker.lock().await.remove(id);
        });
    }
}
