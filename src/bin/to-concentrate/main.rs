mod cli;
mod setup;

use clap::Parser;
use cli::Arguments;
use snafu::{prelude::*, Whatever};

#[snafu::report]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Whatever> {
    let args = Arguments::parse();
    let client = setup::bootstrap(&args).whatever_context("Could not bootstrap application")?;

    client
        .run(args.command.into())
        .await
        .whatever_context("Client failed to run")?;

    Ok(())
}
