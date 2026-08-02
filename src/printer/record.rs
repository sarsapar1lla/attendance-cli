use std::sync::LazyLock;

use comfy_table::{
    Attribute, Cell, CellAlignment, Color, ContentArrangement, modifiers::UTF8_ROUND_CORNERS,
    presets::UTF8_FULL_CONDENSED,
};
use jiff::tz::TimeZone;

use crate::{
    error::Result,
    model::{HalfDay, Key, Mode, Record},
    printer::cell::{Row, header},
};

static NULL_CELL: LazyLock<Cell> = LazyLock::new(|| {
    Cell::new("null")
        .add_attributes(vec![Attribute::Dim, Attribute::Italic])
        .set_alignment(CellAlignment::Center)
});

pub trait Printer {
    fn print(&self, records: &[Record]) -> Result<()>;
}

pub struct Table;

impl Printer for Table {
    fn print(&self, records: &[Record]) -> Result<()> {
        let rows = records.iter().map(Table::row_from);
        let mut table = comfy_table::Table::new();
        table
            .load_preset(UTF8_FULL_CONDENSED)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(Table::header())
            .add_rows(rows);

        print!("{table}");
        Ok(())
    }
}

impl Table {
    fn row_from(record: &Record) -> Row {
        let row_colour = match record.mode() {
            Mode::Create => Color::Green,
            Mode::Append => Color::DarkYellow,
            Mode::Delete => Color::Red,
        };
        let date_format = "%Y-%m-%d (%a)";
        let (date, half_day) = match *record.key() {
            Key::FullDay(date) => (
                Cell::new(date.strftime(date_format)),
                NULL_CELL.clone().set_alignment(CellAlignment::Center),
            ),
            Key::HalfDay {
                date,
                half: HalfDay::Am,
            } => (
                Cell::new(date.strftime(date_format)),
                Cell::new("Morning")
                    .fg(row_colour)
                    .set_alignment(CellAlignment::Left),
            ),
            Key::HalfDay {
                date,
                half: HalfDay::Pm,
            } => (
                Cell::new(date.strftime(date_format)),
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
            Cell::new(record.mode()).fg(row_colour),
            Cell::new(
                record
                    .created()
                    .to_zoned(TimeZone::system())
                    .strftime("%Y-%m-%dT%H:%M:%S %Z"),
            )
            .fg(row_colour),
        ]
    }

    fn header() -> Row {
        vec![
            header("Date"),
            header("Where?"),
            header("Half Day?"),
            header("Description"),
            header("Mode"),
            header("Logged"),
        ]
    }
}
