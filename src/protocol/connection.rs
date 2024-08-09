use bytes::{Buf, BytesMut};
use snafu::prelude::*;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, Error};
use tokio::sync::Semaphore;

use crate::protocol::frame::{Frame, ParseFrameError, WriteFrameError};

/// A wrapper of a stream (typically a socket), which handles sending and
/// receiving frames through the stream.
pub struct Connection<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    stream: S,
    buffer: BytesMut,
    semaphore: Semaphore,
}

impl<S> Connection<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    /// Creates a new [`Connection<S>`].
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            buffer: BytesMut::with_capacity(1024),
            semaphore: Semaphore::new(1),
        }
    }

    /// Serialize a [`Frame`] to bytes and send it through the wrapped stream.
    ///
    /// # Errors
    ///
    /// This function will return an error if serialization fails or network
    /// IO fails.
    pub async fn send(&mut self, frame: Frame) -> Result<(), SendFrameError> {
        // Use a semaphore here to make this function can be called atomically.
        let Ok(_permit) = self.semaphore.acquire().await else {
            unreachable!("Semaphore should not be closed");
        };

        let mut buffer = BytesMut::with_capacity(256);
        frame.write(&mut buffer).context(WriteSnafuS)?;

        self.stream
            .write_all(&buffer)
            .await
            .context(NetworkSnafuS)?;

        Ok(())
    }

    /// Receive bytes from the wrapped stream and then deserialize the
    /// [`Frame`].
    /// # Errors
    ///
    /// This function will return an error if deserialization fails or network
    /// IO fails.
    pub async fn receive(&mut self) -> Result<Frame, ReceiveFrameError> {
        // Use a semaphore here to make this function can be called atomically.
        let Ok(_permit) = self.semaphore.acquire().await else {
            unreachable!("Semaphore should not be closed");
        };

        loop {
            let tmp_buffer = &self.buffer[..];

            match Frame::parse(tmp_buffer) {
                Ok((frame, offset)) => {
                    self.buffer.advance(offset);
                    return Ok(frame);
                }
                Err(ParseFrameError::Incomplete) => {}
                Err(err) => return Err(err).context(ParseSnafuR),
            }

            match self.stream.read_buf(&mut self.buffer).await {
                Ok(0) => return ClosedSnafuR.fail(),
                Err(err) => return Err(err).context(NetworkSnafuR),
                _ => {}
            }
        }
    }
}

#[derive(Debug, Snafu)]
#[snafu(context(suffix(SnafuS)))]
pub enum SendFrameError {
    #[snafu(display("Could not write frame to buffer"))]
    Write { source: WriteFrameError },
    #[snafu(display("Could not send bytes through inner stream"))]
    Network { source: Error },
}

#[derive(Debug, Snafu)]
#[snafu(context(suffix(SnafuR)))]
pub enum ReceiveFrameError {
    #[snafu(display("Could not parse frame from buffer"))]
    Parse { source: ParseFrameError },
    #[snafu(display("Connection is closed by the peer"))]
    Closed,
    #[snafu(display("Could not receive bytes through inner stream"))]
    Network { source: Error },
}
