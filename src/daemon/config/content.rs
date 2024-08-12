use serde::Deserialize;

/// Overall configuration structure in memory.
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct ConfigurationContent {
    pub duration: DurationContent,
    pub notification: NotificationContent,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct DurationContent {
    pub preparation: u64,
    pub concentration: u64,
    pub relaxation: u64,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct NotificationContent {
    pub preparation: MessageContent,
    pub concentration: MessageContent,
    pub relaxation: MessageContent,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct MessageContent {
    pub summary: String,
    pub body: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::daemon::config::reader::DEFAULT_CONTENT;

    #[test]
    fn deserialize_default_content() {
        let actual: ConfigurationContent = toml::from_str(DEFAULT_CONTENT).unwrap();

        let expected = ConfigurationContent {
            duration: DurationContent {
                preparation: 900,
                concentration: 2400,
                relaxation: 600,
            },
            notification: NotificationContent {
                preparation: MessageContent {
                    summary: "Preparation Stage End".to_owned(),
                    body: Some("It's time to start concentrating on learning.".to_owned()),
                },
                concentration: MessageContent {
                    summary: "Concentration Stage End".to_owned(),
                    body: Some("Well done! Remember to have a rest.".to_owned()),
                },
                relaxation: MessageContent {
                    summary: "Relaxation Stage End".to_owned(),
                    body: Some("Feel energetic now? Let's continue.".to_owned()),
                },
            },
        };

        assert_eq!(actual, expected);
    }
}
