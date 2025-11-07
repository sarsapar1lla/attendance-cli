use clap::Parser;

use crate::{
    cli::{Cli, Command},
    printer::{TableRecordPrinter, TableSummaryPrinter},
    repository::FileRepository,
};

mod cli;
mod error;
mod handler;
mod model;
mod printer;
mod repository;

fn main() {
    let cli = Cli::parse();
    let repository = FileRepository::new();
    let summary_handler = handler::summary::Handler::new(&repository, &TableSummaryPrinter);

    let result = match cli.command() {
        Command::Log(args) => handler::log(args, &repository),
        Command::Show(args) => handler::show(args, &repository, &TableRecordPrinter),
        Command::Summary(args) => summary_handler.summary(args),
    };

    match result {
        Ok(()) => {}
        Err(error) => println!("{error}"),
    }
}
