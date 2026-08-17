use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Client {
    pub id: String,
    pub secret: String,
    pub name: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

