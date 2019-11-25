use crate::{Error, LogicError};

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Stdio(err)
    }
}

impl std::convert::From<LogicError> for Error {
    fn from(err: crate::LogicError) -> Self {
        Error::Logic(err)
    }
}

impl std::convert::From<std::num::TryFromIntError> for Error {
    fn from(_: std::num::TryFromIntError) -> Self {
        Error::Logic(LogicError::BadWriteSize)
    }
}

impl std::convert::From<tokio::sync::mpsc::error::SendError> for Error {
    fn from(_: tokio::sync::mpsc::error::SendError) -> Self {
        Error::ChannelClosed
    }
}

impl std::convert::From<tokio::sync::oneshot::error::RecvError> for Error {
    fn from(_: tokio::sync::oneshot::error::RecvError) -> Self {
        Error::ChannelClosed
    }
}