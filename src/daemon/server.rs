use std::sync::Arc;

use snafu::prelude::*;
use tokio::io::{AsyncRead, AsyncWrite, Error as IoError};
use tokio::net::UnixListener;

use crate::domain::client::outbound::QueryResponse;
use crate::domain::daemon::Application;
use crate::protocol::connection::{ReceiveFrameError, SendFrameError};
use crate::protocol::{Connection, Protocol, Request, Response};

/// An dedicated server which listens on a UNIX socket and handles
/// requests from clients.
pub struct Server {
    listener: UnixListener,
    core: Arc<Application>,
}

impl Server {
    /// Creates a new [`Server`].
    pub fn new(listener: UnixListener, core: Application) -> Self {
        Self {
            listener,
            core: Arc::new(core),
        }
    }

    /// Accept connections from a [`UnixListener`] and handle requests.
    ///
    /// # Errors
    ///
    /// This function will return an error if the server fails to accept
    /// connections or any unexpected error occurs during handling requests.
    pub async fn serve(&self) -> Result<(), ServerError> {
        loop {
            let (stream, addr) = match self.listener.accept().await {
                Ok((stream, addr)) => {
                    tracing::info!(?addr, "Accepted connection from {addr:?}");
                    (stream, addr)
                }
                Err(err) => {
                    tracing::error!(?err, "{err}");
                    return Err(err).context(AcceptSnafu);
                }
            };

            let core = Arc::clone(&self.core);
            let connection = Connection::from(stream);
            tokio::spawn(async move {
                if let Err(err) = Self::handle(core, connection).await {
                    tracing::error!(
                        ?addr,
                        ?err,
                        "Could not handle requests from connection {addr:?}"
                    );
                }
            });
        }
    }

    /// Handle requests from an accepted connection.
    ///
    /// # Errors
    ///
    /// This function will return an error if handling connection fails.
    async fn handle<S>(
        core: Arc<Application>,
        mut connection: Connection<S>,
    ) -> Result<(), ServerError>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let request = match connection.receive().await {
            Ok(frame) => match Protocol::from(frame) {
                Protocol::Request(request) => request,
                protocol => return BadRequestSnafu { protocol }.fail(),
            },
            Err(err) => return Err(err).context(ReceiveSnafu),
        };

        match request {
            Request::Pause => {
                core.pause.pause().await;
                connection
                    .send(Protocol::Response(Response::Pause).into())
                    .await
                    .context(SendSnafu)
            }
            Request::Resume => {
                core.resume.resume().await;
                connection
                    .send(Protocol::Response(Response::Resume).into())
                    .await
                    .context(SendSnafu)
            }
            Request::Query => {
                let response = core.query.query().await;
                connection
                    .send(Protocol::Response(response.into()).into())
                    .await
                    .context(SendSnafu)
            }
            Request::Skip => {
                core.skip.skip().await;
                connection
                    .send(Protocol::Response(Response::Skip).into())
                    .await
                    .context(SendSnafu)
            }
        }
    }
}

impl From<QueryResponse> for Response {
    fn from(value: QueryResponse) -> Self {
        Response::Query {
            stage: value.stage,
            total: value.total,
            remaining: value.remaining,
            past: value.past,
        }
    }
}

/// An error type for server.
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum ServerError {
    #[snafu(display("Could not accept a connection"))]
    Accept { source: IoError },
    #[snafu(display("Could not receive a request"))]
    Receive { source: ReceiveFrameError },
    #[snafu(display("Could not handle {protocol:?}"))]
    BadRequest { protocol: Protocol },
    #[snafu(display("Could not send a response"))]
    Send { source: SendFrameError },
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::future;

    use tokio::io::DuplexStream;
    use tokio::time::Duration;

    use crate::domain::daemon::inbound::{
        MockPausePort, MockQueryPort, MockResumePort, MockSkipPort,
    };

    #[tokio::test]
    async fn server_handle() {
        let core = new_core();
        let (connection, mut client) = new_connection_with(Protocol::Request(Request::Query)).await;
        assert!(Server::handle(core, connection).await.is_ok());
        assert_eq!(
            client.receive().await.unwrap(),
            Protocol::Response(Response::Query {
                stage: "Preparation".to_owned(),
                total: Duration::from_secs(20),
                remaining: Duration::from_secs(15),
                past: Duration::from_secs(5),
            })
            .into(),
        );
    }

    #[tokio::test]
    async fn server_handle_error_bad_request() {
        let core = new_core();
        let (connection, _) = new_connection_with(Protocol::Response(Response::Pause)).await;
        assert!(matches!(
            Server::handle(core, connection).await,
            Err(ServerError::BadRequest {
                protocol: Protocol::Response(Response::Pause)
            }),
        ))
    }

    #[tokio::test]
    async fn server_handle_error_send() {
        let core = new_core();
        let (connection, client) = new_connection_with(Protocol::Request(Request::Pause)).await;
        drop(client);
        assert!(matches!(
            Server::handle(core, connection).await,
            Err(ServerError::Send { .. }),
        ))
    }

    fn new_core() -> Arc<Application> {
        let mut pause = MockPausePort::new();
        pause
            .expect_pause()
            .returning(|| Box::pin(future::ready(())));

        let mut resume = MockResumePort::new();
        resume
            .expect_resume()
            .returning(|| Box::pin(future::ready(())));

        let mut query = MockQueryPort::new();
        query.expect_query().returning(|| {
            Box::pin(future::ready(QueryResponse {
                stage: "Preparation".to_owned(),
                total: Duration::from_secs(20),
                remaining: Duration::from_secs(15),
                past: Duration::from_secs(5),
            }))
        });

        let mut skip = MockSkipPort::new();
        skip.expect_skip().returning(|| Box::pin(future::ready(())));

        let core = Application {
            pause: Arc::new(pause),
            resume: Arc::new(resume),
            query: Arc::new(query),
            skip: Arc::new(skip),
        };

        Arc::new(core)
    }

    async fn new_connection_with(
        data_recv: Protocol,
    ) -> (Connection<DuplexStream>, Connection<DuplexStream>) {
        let (server, client) = tokio::io::duplex(1024);
        let server = Connection::from(server);
        let mut client = Connection::from(client);
        client.send(data_recv.into()).await.unwrap();
        (server, client)
    }
}
