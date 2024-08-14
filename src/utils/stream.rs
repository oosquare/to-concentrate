use tokio::io::DuplexStream;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::UnixStream;

/// Abstract form of types that are capable of async IO.
pub trait Stream: AsyncRead + AsyncWrite + Unpin + Send {}

impl Stream for Box<dyn Stream> {}

impl Stream for UnixStream {}

impl Stream for DuplexStream {}
