use serde_json::Value;

#[derive(Debug)]
pub struct WhoAmIResponse {
    pub name: Option<Value>,
    pub username: Value,
    pub idp_display_name: Option<Value>,
    pub organization: Option<Value>,
}