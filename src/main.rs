use clap::Parser;

use crate::{
    cli::{Cli, Command},
    repository::{InMemoryRepository, Repository},
};

mod cli;
mod handler;
mod model;
mod repository;

fn main() {
    // attendance log  .. default behaviour, office attendance on day of execution
    // attendance log --type wfh  .. log working from home day
    // attendance log --date '2025-10-20'  .. log attendance on specific day
    // attendance log --type other --date '2025-10-20' --append  .. append existing record for a certain day
    //
    // attendance summary
    // attendance summary --from 2025-09-01 --to 2025-11-01
    //
    // attendance show
    // attendance show --top 10

    let cli = Cli::parse();
    let repository = InMemoryRepository::new();

    match cli.command() {
        Command::Log(args) => handler::log(args, &repository),
        Command::Show(args) => handler::show(args, &repository),
    }

    let records = repository.get();
    for record in records {
        println!("{:?}", record);
    }
}
