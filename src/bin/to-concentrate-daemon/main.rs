mod cli;
mod setup;

use clap::Parser;
use snafu::{prelude::*, Whatever};

use crate::cli::Arguments;

#[snafu::report]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Whatever> {
    let arg = Arguments::parse();

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(arg.verbosity)
        // .with_line_number(true)
        .pretty()
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .whatever_context("Could not setup logger")?;

    let server = setup::bootstrap(arg).await?;

    server
        .serve()
        .await
        .whatever_context("Server failed to serve with fatal")?;

    Ok(())
}
