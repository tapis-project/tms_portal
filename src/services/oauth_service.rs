use crate::db::config_dao::state_public_key;
use crate::models::oidc_state::OAuthState;
use crate::models::tms_internal::TmsResult;
use crate::utils::jwt_utils::JwtDecoderBuilder;
use jsonwebtoken::DecodingKey;
use url::Url;

pub struct Jwt;

// pub async fn decode_jwt(token: &String) -> TmsResult<Jwt> {
//     let insecure_decode_result = insecure_decode::<HashMap<String, Value>>(token.to_string());
//     let issuer = match insecure_decode_result {
//         Ok(insecure_decoded) => {
//             let issuer = insecure_decoded.claims.get("iss");
//
//             match issuer {
//                 Some(value) => Some(value.clone()),
//                 _ => None,
//             }
//         }
//
//         Err(msg) => {
//             println!("Error: {:?}", msg);
//             None
//         }
//     };
//
//     if issuer.is_some() {
//         let issuer_string = issuer.unwrap();
//         println!("Issuer: {:?}", &issuer_string);
//
//         // TODO:  get this url from somewhere else.
//         let pub_key_url = Url::parse("https://cilogon.org/oauth2/certs");
//         if let Ok(pub_key_url) = pub_key_url {
//             let pub_key = get_public_key(&pub_key_url).await;
//             println!("PubKey: {:?}", &pub_key);
//
//             // TODO: get CORRECT jwk - not just the first one and corect algo etc...
//             let set: JwkSet = serde_json::de::from_str(pub_key.unwrap().as_str()).unwrap();
//
//             let jwk = set.keys.get(0).unwrap();
//             let decoding_key = DecodingKey::from_jwk(jwk);
//             match decoding_key {
//                 Ok(key) => {
//                     let mut validation = Validation::new(Algorithm::RS256);
//                     let mut audiences: HashSet<String> = HashSet::new();
//                     audiences
//                         .insert("cilogon:/client_id/3d38b53c9709489136c9b68c8f769c99".to_string());
//                     validation.aud = Some(audiences);
//
//                     let result: jsonwebtoken::errors::Result<TokenData<HashMap<String, Value>>> =
//                         decode(token, &key, &validation);
//                     println!("Result: {:?}", &result);
//                 }
//                 _ => {
//                     println!("Error decoding key");
//                 }
//             }
//         }
//     }
//
//     //    Err("Unknown issuer!".to_string())
//     Ok(Jwt {})
// }

pub async fn get_public_key(pub_key_url: &Url) -> Option<String> {
    let client = reqwest::Client::new();
    let response = client.get(pub_key_url.as_str()).send().await;
    match response {
        Ok(response) => match response.text().await {
            Ok(text) => Some(text),
            _ => None,
        },
        Err(_) => None,
    }
}

// pub async fn decode_jwt(token_string: &String) {
//     match JwtDecoderBuilder::builder()
//         .jwks_url("https://cilogon.org/oauth2/certs")
//         .decode(&token_string)
//         .await
//     {
//         Ok(decoded) => TmsHttpResponse::from((StatusCode::OK, Entity::Success(Json(decoded)))),
//         Err(error) => TmsHttpResponse::from((
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Entity::from(TmsApiErrorResult {
//                 message: error.to_string(),
//             }),
//         )),
//     }
// }

pub async fn decode_state(state_string: &String) -> TmsResult<OAuthState> {
    let decoding_key = match DecodingKey::from_rsa_pem(&state_public_key().as_bytes()) {
        Ok(decoding_key) => decoding_key,
        Err(error) => return Err(error.to_string()),
    };

    JwtDecoderBuilder::builder()
        .public_key(decoding_key)
        .decode::<OAuthState>(&state_string)
        .await
}
