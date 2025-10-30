use chrono::{Duration, DurationRound};
use comfy_table::{
    Attribute, Cell, CellAlignment, ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS,
    presets::UTF8_FULL_CONDENSED,
};

use crate::model::{Record, Summary};

pub trait RecordPrinter {
    fn print(&self, records: &[Record]);
}

pub struct TableRecordPrinter;

impl RecordPrinter for TableRecordPrinter {
    fn print(&self, records: &[Record]) {
        let rows: Vec<Vec<Cell>> = records.iter().map(TableRecordPrinter::row_from).collect();
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL_CONDENSED)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(TableRecordPrinter::header())
            .add_rows(rows);

        print!("{table}");
    }
}

impl TableRecordPrinter {
    fn header() -> Vec<Cell> {
        vec![
            header_cell("Id"),
            header_cell("Created"),
            header_cell("State"),
            header_cell("Record Type"),
            header_cell("Record Date"),
            header_cell("Description"),
        ]
    }

    fn row_from(record: &Record) -> Vec<Cell> {
        vec![
            Cell::new(record.id()),
            Cell::new(
                record
                    .created()
                    .duration_trunc(Duration::seconds(1))
                    .unwrap(),
            ),
            Cell::new(record.state()),
            Cell::new(record.record_type()),
            Cell::new(record.date()),
            record.description().map_or_else(
                || {
                    Cell::new("null")
                        .add_attributes(vec![Attribute::Dim, Attribute::Italic])
                        .set_alignment(CellAlignment::Center)
                },
                Cell::new,
            ),
        ]
    }
}

pub trait SummaryPrinter {
    fn print(&self, summaries: &[Summary]);
}

pub struct TableSummaryPrinter;

impl SummaryPrinter for TableSummaryPrinter {
    fn print(&self, summaries: &[Summary]) {
        let rows: Vec<Vec<Cell>> = summaries
            .iter()
            .map(TableSummaryPrinter::row_from)
            .collect();

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL_CONDENSED)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(TableSummaryPrinter::header())
            .add_rows(rows);

        print!("{table}");
    }
}

impl TableSummaryPrinter {
    fn row_from(summary: &Summary) -> Vec<Cell> {
        let month = format!(
            "{} {}",
            summary.month().month().name(),
            summary.month().year()
        );
        let attendance = (summary.attendance() * 100.0).round();
        let attendance = format!("{attendance}%");
        vec![
            Cell::new(month),
            Cell::new(summary.office_days()).set_alignment(CellAlignment::Right),
            Cell::new(summary.workdays()).set_alignment(CellAlignment::Right),
            Cell::new(attendance).set_alignment(CellAlignment::Right),
        ]
    }

    fn header() -> Vec<Cell> {
        vec![
            header_cell("Month"),
            header_cell("Days in Office"),
            header_cell("Working Days"),
            header_cell("Attendance"),
        ]
    }
}

fn header_cell(name: &str) -> Cell {
    Cell::new(name)
        .set_alignment(CellAlignment::Center)
        .add_attribute(Attribute::Bold)
}
