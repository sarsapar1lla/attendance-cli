use bon::Builder;
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum RecordType {
    Office,
    WorkingFromHome,
    AnnualLeave,
    Sick,
    Other,
}

#[derive(Debug, Clone, Builder)]
pub struct Record {
    id: Uuid,
    created: DateTime<Utc>,
    record_type: RecordType,
    date: NaiveDate,
    description: Option<String>,
}
