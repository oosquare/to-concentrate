use serde::{Deserialize, Serialize};
use tokio::time::Duration;

/// A [`Protocol`] represents the underlying data type used by
/// the protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Protocol {
    Request(Request),
    Response(Response),
}

/// A [`Request`] represents requests from a client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum Request {
    Pause,
    Resume,
    Query,
    Skip,
}

/// A [`Response`] represents a daemon's reply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum Response {
    Pause,
    Resume,
    Query {
        current: String,
        stage: String,
        total: Duration,
        remaining: Duration,
        past: Duration,
    },
    Skip,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_deserialize() {
        let text = serde_json::json!({
            "type": "Response",
            "method": "Query",
            "current": "Running",
            "stage": "Preparation",
            "total": {
                "secs": 20,
                "nanos": 0
            },
            "remaining": {
                "secs": 15,
                "nanos": 0
            },
            "past": {
                "secs": 5,
                "nanos": 0
            }
        });

        let data = Protocol::Response(Response::Query {
            current: "Running".to_owned(),
            stage: "Preparation".to_owned(),
            total: Duration::from_secs(20),
            remaining: Duration::from_secs(15),
            past: Duration::from_secs(5),
        });

        assert_eq!(serde_json::from_value::<Protocol>(text).unwrap(), data);
    }
}
