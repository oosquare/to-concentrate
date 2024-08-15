use std::sync::Arc;

use snafu::prelude::*;

use crate::client::app::connector::{ConnectError, Connector};
use crate::domain::client::outbound::{BadResponseSnafu, UnavailableSnafu};
use crate::domain::client::outbound::{QueryPort, QueryResponse, RequestDaemonError};
use crate::protocol::{Connection, Protocol, Request, Response};

/// A [`QueryPort`] implementation
pub struct QueryService {
    connector: Arc<dyn Connector>,
}

impl QueryService {
    pub fn new(connector: Arc<dyn Connector>) -> Self {
        Self { connector }
    }
}

#[async_trait::async_trait]
impl QueryPort for QueryService {
    async fn query(&self) -> Result<QueryResponse, RequestDaemonError> {
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
        let request = Protocol::Request(Request::Query);

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
            Protocol::Response(Response::Query {
                current,
                stage,
                total,
                remaining,
                past,
            }) => Ok(QueryResponse {
                current,
                stage,
                total,
                remaining,
                past,
            }),
            _ => BadResponseSnafu.fail(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::time::Duration;

    use crate::client::app::connector::DuplexConnector;

    #[tokio::test]
    async fn query_service_run() {
        let (connector, mut server) = DuplexConnector::new(256);

        tokio::spawn(async move {
            let server = server.recv().await.unwrap();
            let mut connection = Connection::from(server);
            let response = Protocol::Response(Response::Query {
                current: "Running".to_owned(),
                stage: "Preparation".to_owned(),
                total: Duration::from_secs(20),
                remaining: Duration::from_secs(15),
                past: Duration::from_secs(5),
            });
            connection.send(response.into()).await.unwrap();
        });

        let service = QueryService::new(Arc::new(connector));
        let response = service.query().await.unwrap();
        assert_eq!(response.current, "Running");
        assert_eq!(response.stage, "Preparation");
        assert_eq!(response.total.as_secs(), 20);
        assert_eq!(response.remaining.as_secs(), 15);
        assert_eq!(response.past.as_secs(), 5);
    }

    #[tokio::test]
    async fn query_service_error_unavailable() {
        let (connector, server) = DuplexConnector::new(256);
        drop(server);

        let service = QueryService::new(Arc::new(connector));
        assert!(matches!(
            service.query().await,
            Err(RequestDaemonError::Unavailable { .. })
        ));
    }

    #[tokio::test]
    async fn query_service_error_unknown() {
        let (connector, mut server) = DuplexConnector::new(256);

        tokio::spawn(async move {
            let _ = server.recv().await.unwrap();
        });

        let service = QueryService::new(Arc::new(connector));
        assert!(matches!(
            service.query().await,
            Err(RequestDaemonError::Unknown { .. })
        ));
    }

    #[tokio::test]
    async fn query_service_error_bad_response() {
        let (connector, mut server) = DuplexConnector::new(256);

        tokio::spawn(async move {
            let server = server.recv().await.unwrap();
            let mut connection = Connection::from(server);
            let response = Protocol::Response(Response::Skip);
            connection.send(response.into()).await.unwrap();
        });

        let service = QueryService::new(Arc::new(connector));
        assert!(matches!(
            service.query().await,
            Err(RequestDaemonError::BadResponse)
        ));
    }
}
