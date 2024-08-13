#[macro_export]
macro_rules! tracing_report {
    ($error:expr) => {
        tracing::error!(err = %snafu::Report::from_error(&$error));
    };
    ($error:expr, $message:expr) => {
        let whatever_error = <snafu::Whatever as snafu::FromString>::with_source($error.clone().into(), $message.to_string());
        tracing::error!(err = %snafu::Report::from_error(whatever_error));
    };
}
