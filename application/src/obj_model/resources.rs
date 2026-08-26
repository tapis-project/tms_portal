use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ResourceProviderLogin {
    pub id:i32,
    pub tms_identity:String,
    pub provider_account:String,
    pub provider_uuid:Option<Uuid>,
    pub last_login:DateTime<Utc>,
    pub enabled:bool,
    pub created:DateTime<Utc>,
    pub updated:DateTime<Utc>,
}
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ResourceAccountLink {
    pub id:i32,
    pub tms_identity:String,
    pub resource_provider_account:String,
    pub resource_provider_uuid:Uuid,
    pub resource_provider_id:String,
    pub resource_provider_name:String,
    pub last_login:DateTime<Utc>,
    pub enabled:bool,
    pub created:DateTime<Utc>,
    pub updated:DateTime<Utc>,
}
