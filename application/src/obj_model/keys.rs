use chrono::{DateTime, Utc};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Key {
    pub kid: String,
    pub jwt_public_key: String,
    pub jwt_private_key: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

