use std::sync::Arc;

use snafu::prelude::*;

use crate::client::app::connector::{ConnectError, Connector};
use crate::domain::client::outbound::{BadResponseSnafu, UnavailableSnafu};
use crate::domain::client::outbound::{RequestDaemonError, ResumePort};
use crate::protocol::{Connection, Protocol, Request, Response};

/// A [`ResumePort`] implementation
pub struct ResumeService {
    connector: Arc<dyn Connector>,
}

impl ResumeService {
    pub fn new(connector: Arc<dyn Connector>) -> Self {
        Self { connector }
    }
}

#[async_trait::async_trait]
impl ResumePort for ResumeService {
    async fn resume(&self) -> Result<(), RequestDaemonError> {
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
        let request = Protocol::Request(Request::Resume);

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
            Protocol::Response(Response::Resume) => Ok(()),
            _ => BadResponseSnafu.fail(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::client::app::connector::DuplexConnector;

    #[tokio::test]
    async fn resume_service_run() {
        let (connector, mut server) = DuplexConnector::new(256);

        tokio::spawn(async move {
            let server = server.recv().await.unwrap();
            let mut connection = Connection::from(server);
            let response = Protocol::Response(Response::Resume);
            connection.send(response.into()).await.unwrap();
        });

        let service = ResumeService::new(Arc::new(connector));
        assert!(service.resume().await.is_ok());
    }

    #[tokio::test]
    async fn resume_service_error_unavailable() {
        let (connector, server) = DuplexConnector::new(256);
        drop(server);

        let service = ResumeService::new(Arc::new(connector));
        assert!(matches!(
            service.resume().await,
            Err(RequestDaemonError::Unavailable { .. })
        ));
    }

    #[tokio::test]
    async fn resume_service_error_unknown() {
        let (connector, mut server) = DuplexConnector::new(256);

        tokio::spawn(async move {
            let _ = server.recv().await.unwrap();
        });

        let service = ResumeService::new(Arc::new(connector));
        assert!(matches!(
            service.resume().await,
            Err(RequestDaemonError::Unknown { .. })
        ));
    }

    #[tokio::test]
    async fn resume_service_error_bad_response() {
        let (connector, mut server) = DuplexConnector::new(256);

        tokio::spawn(async move {
            let server = server.recv().await.unwrap();
            let mut connection = Connection::from(server);
            let response = Protocol::Response(Response::Skip);
            connection.send(response.into()).await.unwrap();
        });

        let service = ResumeService::new(Arc::new(connector));
        assert!(matches!(
            service.resume().await,
            Err(RequestDaemonError::BadResponse)
        ));
    }
}
