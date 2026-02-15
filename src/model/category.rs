#[derive(Debug, PartialEq)]
pub enum WeekendDay {
    Saturday,
    Sunday,
}

#[derive(Debug, PartialEq)]
pub enum Category {
    Workday,
    Weekend(WeekendDay),
    BankHoliday,
}
