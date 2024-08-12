mod cli;
mod setup;

use clap::Parser;
use snafu::{prelude::*, Whatever};

use crate::cli::Arguments;

#[snafu::report]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Whatever> {
    let arg = Arguments::parse();
    let server = setup::bootstrap(arg).await?;

    server
        .serve()
        .await
        .whatever_context("Server failed to serve with fatal")?;

    Ok(())
}
