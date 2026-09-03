use chrono::{DateTime, Utc};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ResourceProviderLogin {
    pub id:i32,
    pub tms_identity:String,
    pub enabled:bool,
    pub created:DateTime<Utc>,
    pub updated:DateTime<Utc>,
    pub rp_id:String,
    pub rp_account:String,
    pub last_login:DateTime<Utc>,
}
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ResourceAccountLink {
    pub id:i32,
    pub tms_identity:String,
    pub rp_account:String,
    pub rp_id:String,
    pub rp_name:String,
    pub last_login:DateTime<Utc>,
    pub enabled:bool,
    pub created:DateTime<Utc>,
    pub updated:DateTime<Utc>,
}
