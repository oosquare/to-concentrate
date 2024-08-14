use std::sync::Arc;

use snafu::prelude::*;

use crate::client::app::connector::{ConnectError, Connector};
use crate::domain::client::outbound::{BadResponseSnafu, UnavailableSnafu};
use crate::domain::client::outbound::{RequestDaemonError, SkipPort};
use crate::protocol::{Connection, Protocol, Request, Response};

/// A [`SkipPort`] implementation
pub struct SkipService {
    connector: Arc<dyn Connector>,
}

impl SkipService {
    pub fn new(connector: Arc<dyn Connector>) -> Self {
        Self { connector }
    }
}

#[async_trait::async_trait]
impl SkipPort for SkipService {
    async fn skip(&self) -> Result<(), RequestDaemonError> {
        let stream = match self.connector.connect().await {
            Ok(stream) => stream,
            Err(err) => match err {
                ConnectError::Unavailable { endpoint } => {
                    return UnavailableSnafu { endpoint }.fail()
                }
                err => return Err(err).whatever_context("Could not connect"),
            },
        };

        let mut connection = Connection::from(stream);
        let request = Protocol::Request(Request::Skip);

        connection
            .send(request.into())
            .await
            .whatever_context("Could not send request")?;

        let response: Protocol = connection
            .receive()
            .await
            .whatever_context("Could not receive response")?
            .into();

        match response {
            Protocol::Response(Response::Skip) => Ok(()),
            _ => BadResponseSnafu.fail(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::client::app::connector::DuplexConnector;

    #[tokio::test]
    async fn skip_service_run() {
        let (connector, mut server) = DuplexConnector::new(256);

        tokio::spawn(async move {
            let server = server.recv().await.unwrap();
            let mut connection = Connection::from(server);
            let response = Protocol::Response(Response::Skip);
            connection.send(response.into()).await.unwrap();
        });

        let service = SkipService::new(Arc::new(connector));
        assert!(service.skip().await.is_ok());
    }

    #[tokio::test]
    async fn skip_service_error_unavailable() {
        let (connector, server) = DuplexConnector::new(256);
        drop(server);

        let service = SkipService::new(Arc::new(connector));
        assert!(matches!(
            service.skip().await,
            Err(RequestDaemonError::Unavailable { .. })
        ));
    }

    #[tokio::test]
    async fn skip_service_error_unknown() {
        let (connector, mut server) = DuplexConnector::new(256);

        tokio::spawn(async move {
            let _ = server.recv().await.unwrap();
        });

        let service = SkipService::new(Arc::new(connector));
        assert!(matches!(
            service.skip().await,
            Err(RequestDaemonError::Unknown { .. })
        ));
    }

    #[tokio::test]
    async fn skip_service_error_bad_response() {
        let (connector, mut server) = DuplexConnector::new(256);

        tokio::spawn(async move {
            let server = server.recv().await.unwrap();
            let mut connection = Connection::from(server);
            let response = Protocol::Response(Response::Pause);
            connection.send(response.into()).await.unwrap();
        });

        let service = SkipService::new(Arc::new(connector));
        assert!(matches!(
            service.skip().await,
            Err(RequestDaemonError::BadResponse)
        ));
    }
}
