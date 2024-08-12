use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ConfigurationContent {
    pub duration: DurationContent,
    pub notification: NotificationContent,
}

#[derive(Debug, Deserialize)]
pub struct DurationContent {
    pub preparation: u64,
    pub concentration: u64,
    pub relaxation: u64,
}

#[derive(Debug, Deserialize)]
pub struct NotificationContent {
    pub preparation: MessageContent,
    pub concentration: MessageContent,
    pub relaxation: MessageContent,
}

#[derive(Debug, Deserialize)]
pub struct MessageContent {
    pub summary: String,
    pub body: Option<String>,
}
