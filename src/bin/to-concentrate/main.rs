mod cli;

use clap::Parser;
use cli::Arguments;

fn main() {
    Arguments::parse();
}
