use std::sync::LazyLock;

use chrono::{Duration, DurationRound};
use comfy_table::{
    Attribute, Cell, CellAlignment, Color, ContentArrangement, Table,
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL_CONDENSED,
};

use crate::model::{HalfDay, Key, Mode, Record, Summary};

static NULL_CELL: LazyLock<Cell> = LazyLock::new(|| {
    Cell::new("null")
        .add_attributes(vec![Attribute::Dim, Attribute::Italic])
        .set_alignment(CellAlignment::Center)
});

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
            header_cell("Date"),
            header_cell("Where?"),
            header_cell("Half Day?"),
            header_cell("Description"),
            header_cell("Logged"),
        ]
    }

    fn row_from(record: &Record) -> Vec<Cell> {
        let row_colour = match record.mode() {
            Mode::Create => Color::Green,
            Mode::Append => Color::DarkYellow,
            Mode::Delete => Color::Red,
        };
        let (date, half_day) = match *record.key() {
            Key::FullDay(date) => (
                Cell::new(date),
                NULL_CELL.clone().set_alignment(CellAlignment::Center),
            ),
            Key::HalfDay {
                date,
                half: HalfDay::Am,
            } => (
                Cell::new(date),
                Cell::new("Morning")
                    .fg(row_colour)
                    .set_alignment(CellAlignment::Left),
            ),
            Key::HalfDay {
                date,
                half: HalfDay::Pm,
            } => (
                Cell::new(date),
                Cell::new("Afternoon")
                    .fg(row_colour)
                    .set_alignment(CellAlignment::Left),
            ),
        };
        vec![
            date.fg(row_colour),
            Cell::new(record.record_type()).fg(row_colour),
            half_day,
            record
                .description()
                .map_or_else(|| NULL_CELL.clone(), Cell::new),
            Cell::new(
                record
                    .created()
                    .duration_trunc(Duration::seconds(1))
                    .unwrap(),
            )
            .fg(row_colour),
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
        let month = summary.month().format("%B %Y");
        let attendance = (summary.attendance() * 100.0).round();
        let attendance = format!("{attendance}%");
        let attendance_colour = match summary.attendance() {
            x if x < 0.30 => Color::Red,
            x if (0.30..0.50).contains(&x) => Color::DarkYellow,
            _ => Color::Green,
        };
        vec![
            Cell::new(month),
            Cell::new(summary.target_days()).set_alignment(CellAlignment::Right),
            Cell::new(summary.office_days()).set_alignment(CellAlignment::Right),
            Cell::new(summary.workdays()).set_alignment(CellAlignment::Right),
            Cell::new(attendance)
                .set_alignment(CellAlignment::Right)
                .fg(attendance_colour),
        ]
    }

    fn header() -> Vec<Cell> {
        vec![
            header_cell("Month"),
            header_cell("Target Days"),
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
