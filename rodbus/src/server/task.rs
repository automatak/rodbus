use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

use crate::common::cursor::ReadCursor;
use crate::common::frame::{Frame, FrameFormatter, FrameHeader, FramedReader};
use crate::common::function::FunctionCode;
use crate::error::details::ExceptionCode;
use crate::error::*;
use crate::server::handler::{RequestHandler, ServerHandlerMap};
use crate::server::request::Request;
use crate::server::response::ErrorResponse;
use crate::tcp::frame::{MBAPFormatter, MBAPParser};

pub(crate) struct SessionTask<T, U>
where
    T: RequestHandler,
    U: AsyncRead + AsyncWrite + Unpin,
{
    io: U,
    handlers: ServerHandlerMap<T>,
    shutdown: tokio::sync::mpsc::Receiver<()>,
    reader: FramedReader<MBAPParser>,
    writer: MBAPFormatter,
}

impl<T, U> SessionTask<T, U>
where
    T: RequestHandler,
    U: AsyncRead + AsyncWrite + Unpin,
{
    pub(crate) fn new(
        io: U,
        handlers: ServerHandlerMap<T>,
        shutdown: tokio::sync::mpsc::Receiver<()>,
    ) -> Self {
        Self {
            io,
            handlers,
            shutdown,
            reader: FramedReader::new(MBAPParser::new()),
            writer: MBAPFormatter::new(),
        }
    }

    async fn reply_with_error(
        &mut self,
        header: FrameHeader,
        err: ErrorResponse,
    ) -> Result<(), Error> {
        let bytes = self.writer.error(header, err)?;
        self.io.write_all(bytes).await?;
        Ok(())
    }

    pub(crate) async fn run(&mut self) -> Result<(), Error> {
        loop {
            self.run_one().await?;
        }
    }

    async fn run_one(&mut self) -> Result<(), Error> {
        tokio::select! {
            frame = self.reader.next_frame(&mut self.io) => {
               self.reply_to_request(frame?).await
            }
            _ = self.shutdown.recv() => {
               Err(crate::error::Error::Shutdown)
            }
        }
    }

    async fn reply_to_request(&mut self, frame: Frame) -> Result<(), Error> {
        let mut cursor = ReadCursor::new(frame.payload());

        // if no addresses match, then don't respond
        let handler = match self.handlers.get(frame.header.unit_id) {
            None => {
                log::warn!(
                    "received frame for unmapped unit id: {}",
                    frame.header.unit_id.value
                );
                return Ok(());
            }
            Some(handler) => handler,
        };

        let function = match cursor.read_u8() {
            Err(_) => {
                log::warn!("received an empty frame");
                return Ok(());
            }
            Ok(value) => match FunctionCode::get(value) {
                Some(x) => x,
                None => {
                    log::warn!("received unknown function code: {}", value);
                    return self
                        .reply_with_error(frame.header, ErrorResponse::unknown_function(value))
                        .await;
                }
            },
        };

        let request = match Request::parse(function, &mut cursor) {
            Ok(x) => x,
            Err(err) => {
                log::warn!("error parsing {:?} request: {}", function, err);
                let reply = self.writer.exception(
                    frame.header,
                    function,
                    ExceptionCode::IllegalDataValue,
                )?;
                self.io.write_all(reply).await?;
                return Ok(());
            }
        };

        // get the reply data (or exception reply)
        let reply_frame: &[u8] = {
            let mut lock = handler.lock().await;
            request.get_reply(frame.header, lock.as_mut(), &mut self.writer)?
        };

        // reply with the bytes
        self.io.write_all(reply_frame).await?;
        Ok(())
    }
}
