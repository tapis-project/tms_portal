use std::collections::HashSet;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Idp {
    pub id: String,
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
    pub identity_redirect_url: String,
    pub oauth2_token_url: String,
    pub user_info_url: String,
    pub scope: String,
}
pub async fn get_idps() -> Result<HashSet<Idp>, String> {
    let cilogon_idp = Idp {
        id:"cilogon_idp".to_string(),
        name:"CILogon IDP".to_string(),
        client_id:"cilogon:/client_id/3d38b53c9709489136c9b68c8f769c99".to_string(),
        client_secret:"_-9v-A023hVLquAlLjToWSQoF5XOwCKjG8i8QHtCN3K4c8fjF_ILSmf4ZekXafk0VC6q_T66WOntSUxgJLjN1Q".to_string(),
        identity_redirect_url: "https://cilogon.org/authorize".to_string(),
        oauth2_token_url:"https://cilogon.org/oauth2/token".to_string(),
        user_info_url:"https://cilogon.org/oauth2/userinfo".to_string(),
        scope:"openid profile email org.cilogon.userinfo".to_string(),
    };

    let mut idps = HashSet::new();
    idps.insert(cilogon_idp);
    Ok(idps)
}

pub async fn get_idp_by_id(id: &String) -> Result<Idp, String> {
    let idps = get_idps().await?;
    let idp = idps
        .iter()
        .find(|idp| -> bool { idp.id.eq(id) })
        .ok_or_else(|| format!("Could not find id: {}", id))?;
    Ok(idp.clone())
}
