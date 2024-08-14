use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use snafu::prelude::*;
use tokio::io::DuplexStream;
use tokio::net::UnixStream;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::utils::stream::Stream;

/// Abstract connector which returns a stream with a given endpoint.
#[async_trait::async_trait]
pub trait Connector: Send + Sync + 'static {
    /// Return a stream connected to a peer.
    ///
    /// # Errors
    ///
    /// This function will return an error if the connection fails to establish.
    async fn connect(&self) -> Result<Box<dyn Stream>, ConnectError>;
}

/// An error for connecting procedure.
#[derive(Debug, Snafu, Clone)]
#[non_exhaustive]
pub enum ConnectError {
    #[snafu(display("Endpoint {endpoint} is unavailable"))]
    Unavailable { endpoint: String },
    #[snafu(display("Could not connect due to system error"))]
    System {
        #[snafu(source(from(IoError, Arc::new)))]
        source: Arc<IoError>,
    },
}

/// A [`Connector`] implementation which returns a [`UnixStream`].
#[derive(Debug, Clone)]
pub struct UnixConnector {
    path: PathBuf,
}

impl UnixConnector {
    /// Create a [`UnixConnector`] which will connect to `path`.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

#[async_trait::async_trait]
impl Connector for UnixConnector {
    async fn connect(&self) -> Result<Box<dyn Stream>, ConnectError> {
        match UnixStream::connect(self.path.as_path()).await {
            Ok(stream) => Ok(Box::new(stream)),
            Err(err) => match err.kind() {
                IoErrorKind::NotFound => UnavailableSnafu {
                    endpoint: self.path.to_string_lossy(),
                }
                .fail(),
                _ => Err(err).context(SystemSnafu),
            },
        }
    }
}

/// A [`Connector`] implementation which returns a [`DuplexStream`]. This is
/// typically used for testing purpose.
#[derive(Debug, Clone)]
pub struct DuplexConnector {
    peer: Sender<DuplexStream>,
    buffer_size: usize,
}

impl DuplexConnector {
    /// Create a [`DuplexConnector`] and return a channel receiver which
    /// receives the [`DuplexStream`].
    pub fn new(buffer_size: usize) -> (Self, Receiver<DuplexStream>) {
        let (sender, receiver) = mpsc::channel(1);
        let connector = Self {
            peer: sender,
            buffer_size,
        };
        (connector, receiver)
    }
}

#[async_trait::async_trait]
impl Connector for DuplexConnector {
    async fn connect(&self) -> Result<Box<dyn Stream>, ConnectError> {
        let (local, peer) = tokio::io::duplex(self.buffer_size);
        self.peer.send(peer).await.map_err(|_| {
            UnavailableSnafu {
                endpoint: "<memory>",
            }
            .build()
        })?;
        Ok(Box::new(local))
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;

    #[tokio::test]
    async fn unix_connector_error_unavailable() {
        let connector = UnixConnector::new("inexistent.socket");
        assert!(matches!(
            connector.connect().await,
            Err(ConnectError::Unavailable { .. })
        ))
    }

    #[tokio::test]
    async fn duplex_connector() {
        let (connector, mut peer) = DuplexConnector::new(256);
        let mut local = connector.connect().await.unwrap();
        let mut peer = peer.recv().await.unwrap();
        local.write_all(b"bytes").await.unwrap();
        drop(local);

        let mut buf = BytesMut::new();
        peer.read_buf(&mut buf).await.unwrap();
        assert_eq!(&buf[..], b"bytes");
    }

    #[tokio::test]
    async fn duplex_connector_error_unavailable() {
        let (connector, peer) = DuplexConnector::new(256);
        drop(peer);
        assert!(matches!(
            connector.connect().await,
            Err(ConnectError::Unavailable { .. })
        ));
    }
}
