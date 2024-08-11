use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ConfigurationContent {
    duration: DurationContent,
    message: NotificationContent,
}

#[derive(Debug, Deserialize)]
pub struct DurationContent {
    preparation: u64,
    concentration: u64,
    relaxation: u64,
}

#[derive(Debug, Deserialize)]
pub struct NotificationContent {
    preparation: MessageContent,
    concentration: MessageContent,
    relaxation: MessageContent,
}

#[derive(Debug, Deserialize)]
pub struct MessageContent {
    summary: String,
    body: Option<String>,
}
