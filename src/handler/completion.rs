use std::io::Write;

use crate::{
    cli::{
        Cli,
        completion::{Arguments, Shell},
    },
    error::Result,
};
use clap::CommandFactory;

pub fn completion(args: &Arguments, buffer: &mut dyn Write) -> Result<()> {
    let mut cli = <Cli as CommandFactory>::command();
    let name = cli.get_name().to_string();
    let generator = generator(args.shell());

    clap_complete::generate(generator, &mut cli, name, buffer);

    Ok(())
}

fn generator(shell: Shell) -> impl clap_complete::Generator {
    match shell {
        Shell::Bash => clap_complete::Shell::Bash,
        Shell::Fish => clap_complete::Shell::Fish,
        Shell::Zsh => clap_complete::Shell::Zsh,
    }
}
