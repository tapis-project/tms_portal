use chrono::{DateTime, Utc};

pub struct Delegation {
    pub id: i32,
    pub client_id: String,
    pub rp_account: String,
    pub expires_at: DateTime<Utc>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub tms_identity: String,
    pub rp_id: String,
}