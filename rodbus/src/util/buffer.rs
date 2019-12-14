use tokio::io::{AsyncRead, AsyncReadExt};

use crate::error::*;

pub struct ReadBuffer {
    buffer: Vec<u8>,
    begin: usize,
    end: usize
}

impl ReadBuffer {

    pub fn new(capacity: usize) -> Self {
        ReadBuffer {
            buffer: vec![0; capacity],
            begin: 0,
            end: 0
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.begin
    }

    pub fn is_empty(&self) -> bool {
        self.begin == self.end
    }

    pub fn read(&mut self, count: usize)-> std::result::Result<&[u8], bugs::Error> {
        if self.len() < count {
            return Err(bugs::ErrorKind::InsufficientBytesForRead(count, self.len()).into());
        }

        let ret = &self.buffer[self.begin .. (self.begin + count)];
        self.begin += count;
        Ok(ret)
    }
    pub fn read_u8(&mut self) -> std::result::Result<u8, bugs::Error> {
        if self.is_empty() {
            return Err(bugs::ErrorKind::InsufficientBytesForRead(1, 0).into());
        }

        let ret = self.buffer[self.begin];
        self.begin += 1;
        Ok(ret)
    }
    pub fn read_u16_be(&mut self) -> std::result::Result<u16, bugs::Error> {
        let b1 = self.read_u8()? as u16;
        let b2 = self.read_u8()? as u16;
        Ok((b1 << 8) | b2)
    }

    pub async fn read_some<T : AsyncRead + Unpin>(&mut self, io: &mut T) -> std::result::Result<usize, std::io::Error> {

        // before we read any data, check to see if the buffer is empty and adjust the indices
        // this allows use to make the biggest read possible, and avoids subsequent buffer shifting later
        if self.is_empty() {
            self.begin = 0;
            self.end = 0;
        }

        // if we've reached capacity, but still need more data we have to shift
        if self.end == self.buffer.capacity() {
            let length = self.len();
            self.buffer.copy_within(self.begin..self.end, 0);
            self.begin = 0;
            self.end = length;
        }

        let count = io.read(&mut self.buffer[self.end..]).await?;
        if count == 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
        }
        self.end += count;
        Ok(count)
    }
}