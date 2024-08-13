use std::sync::Arc;

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

impl<S> From<S> for Connection<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn from(value: S) -> Self {
        Self {
            stream: value,
            buffer: BytesMut::with_capacity(1024),
            semaphore: Semaphore::new(1),
        }
    }
}

#[derive(Debug, Snafu, Clone)]
#[snafu(context(suffix(SnafuS)))]
pub enum SendFrameError {
    #[snafu(display("Could not write frame to buffer"))]
    Write { source: WriteFrameError },
    #[snafu(display("Could not send bytes through inner stream"))]
    Network {
        #[snafu(source(from(Error, Arc::new)))]
        source: Arc<Error>,
    },
}

#[derive(Debug, Snafu, Clone)]
#[snafu(context(suffix(SnafuR)))]
pub enum ReceiveFrameError {
    #[snafu(display("Could not parse frame from buffer"))]
    Parse { source: ParseFrameError },
    #[snafu(display("Connection is closed by the peer"))]
    Closed,
    #[snafu(display("Could not receive bytes through inner stream"))]
    Network {
        #[snafu(source(from(Error, Arc::new)))]
        source: Arc<Error>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    use bytes::BufMut;
    use tokio::time::Duration;

    use crate::protocol::{Protocol, Response};

    #[tokio::test]
    async fn connection_send() {
        let (frame, expected) = new_frame();
        let (sender, mut receiver) = tokio::io::duplex(1024);
        let mut connection = Connection::from(sender);

        let handle = tokio::spawn(async move {
            let mut buffer = Vec::with_capacity(256);
            receiver.read_to_end(&mut buffer).await.unwrap();
            assert_eq!(buffer, expected);
        });

        assert!(connection.send(frame).await.is_ok());
        drop(connection);
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn connection_receive() {
        let (expected, buffer) = new_frame();
        let (mut sender, receiver) = tokio::io::duplex(1024);
        let mut connection = Connection::from(receiver);

        tokio::spawn(async move {
            for _ in 0..64 {
                sender.write_all(&buffer[..]).await.unwrap();
            }
        });

        for _ in 0..64 {
            assert_eq!(connection.receive().await.unwrap(), expected);
        }
    }

    #[tokio::test]
    async fn connection_receive_error_parse() {
        let (mut sender, receiver) = tokio::io::duplex(1024);
        let mut connection = Connection::from(receiver);

        tokio::spawn(async move {
            let mut raw = BytesMut::new();
            raw.put_u8(b'+');
            raw.put_u64(8);
            raw.put_slice(b"whatever");
            sender.write_all(&raw[..]).await.unwrap();
        });

        assert!(matches!(
            connection.receive().await,
            Err(ReceiveFrameError::Parse {
                source: ParseFrameError::Deserialization { .. }
            })
        ));
    }

    #[tokio::test]
    async fn connection_receive_error_closed() {
        let (expected, buffer) = new_frame();
        let (mut sender, receiver) = tokio::io::duplex(1024);
        let mut connection = Connection::from(receiver);

        tokio::spawn(async move {
            sender.write_all(&buffer[..]).await.unwrap();
        });

        assert_eq!(connection.receive().await.unwrap(), expected);
        assert!(matches!(
            connection.receive().await,
            Err(ReceiveFrameError::Closed)
        ));
    }

    fn new_frame() -> (Frame, BytesMut) {
        let frame: Frame = Protocol::Response(Response::Query {
            stage: "Preparation".to_owned(),
            total: Duration::from_secs(20),
            remaining: Duration::from_secs(15),
            past: Duration::from_secs(5),
        })
        .into();

        let mut buffer = BytesMut::with_capacity(256);
        frame.write(&mut buffer).unwrap();
        (frame, buffer)
    }
}
