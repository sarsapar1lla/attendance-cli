use comfy_table::{Attribute, Cell, CellAlignment};

pub fn header(name: &str) -> Cell {
    Cell::new(name)
        .set_alignment(CellAlignment::Center)
        .add_attribute(Attribute::Bold)
}
