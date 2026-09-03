use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Client {
    pub id: i32,
    pub client_id: String,
    pub secret: String,
    pub name: String,
    pub enabled: bool,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

