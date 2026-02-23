use std::io::Write;

use clap::CommandFactory;

use crate::cli::Cli;

const BIN_NAME: &str = env!("CARGO_BIN_NAME");

pub fn spec(buffer: &mut dyn Write) {
    let mut cli = <Cli as CommandFactory>::command();
    clap_usage::generate(&mut cli, BIN_NAME, buffer);
}
