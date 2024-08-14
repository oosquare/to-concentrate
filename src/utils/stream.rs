use tokio::io::DuplexStream;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::UnixStream;

/// Abstract form of types that are capable of async IO.
pub trait Stream: AsyncRead + AsyncWrite + Unpin {}

impl Stream for UnixStream {}

impl Stream for DuplexStream {}
