use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use axum::{Json, Router, routing::get, extract::Query, Form};
use axum::handler::Handler;
use axum_extra::extract::cookie::{Cookie, Key};
use axum_extra::extract::PrivateCookieJar;
use reqwest::StatusCode;
use url::Url;
use crate::AppState;
use crate::db::idp_dao::{get_idps, get_idp_by_id};
use crate::models::authorize::{AuthCodeQueryParams, AuthorizationCodeResponse, AuthorizeByIdpRequest};
use crate::models::idp::IdpResponse;
use crate::models::oidc_state::{OAuthState};
use crate::models::responses::{Entity, HttpResponse, TmsApiErrorResult};
use crate::services::oauth_service::decode_jwt;

// impl FromRef<AppState> for Key {
//     fn from_ref(state: &AppState) -> Self {
//         state.key.clone()
//     }
// }

pub fn router() -> Router {
    let state = AppState {
        // Generate a secure key
        //
        // TODO:  You probably don't wanna generate a new one each time the app starts though
        key: Key::generate(),
    };

    return Router::new()
        .route("/oauth2/callback", get(get_callback_handler))
        .route("/oauth2/idp", get(get_idp_handler))
        .route("/oauth2/authorize", get(get_authorize_handler))
        .route("/oauth2/get", get(oauth_get_secret))
        .route("/oauth2/set", get(oauth_set_secret))
        .with_state(state);

    /*
    .route("/oauth2/decode", get(decode_jwt_working));
     */
//        .route("/oauth2/testing", get(get_testing_handler));
}
// pub async fn get_testing_handler(query_params:Query<HashMap<String, String>>) -> (StatusCode, Json<Value>) {
//     if(query_params.get("error").unwrap().eq("true")) {
//         let idps = get_idps().await.unwrap();
//         return (StatusCode::BAD_REQUEST, Json(serde_json::to_value(idps).unwrap()))
//     }
//     (StatusCode::OK, Json(json!({"hello":"world"})))
// }
pub async fn get_idp_handler() -> HttpResponse<HashSet<IdpResponse>> {
    let mut idp_result:HashSet<IdpResponse> = HashSet::new();
    match get_idps().await {
        Ok(idps) => {
            idps.iter().for_each(| idp | {
                idp_result.insert(idp.clone().into());
            });

            HttpResponse::from(
                (StatusCode::OK, Entity::Success(Json(idp_result)))
            )
        },

        Err(err) => {
            let error = TmsApiErrorResult {
                message: format!("{}", err)
            };

            HttpResponse::from((StatusCode::INTERNAL_SERVER_ERROR, Entity::from(error)))
        }
    }
}

