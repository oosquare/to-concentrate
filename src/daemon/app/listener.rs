use std::fmt::Debug;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::path::Path;
use std::sync::Arc;

use snafu::prelude::*;
use tokio::io::DuplexStream;
use tokio::net::UnixListener as TokioUnixListener;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::utils::stream::Stream;

/// Abstract listener which listens on a given endpoint and accepts connections.
#[async_trait::async_trait]
pub trait Listener {
    /// Accept connections and return its corresponding stream.
    ///
    /// # Errors
    ///
    /// This function will return an error if the connection fails to establish.
    async fn accept(&self) -> Result<Box<dyn Stream>, ListenError>;
}

/// An error for listening procedure.
#[derive(Debug, Snafu, Clone)]
#[non_exhaustive]
pub enum ListenError {
    #[snafu(display("Could not bind to occupied endpoint {endpoint}"))]
    InUse { endpoint: String },
    #[snafu(display("Could not bind due to system error"))]
    BindSystem {
        #[snafu(source(from(IoError, Arc::new)))]
        source: Arc<IoError>,
    },
    #[snafu(display("Could not bind: {message}"))]
    BindUnknown { message: String },
    #[snafu(display("Could not accept connection due to system error"))]
    AcceptSystem {
        #[snafu(source(from(IoError, Arc::new)))]
        source: Arc<IoError>,
    },
}

/// A [`Listener`] implementation which returns [`UnixStream`]s.
#[derive(Debug)]
pub struct UnixListener {
    listener: TokioUnixListener,
}

impl UnixListener {
    /// Create a [`UnixListener`] with a given UNIX socket path.
    ///
    /// # Errors
    ///
    /// This function will return an error if it fails to bind to the socket.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, ListenError> {
        match TokioUnixListener::bind(path.as_ref()) {
            Ok(listener) => Ok(Self { listener }),
            Err(err) => match err.kind() {
                IoErrorKind::AddrInUse => InUseSnafu {
                    endpoint: path.as_ref().to_string_lossy(),
                }
                .fail(),
                _ => Err(err).context(BindSystemSnafu),
            },
        }
    }

    /// Return its internal listener.
    pub fn into_inner(self) -> TokioUnixListener {
        self.listener
    }
}

#[async_trait::async_trait]
impl Listener for UnixListener {
    async fn accept(&self) -> Result<Box<dyn Stream>, ListenError> {
        self.listener
            .accept()
            .await
            .map(|(stream, _)| -> Box<dyn Stream> { Box::new(stream) })
            .context(AcceptSystemSnafu)
    }
}

impl From<TokioUnixListener> for UnixListener {
    fn from(value: TokioUnixListener) -> Self {
        Self { listener: value }
    }
}

/// A [`Listener`] implementation which returns [`DuplexStream`]s. This is
/// typically used for testing purpose.
#[derive(Debug)]
pub struct DuplexListener {
    peer: Sender<DuplexStream>,
    buffer_size: usize,
}

impl DuplexListener {
    /// Create a [`DuplexListener`] and return a channel receiver which
    /// receives the [`DuplexStream`].
    pub fn new(buffer_size: usize) -> (Self, Receiver<DuplexStream>) {
        let (sender, receiver) = mpsc::channel(1);
        let listener = Self {
            peer: sender,
            buffer_size,
        };
        (listener, receiver)
    }
}

#[async_trait::async_trait]
impl Listener for DuplexListener {
    async fn accept(&self) -> Result<Box<dyn Stream>, ListenError> {
        let (local, peer) = tokio::io::duplex(self.buffer_size);
        self.peer.send(peer).await.map_err(|_| {
            BindUnknownSnafu {
                message: "Peer already closed",
            }
            .build()
        })?;
        Ok(Box::new(local))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_fs::{prelude::*, TempDir};
    use bytes::BytesMut;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn unix_listener_error_in_use() {
        let tmp = TempDir::new().expect("Test environment should support temporary directories");
        tmp.child("in-use.socket").touch().unwrap();
        let path = tmp.child("in-use.socket").to_path_buf();
        assert!(matches!(
            UnixListener::new(path),
            Err(ListenError::InUse { .. })
        ));
    }

    #[tokio::test]
    async fn duplex_listener() {
        let (connector, mut peer) = DuplexListener::new(256);
        let mut local = connector.accept().await.unwrap();
        let mut peer = peer.recv().await.unwrap();
        local.write_all(b"bytes").await.unwrap();
        drop(local);

        let mut buf = BytesMut::new();
        peer.read_buf(&mut buf).await.unwrap();
        assert_eq!(&buf[..], b"bytes");
    }

    #[tokio::test]
    async fn duplex_connector_error_bind_unknown() {
        let (connector, peer) = DuplexListener::new(256);
        drop(peer);
        assert!(matches!(
            connector.accept().await,
            Err(ListenError::BindUnknown { .. })
        ));
    }
}
