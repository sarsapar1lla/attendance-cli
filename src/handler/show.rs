use itertools::Itertools;

use crate::{cli, error::Result, model::Record, printer::RecordPrinter, repository::Repository};

pub fn show(
    args: &cli::ShowArgs,
    repository: &dyn Repository,
    printer: &dyn RecordPrinter,
) -> Result<()> {
    let records = repository.get()?;

    let sorted: Vec<Record> = records
        .into_iter()
        .sorted_by(|a, b| a.created().cmp(b.created()).reverse())
        .collect();

    let truncated = match args.top() {
        None => sorted.as_slice(),
        Some(count) => &sorted.as_slice()[0..count],
    };

    printer.print(truncated);
    Ok(())
}