pub async fn get_callback_handler(jar: PrivateCookieJar, query_params:Query<AuthCodeQueryParams>) -> HttpResponse<AuthorizationCodeResponse> {
    println!("code: {:?}", query_params.0.code);
    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", "cilogon:/client_id/3d38b53c9709489136c9b68c8f769c99"),
        ("client_secret", "_-9v-A023hVLquAlLjToWSQoF5XOwCKjG8i8QHtCN3K4c8fjF_ILSmf4ZekXafk0VC6q_T66WOntSUxgJLjN1Q"),
        ("code", &query_params.0.code),
    ];

    match jar.get("oauth2/idp_id") {
        Some(idp_id_cookie) => {
            println!("Request callback from provider: {:?}", idp_id_cookie.value());
        },
        None => {
            println!("No cookies found!!");
        }
    };

    let state_string = match &query_params.state {
        Some(state_string) => state_string,
        None => return HttpResponse::from((StatusCode::INTERNAL_SERVER_ERROR, Entity::from(TmsApiErrorResult {message: "Internal Server Error: unable to find state".to_string()})))
    };
    let state = match OAuthState::try_from(state_string) {
        Ok(state) => state,
        Err(err) => return HttpResponse::from( (StatusCode::INTERNAL_SERVER_ERROR, Entity::from(err)))
    };

    println!("state: {:?}", state);

    // let state = match OAuthState::from(&query_params.state) {
    //     Some(state) => state,
    //     None => {
    //         return HttpResponse::from((StatusCode::INTERNAL_SERVER_ERROR, "Unable to find state"));
    //     }
    // }

    // TODO:   get url from somewhere - not hard coded
    let client = reqwest::Client::new();
    let response = client.post("https://cilogon.org/oauth2/token")
        .form(&params)
        .send()
        .await;
    match response {
        Ok(response) => {
            let status_code = response.status();
            let response_text = response.text().await.unwrap().clone();
            let response_object:AuthorizationCodeResponse = serde_json::from_str(response_text.as_str()).unwrap();
            decode_jwt(&response_object.id_token).await;
            HttpResponse::from((status_code, Entity::Success(Json(response_object))))
        }

        Err(err) => {
            let error = TmsApiErrorResult {
                message: format!("{}", err)
            };
            HttpResponse::from((StatusCode::INTERNAL_SERVER_ERROR, Entity::from(error)))
        }
    }
    //
    // let statusCode = StatusCode::from(&response.unwrap().status());
    // let text = response.unwrap().text().await.unwrap().clone();
    // let response_object:AuthorizationCodeResponse = serde_json::from_str(text.as_str()).unwrap();
    // HttpResponse::from((statusCode, Entity::Success(Json(response_object))))
 }

/*
#[debug_handler]
pub async fn get_idp_handler() -> Result<Json<Value>, Error> {
    let idps = get_idps().await;
    match idps {
        Ok(idps) => Ok(Json(idps)),
        Err(err) => Err(Error::NotFound)
    }
}
*/
pub async fn get_authorize_handler(jar: PrivateCookieJar, form_data:Form<AuthorizeByIdpRequest>) -> (PrivateCookieJar, HttpResponse<()>) {
    let idp = get_idp_by_id(&form_data.idp_id).await;
    // match idp {
    //     Ok(idp) => {
    //         let cookie=Cookie::build(("idp_id", &idp.id.clone()));
    //     },
    //     Err(err) => {
    //     }
    // }
    match idp {
        Ok(idp) => {
            let oauth_state = OAuthState { idp_id: form_data.idp_id.clone(), exp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() + 300000 };
//            let jws = oauth_state.encode_jws();
//            println!("XXX: {:#?}", &jws);
//            let decoded_jws = decode_jws(&jws.unwrap());
//            println!("XXX: {:#?}", &decoded_jws);

            let encoded_state = match oauth_state.encode() {
                Ok(state_string) => {
                    println!("state: {:?}", state_string);
                    state_string
                },
                Err(error) => {
                    println!("error: {:?}", error);
                    return (jar, HttpResponse::from( (StatusCode::INTERNAL_SERVER_ERROR, Entity::from(error)) ))
                },
            };

            // TODO:  make a real nonce
            let location = Url::parse_with_params(&idp.identity_redirect_url, [
                ("response_type", "code"),
                ("client_id", &idp.client_id),
                ("redirect_uri","http://localhost:8080/oauth2/callback"),
                ("scope", &idp.scope),
                ("state", &encoded_state),
                ("nonce", "TODO: Add a real nonce")
            ]).unwrap();

            let updated_jar = jar.add(("oauth2/idp_id", idp.id));

            let mut headers = HashMap::new();
            headers.insert("location".to_string(), location.to_string());
            (updated_jar, HttpResponse::from((StatusCode::TEMPORARY_REDIRECT, headers)))
        }

        Err(err) => {
            let error = TmsApiErrorResult {
                message: format!("{}", err)
            };

            (jar, HttpResponse::from((StatusCode::BAD_REQUEST, Entity::from(error))))
        }
    }
}



