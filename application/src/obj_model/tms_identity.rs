use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct TmsIdentity {
    pub seq_id: i32,
    pub tms_identity: String,
    pub created: DateTime<Utc>,
}
