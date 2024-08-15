use std::sync::Arc;

use snafu::prelude::*;
use tracing::{field::Empty, Instrument, Span};

use crate::domain::client::outbound::QueryResponse;
use crate::domain::daemon::ApplicationCore;
use crate::protocol::connection::{ReceiveFrameError, SendFrameError};
use crate::protocol::{Connection, Protocol, Request, Response};
use crate::tracing_report;
use crate::utils::stream::Stream;

use super::listener::{ListenError, Listener};

/// An dedicated server which listens on a UNIX socket and handles
/// requests from clients.
pub struct Server {
    listener: Box<dyn Listener>,
    core: Arc<ApplicationCore>,
}

impl Server {
    /// Creates a new [`Server`].
    pub fn new(listener: Box<dyn Listener>, core: ApplicationCore) -> Self {
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
    #[tracing::instrument(skip(self))]
    pub async fn serve(&self) -> Result<(), ServerError> {
        loop {
            let stream = match self.listener.accept().await {
                Ok(stream) => {
                    tracing::info!("Accepted connection");
                    stream
                }
                Err(err) => {
                    tracing_report!(err);
                    return Err(err).context(ListenSnafu);
                }
            };

            let core = Arc::clone(&self.core);
            let connection = Connection::from(stream);

            let span = tracing::info_span!("handle", req = Empty).or_current();
            tokio::spawn(
                async move {
                    if let Err(err) = Self::handle(core, connection).await {
                        tracing_report!(err, format!("Could not handle requests"));
                    }
                }
                .instrument(span),
            );
        }
    }

    /// Handle requests from an accepted connection.
    ///
    /// # Errors
    ///
    /// This function will return an error if handling connection fails.
    async fn handle<S: Stream>(
        core: Arc<ApplicationCore>,
        mut connection: Connection<S>,
    ) -> Result<(), ServerError> {
        let request = match connection.receive().await {
            Ok(frame) => match Protocol::from(frame) {
                Protocol::Request(request) => request,
                protocol => return BadRequestSnafu { protocol }.fail(),
            },
            Err(err) => return Err(err).context(ReceiveSnafu),
        };

        Span::current().record("req", format!("{request:?}"));

        match request {
            Request::Pause => {
                tracing::info!("Received request");
                core.pause.pause().await;
                tracing::info!("Handled request");
                connection
                    .send(Protocol::Response(Response::Pause).into())
                    .await
                    .context(SendSnafu)
                    .inspect(|_| tracing::info!("Sent response"))
            }
            Request::Resume => {
                tracing::info!("Received request");
                core.resume.resume().await;
                tracing::info!("Handled request");
                connection
                    .send(Protocol::Response(Response::Resume).into())
                    .await
                    .context(SendSnafu)
                    .inspect(|_| tracing::info!("Sent response"))
            }
            Request::Query => {
                tracing::info!("Received request");
                let response = core.query.query().await;
                tracing::info!("Handled request");
                connection
                    .send(Protocol::Response(response.into()).into())
                    .await
                    .context(SendSnafu)
                    .inspect(|_| tracing::info!("Sent response"))
            }
            Request::Skip => {
                tracing::info!("Received request");
                core.skip.skip().await;
                tracing::info!("Handled request");
                connection
                    .send(Protocol::Response(Response::Skip).into())
                    .await
                    .context(SendSnafu)
                    .inspect(|_| tracing::info!("Sent response"))
            }
        }
    }
}

impl From<QueryResponse> for Response {
    fn from(value: QueryResponse) -> Self {
        Response::Query {
            current: value.current,
            stage: value.stage,
            total: value.total,
            remaining: value.remaining,
            past: value.past,
        }
    }
}

/// An error type for server.
#[derive(Debug, Snafu, Clone)]
#[non_exhaustive]
pub enum ServerError {
    #[snafu(display("Could not accept a connection"))]
    Listen { source: ListenError },
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
                current: "Running".to_owned(),
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

    fn new_core() -> Arc<ApplicationCore> {
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
                current: "Running".to_owned(),
                stage: "Preparation".to_owned(),
                total: Duration::from_secs(20),
                remaining: Duration::from_secs(15),
                past: Duration::from_secs(5),
            }))
        });

        let mut skip = MockSkipPort::new();
        skip.expect_skip().returning(|| Box::pin(future::ready(())));

        let core = ApplicationCore {
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