/*
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    acr: String,
    affiliation: String,
    aud: String,
    auth_time: i64,
    eppn: String,
    exp: i64,
    family_name: String,
    given_name: String,
    iat: i64,
    idp: String,
    idp_name: String,
    iss: String,
    jti: String,
    name: String,
    nbf: i64,
    sub: String
}

pub async fn decode_jwt_working() -> Json<Value> {

//    let token = "eyJraWQiOiIyNDRCMjM1RjZCMjhFMzQxMDhEMTAxRUFDNzM2MkM0RSIsInR5cCI6IkpXVCIsImFsZyI6IlJTMjU2In0.eyJzdWIiOiJodHRwOi8vY2lsb2dvbi5vcmcvc2VydmVyRS91c2Vycy8yMjg5MTciLCJpZHBfbmFtZSI6IlVuaXZlcnNpdHkgb2YgVGV4YXMgYXQgQXVzdGluIiwiZXBwbiI6ImRsdjY0NEB1dGV4YXMuZWR1IiwiaXNzIjoiaHR0cHM6Ly9jaWxvZ29uLm9yZyIsImdpdmVuX25hbWUiOiJEYW4iLCJhY3IiOiJodHRwczovL2lkbS51dHN5c3RlbS5lZHUvYXV0aG5jb250ZXh0L3R3b2ZhY3RvcmJhc2ljIiwiYXVkIjoiY2lsb2dvbjovY2xpZW50X2lkLzNkMzhiNTNjOTcwOTQ4OTEzNmM5YjY4YzhmNzY5Yzk5IiwibmJmIjoxNzczMDk3ODYwLCJpZHAiOiJodHRwczovL2VudGVycHJpc2UubG9naW4udXRleGFzLmVkdS9pZHAvc2hpYmJvbGV0aCIsImFmZmlsaWF0aW9uIjoic3RhZmZAdXRleGFzLmVkdTttZW1iZXJAdXRleGFzLmVkdSIsImF1dGhfdGltZSI6MTc3Mjc0NTY0NCwibmFtZSI6IkRhbiBWZXJub24iLCJleHAiOjE3NzMwOTg3NjAsImZhbWlseV9uYW1lIjoiVmVybm9uIiwiaWF0IjoxNzczMDk3ODYwLCJqdGkiOiJodHRwczovL2NpbG9nb24ub3JnL29hdXRoMi9pZFRva2VuLzcxODdjNzg3ZmM0YzZlYTZmNDRhZmJhM2YxNzM4ZmIvMTc3Mjc0OTc0MDQzMyJ9.aSEGULt1fgVtUW8DB8mGeE2dmK9cTU2q4NSAMssqxfSYrw4rwrGop97gd1l3tDiSI4hCpTAmZHs_43GK5No7DqRznsZj-OqorgWIxcoi4rS29aizXedse6ltI-2ozc3IrLHLBnD43GNbwaAt-XCWShTBVmA7B_YdsmZzzdX2B8RLSrjuJeGetzPD7Jsv1ZsuwGj3m4TrvvTbDec1-FsCV1E8C-5scUsjdBxMUiZxvCgEsuqRpSjKoFms-My3eXJNooU5SgERcgxVELED1PuFUbtYHnWOM1dajaYrcEuCCrj6ZitfFQljCOcXZM9s74EA2trnMlq04QNSvItpp8G5UA";
    let token = "eyJraWQiOiIyNDRCMjM1RjZCMjhFMzQxMDhEMTAxRUFDNzM2MkM0RSIsInR5cCI6IkpXVCIsImFsZyI6IlJTMjU2In0.eyJzdWIiOiJodHRwOi8vY2lsb2dvbi5vcmcvc2VydmVyRS91c2Vycy8yMjg5MTciLCJpZHBfbmFtZSI6IlVuaXZlcnNpdHkgb2YgVGV4YXMgYXQgQXVzdGluIiwiZXBwbiI6ImRsdjY0NEB1dGV4YXMuZWR1IiwiaXNzIjoiaHR0cHM6Ly9jaWxvZ29uLm9yZyIsImdpdmVuX25hbWUiOiJEYW4iLCJhY3IiOiJodHRwczovL2lkbS51dHN5c3RlbS5lZHUvYXV0aG5jb250ZXh0L3R3b2ZhY3RvcmJhc2ljIiwiYXVkIjoiY2lsb2dvbjovY2xpZW50X2lkLzNkMzhiNTNjOTcwOTQ4OTEzNmM5YjY4YzhmNzY5Yzk5IiwibmJmIjoxNzczMTYxNDMzLCJpZHAiOiJodHRwczovL2VudGVycHJpc2UubG9naW4udXRleGFzLmVkdS9pZHAvc2hpYmJvbGV0aCIsImFmZmlsaWF0aW9uIjoic3RhZmZAdXRleGFzLmVkdTttZW1iZXJAdXRleGFzLmVkdSIsImF1dGhfdGltZSI6MTc3Mjc0NTY0NCwibmFtZSI6IkRhbiBWZXJub24iLCJleHAiOjE3NzMxNjIzMzMsImZhbWlseV9uYW1lIjoiVmVybm9uIiwiaWF0IjoxNzczMTYxNDMzLCJqdGkiOiJodHRwczovL2NpbG9nb24ub3JnL29hdXRoMi9pZFRva2VuLzcxODdjNzg3ZmM0YzZlYTZmNDRhZmJhM2YxNzM4ZmIvMTc3Mjc0OTc0MDQzMyJ9.IllyRSg_0c-zz2RCh4AQpc6fbcELgVksUd2_k-P9FBc_Qu2xscmk6o_FM8glspA9fcQny6quNZzie9yWwSlPEUVOQaDcpYN1FBXBu_IPmKTqj1nDnPD3IGpk7Z3b3X88VLPe-JTg8JrgJXMsG9Eei0k7y52yYvr7GgkqKZlcJ0woq7KNJxWVVj4z13JGq3L0cYJyxOglXYKrjbdcJV_8epkR5wzElbuoAbsQ_r8PLIl709XZiZbiKlANDAQu-6IYB7uI-QYT6Ep4Oic6JvXmD6Jmfh4s4kYH6b8lQwDD_VyFzjsajDYpJS-UXo3Eu6dc9KGYp1IAC83QmHhwJroqrA";
    let token_copy = token.clone();

    let key = DecodingKey::from_secret(&[]);
//    let mut validation = Validation::new(Algorithm::RS256);
//    validation.insecure_disable_signature_validation();
//    let tokenCopy = token;

    let jwt: TokenData<HashMap<String, Value>> = insecure_decode::<HashMap<String, Value>>(token.to_string()).unwrap();


    println!("token_header: {0}", serde_json::ser::to_string_pretty(&jwt.header).unwrap());
    println!("token_claims: {0}", serde_json::ser::to_string_pretty(&jwt.claims).unwrap());
    println!("token_claims_as_map idp_name: {0}", &jwt.claims.get("idp_name").unwrap());
    let value = Json(json!({"hello":"world"}));
    let issuer = jwt.claims.get("iss").unwrap();
    if(issuer.is_string() && issuer.as_str().unwrap() == "https://cilogon.org") {
        let client = reqwest::Client::new();
        let response = client.get("https://cilogon.org/oauth2/certs")
            .send()
            .await;

        println!("response: {:?}", response);
//        let jwks_result_string:HashMap<String, Value> = serde_json::de::from_str(response.unwrap().text()).unwrap();
        let json_response:HashMap<String, Value> = serde_json::de::from_str(&response.unwrap().text().await.unwrap()).unwrap();
        println!("json_response: {:?}", json_response);
        println!("keys: {:?}", json_response.keys());
//        let keys = json_response;
//        let keys = json_response.get("keys");
//        println!("The Keys: {:?}", keys.unwrap());

//        println!("Key: {}", keys.unwrap().as_array().unwrap().get(0).unwrap());
//        let theKey = serde_json::ser::to_string_pretty(keys.unwrap().as_array().unwrap().get(0).unwrap()).unwrap();
//        println!("");
//        println!("The Key: {:#?}", theKey);

//        let theRealKey = DecodingKey::from_secret(theKey.as_bytes());
//        let jwk_value = serde_json::to_value(theKey).unwrap();
//        let jwk_value:Value = serde_json::to_value(theKey).unwrap();
//        let jwk_value = json!(theKey.as_str());

        println!("");
//        println!("value: {:#?}", jwk_value.as_str());

        println!("");
//        println!("theKeySet: {:#?}", keys);

        let set:JwkSet = serde_json::de::from_str(serde_json::to_string(&json_response).unwrap().as_str()).unwrap();
        println!("");
        println!("theKeySet Set: {:#?}", set);

        let ajwk = set.keys.get(0).unwrap();


        println!("");
        // let keystring = "{
        //     \"kty\": \"RSA\",
        //     \"e\": \"AQAB\",
        //     \"kid\": \"4425a55c-35bc-44f5-9bd6-fdc7cdd36a31\",
        //     \"n\": \"hpJRWPJAv9yolz9ewiLJMk1udGZC0JSWX1OwqaTUpWgcvDricRmS0TVxWeDaHig7lprkb7YowOdBv20TWeeNNO1HxTO5nVDwg2jQ8IliR2Gscwi9pC6gektC9CXEBEEJYnl0rx9kazSvMTwXD_92jZt3k8ixbuNzX6ZcfotXe_vmFu7Jtgxr9XYzZXTMMjXLC4qt02oURVHOMwjR0ziuIIn0wZXvy7kGScnRZgYvyTBpIfvsRtVIEye5XPnk-DCBRo769qMFCDXD1Qhc9ePaxLseqhtO00XEqv8SZ6MnvjWpuvlxW6GPmVHK6h2tvD2Lk4GUO9qsj8wrU1KrsUgfsQ\"
        // }";
        // let jwk:Jwk = serde_json::de::from_str(keystring).unwrap();

        let theRealKey = DecodingKey::from_jwk(ajwk).unwrap();
//        let theRealKey = DecodingKey::from_jwk(&jwk).unwrap();

        println!("theRealKeyxxx: {:#?}", theRealKey);
        println!("theRealKey: {:#?}", theRealKey.kind());
        let mut validation = Validation::new(RS256);
        let mut audiences:HashSet<String> = HashSet::new();
        audiences.insert("cilogon:/client_id/3d38b53c9709489136c9b68c8f769c99".to_string());
        validation.aud = Some(audiences);
        println!("validation: {:?}", validation);
        println!("Before Validation");
        let tc = token_copy.to_string();
        let validatedJwt = decode::<HashMap<String, Value>>(tc, &theRealKey, &validation);
        println!("Validated Token: {0}", serde_json::ser::to_string_pretty(&validatedJwt.unwrap().claims).unwrap());

        println!("Valid!!");
        return value;
    }
    println!("invalid");

    value
}
 */
async fn oauth_set_secret(
    jar: PrivateCookieJar,
) -> (PrivateCookieJar, HttpResponse<HashSet<IdpResponse>>) {
    let updated_jar = jar.add(Cookie::new("secret", "secret-data"));

    let mut idp_result:HashSet<IdpResponse> = HashSet::new();
    match get_idps().await {
        Ok(idps) => {
            idps.iter().for_each(| idp | {
                idp_result.insert(idp.clone().into());
            });

            (updated_jar, HttpResponse::from( (StatusCode::OK, Entity::Success(Json(idp_result)))) )
        },

        Err(err) => {
            let error = TmsApiErrorResult {
                message: format!("{}", err)
            };

            (updated_jar, HttpResponse::from((StatusCode::INTERNAL_SERVER_ERROR, Entity::from(error))) )
        }
    }}

async fn oauth_get_secret(jar: PrivateCookieJar) -> String{
    if let Some(data) = jar.get("secret") {
        return "secret-data".to_owned();
    } else {
        return "Nothing found".to_owned();;
    }
}