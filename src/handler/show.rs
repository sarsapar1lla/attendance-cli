use crate::{cli, repository::Repository};

pub fn show(args: &cli::ShowArgs, repository: &dyn Repository) {
    let records = repository.get();

    let truncated = match args.top() {
        None => records.as_slice(),
        Some(count) => &records.as_slice()[0..count],
    };

    println!("{:?}", truncated);
}
