use std::io::Write;

use crate::cli::{
    Cli,
    completion::{Arguments, Shell},
};
use clap::CommandFactory;
use clap_complete::aot;

pub fn completion(args: &Arguments, buffer: &mut dyn Write) {
    let mut cli = <Cli as CommandFactory>::command();
    let name = cli.get_name().to_string();
    let generator = generator(&args.shell());

    aot::generate(generator, &mut cli, name, buffer);
}

fn generator(shell: &Shell) -> aot::Shell {
    match shell {
        Shell::Bash => aot::Shell::Bash,
        Shell::Fish => aot::Shell::Fish,
        Shell::Zsh => aot::Shell::Zsh,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_completions_to_buffer() {
        let args = Arguments::new(Shell::Fish);
        let mut buffer = Vec::new();

        completion(&args, &mut buffer);

        assert!(buffer.len() > 0)
    }
}
