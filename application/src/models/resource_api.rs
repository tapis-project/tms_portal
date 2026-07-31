use serde::{Serialize};
use std::collections::HashSet;

#[derive(Debug, Hash, Serialize, Clone, PartialEq, Eq)]
pub struct Resource {
    pub id: String,
    pub name: String,
    pub description: String,
    pub provider_id: String,
    pub provider_account_id: String,
    pub provider_name: String,
}

pub type GetResourceResponse = HashSet<Resource>;
