use comfy_table::{
    Cell, CellAlignment, Color, ContentArrangement, modifiers::UTF8_ROUND_CORNERS,
    presets::UTF8_FULL_CONDENSED,
};

use crate::{
    cli::summary::Arguments,
    error::{Error, Result},
    model::Summary,
    printer::cell::header,
};

pub trait Printer {
    fn print(&self, summaries: &[Summary]) -> Result<()>;
}

pub fn from_args(args: &Arguments) -> Box<dyn Printer> {
    if args.json() {
        Box::new(Json)
    } else {
        Box::new(Table)
    }
}

struct Json;

impl Printer for Json {
    fn print(&self, summaries: &[Summary]) -> Result<()> {
        let json = serde_json::to_string(&summaries).map_err(|e| Error::Io(e.to_string()))?;
        println!("{json}");
        Ok(())
    }
}

struct Table;

impl Printer for Table {
    fn print(&self, summaries: &[Summary]) -> Result<()> {
        let rows: Vec<Vec<Cell>> = summaries.iter().map(Table::row_from).collect();

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
            header("Month"),
            header("Target Days"),
            header("Days in Office"),
            header("Working Days"),
            header("Attendance"),
        ]
    }
}
