use chrono::{DateTime, Utc};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct AllowedRedirect {
    pub uri: String,
    pub client_id: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

