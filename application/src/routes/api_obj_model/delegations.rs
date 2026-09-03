use serde::Serialize;
use crate::obj_model;

#[derive(Debug, Hash, Serialize, Clone, PartialEq, Eq)]
pub struct Delegation {
    pub id: i32,
    pub client_id: String,
    pub rp_account: String,
    pub expires_at: String,
    pub created: String,
    pub updated: String,
    pub tms_identity: String,
    pub rp_id: String,
}

impl From<obj_model::delegation::Delegation> for Delegation {
    fn from(value: obj_model::delegation::Delegation) -> Self {
        Delegation {
            id: value.id,
            client_id: value.client_id,
            rp_account: value.rp_account,
            expires_at: value.expires_at.to_rfc3339(),
            created: value.created.to_rfc3339(),
            updated: value.updated.to_rfc3339(),
            tms_identity: value.tms_identity,
            rp_id: value.rp_id,
        }
    }
}