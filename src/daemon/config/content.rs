use serde::Deserialize;

/// Overall configuration structure in memory.
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Configuration {
    pub duration: DurationSection,
    pub notification: NotificationSection,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct DurationSection {
    pub preparation: u64,
    pub concentration: u64,
    pub relaxation: u64,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct NotificationSection {
    pub preparation: MessageSection,
    pub concentration: MessageSection,
    pub relaxation: MessageSection,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct MessageSection {
    pub summary: String,
    pub body: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::daemon::config::reader::DEFAULT_CONTENT;

    #[test]
    fn deserialize_default_content() {
        let actual: Configuration = toml::from_str(DEFAULT_CONTENT).unwrap();

        let expected = Configuration {
            duration: DurationSection {
                preparation: 900,
                concentration: 2400,
                relaxation: 600,
            },
            notification: NotificationSection {
                preparation: MessageSection {
                    summary: "Preparation Stage End".to_owned(),
                    body: Some("It's time to start concentrating on learning.".to_owned()),
                },
                concentration: MessageSection {
                    summary: "Concentration Stage End".to_owned(),
                    body: Some("Well done! Remember to have a rest.".to_owned()),
                },
                relaxation: MessageSection {
                    summary: "Relaxation Stage End".to_owned(),
                    body: Some("Feel energetic now? Let's continue.".to_owned()),
                },
            },
        };

        assert_eq!(actual, expected);
    }
}
