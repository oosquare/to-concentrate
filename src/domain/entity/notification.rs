use snafu::prelude::*;

/// Essential information in one XDG desktop notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationMessage {
    summary: String,
    body: Option<String>,
}

impl NotificationMessage {
    /// Try to create a [`NotificationMessage`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the summary is empty.
    pub fn try_new(
        summary: String,
        body: Option<String>,
    ) -> Result<Self, TryNewNotificationMessageError> {
        ensure!(!summary.is_empty(), EmptySummarySnafu);
        Ok(Self { summary, body })
    }

    /// Returns a reference to the summary of this [`NotificationMessage`].
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// Returns the body of this [`NotificationMessage`].
    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }
}

impl From<NotificationMessage> for (String, Option<String>) {
    fn from(val: NotificationMessage) -> Self {
        (val.summary, val.body)
    }
}

/// An error type of creating a [`NotificationMessage`].
#[derive(Debug, Clone, Snafu, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewNotificationMessageError {
    #[snafu(display("Summary of a notification must be non-empty."))]
    #[non_exhaustive]
    EmptySummary,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_message_try_new() {
        assert_eq!(
            NotificationMessage::try_new("summary".into(), Some("body".into())),
            Ok(NotificationMessage {
                summary: "summary".into(),
                body: Some("body".into())
            })
        );
        assert_eq!(
            NotificationMessage::try_new("".into(), Some("whatever".into())),
            Err(TryNewNotificationMessageError::EmptySummary)
        );
    }

    #[test]
    fn notification_message_operation() {
        let msg = NotificationMessage::try_new("summary".into(), Some("body".into())).unwrap();
        assert_eq!(msg.summary(), "summary");
        assert_eq!(msg.body(), Some("body"));
        let (inner_summary, inner_body) = msg.into();
        assert_eq!(inner_summary, "summary");
        assert_eq!(inner_body, Some("body".into()));
    }
}
