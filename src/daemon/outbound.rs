use notify_rust::Notification;
use snafu::prelude::*;

use crate::domain::daemon::outbound::{NotifyError, NotifyPort, NotifyRequest};

#[derive(Debug, Clone)]
pub struct NotifyService {
    app_name: String,
}

impl NotifyService {
    pub fn new(app_name: String) -> Self {
        Self { app_name }
    }
}

#[async_trait::async_trait]
impl NotifyPort for NotifyService {
    async fn notify_impl(&self, request: NotifyRequest) -> Result<(), NotifyError> {
        let mut notification = Notification::new();
        notification.appname(&self.app_name);
        notification.summary(&request.summary);

        if let Some(body) = request.body {
            notification.body(&body);
        }

        let _ = whatever!(
            notification.show_async().await,
            "Could not show notification",
        );

        Ok(())
    }
}
