use std::ffi::CStr;
use std::net::SocketAddr;
use std::ptr::{null_mut, null};
use std::str::FromStr;
use tokio::runtime;
use std::os::raw::c_void;
use rodbus::types::{UnitId, AddressRange};
use rodbus::client::session::CallbackSession;
use rodbus::error::ErrorKind;

#[repr(u8)]
pub enum Status {
    Ok,
    Shutdown,
    NoConnection,
    ResponseTimeout,
    BadRequest,
    BadResponse,
    IOError,
    BadFraming,
    Exception,
    InternalError
}

impl std::convert::From<&ErrorKind> for Status {
    fn from(err: &ErrorKind) -> Self {
        match err {
            ErrorKind::Bug(_) => Status::InternalError,
            ErrorKind::NoConnection => Status::NoConnection,
            ErrorKind::BadFrame(_) => Status::BadFraming,
            ErrorKind::Shutdown => Status::Shutdown,
            ErrorKind::ResponseTimeout => Status::ResponseTimeout,
            ErrorKind::BadRequest(_) => Status::BadRequest,
            ErrorKind::Exception(_) => Status::Exception,
            ErrorKind::Io(_) => Status::IOError,
            ErrorKind::BadResponse(_) => Status::BadResponse,
            _ => Status::InternalError,
        }
    }
}

struct ContextStorage {
    context: *mut c_void
}

// we need these so we can send the callback context to the executor
// we rely on the C program to keep the context value alive
// for the duration of the operation, and for it to be thread-safe
unsafe impl Send for ContextStorage {}
unsafe impl Sync for ContextStorage {}

#[no_mangle]
pub extern "C" fn create_runtime() -> *mut tokio::runtime::Runtime {
    match runtime::Builder::new().enable_all().threaded_scheduler().build() {
        Ok(r) => Box::into_raw(Box::new(r)),
        Err(_) => null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn destroy_runtime(runtime: *mut tokio::runtime::Runtime) {
    if runtime != null_mut() {
        unsafe { Box::from_raw(runtime) };
    };
    ()
}

#[no_mangle]
pub extern "C" fn create_tcp_client(
    runtime: *mut tokio::runtime::Runtime,
    address: *const std::os::raw::c_char,
    max_queued_requests: usize,
) -> *mut rodbus::client::channel::Channel {
    // if we can't turn the c-string into SocketAddr, return null
    let addr = {
        match unsafe { CStr::from_ptr(address) }.to_str() {
            // TODO - consider logging?
            Err(_) => return null_mut(),
            Ok(s) => match SocketAddr::from_str(s) {
                // TODO - consider logging?
                Err(_) => return null_mut(),
                Ok(addr) => addr,
            },
        }
    };

    let (handle, task) = rodbus::client::channel::Channel::create_handle_and_task(
        addr,
        max_queued_requests,
        rodbus::client::channel::strategy::default(),
    );

    unsafe {
        (*runtime).spawn(task);
    }

    Box::into_raw(Box::new(handle))
}

#[no_mangle]
pub extern "C" fn read_coils(
    channel: *mut rodbus::client::channel::Channel,
    unit_id: u8,
    start: u16,
    count: u16,
    callback: fn(Status, *const bool, usize, *mut c_void),
    timeout_ms: u32,
    context: *mut c_void
) {
    let mut session : CallbackSession = unsafe {
        CallbackSession::new((*channel).create_session(UnitId::new(unit_id), std::time::Duration::from_millis(timeout_ms as u64)))
    };

    let storage = ContextStorage { context };

    session.read_coils(AddressRange::new(start, count), move |result| {
        match result {
            Err(err) => {
                callback(err.kind().into(), null(), 0, storage.context)
            }
            Ok(values) => {
                let transformed : Vec<bool> = values.iter().map(|x| x.value).collect();
                callback(Status::Ok, transformed.as_ptr(), transformed.len(), storage.context)
            }
        }
    });
}

#[no_mangle]
pub extern "C" fn destroy_tcp_client(client: *mut rodbus::client::channel::Channel) {
    if client != null_mut() {
        unsafe { Box::from_raw(client) };
    };
    ()
}
