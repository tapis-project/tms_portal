use chrono::{DateTime, Utc};

pub struct IssuedToken {
    pub access_token: String,
    pub expiration: DateTime<Utc>,
    pub revoked: bool,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl IssuedToken {
    pub fn is_expired(&self) -> bool {
        self.expiration < Utc::now()
    }

    pub fn is_revoked(&self) -> bool {
        self.revoked
    }
}