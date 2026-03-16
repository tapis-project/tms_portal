use std::collections::{HashMap, HashSet};
use jsonwebtoken::dangerous::insecure_decode;
use jsonwebtoken::{decode, Algorithm, DecodingKey, TokenData, Validation};
use jsonwebtoken::jwk::JwkSet;
use serde_json::Value;
use url::Url;
use crate::models::jwt::{Jwt};

pub async fn decode_jwt(token: &String) -> Result<Jwt, String> {

    let insecure_decode_result =
        insecure_decode::<HashMap<String, Value>>(token.to_string());
    let issuer = match insecure_decode_result {
        Ok(insecure_decoded) => {
            let issuer = insecure_decoded.claims.get("iss");

            match issuer {
                Some(value) => Some(value.clone()),
                _ => None
            }
        },

        Err(msg) => {
            println!("Error: {:?}", msg);
            None
        },
    };

    if issuer.is_some() {
        let issuer_string = issuer.unwrap();
        println!("Issuer: {:?}", &issuer_string);

        // TODO:  get this url from somewhere else.
        let pub_key_url = Url::parse("https://cilogon.org/oauth2/certs");
        if let Ok(pub_key_url) = pub_key_url {
            let pub_key = get_public_key(&pub_key_url).await;
            println!("PubKey: {:?}", &pub_key);

            // TODO: get CORRECT jwk - not just the first one and corect algo etc...
            let set:JwkSet = serde_json::de::from_str(pub_key.unwrap().as_str()).unwrap();

            let jwk = set.keys.get(0).unwrap();
            let decoding_key = DecodingKey::from_jwk(jwk);
            match decoding_key {
                Ok(key) => {
                    let mut validation = Validation::new(Algorithm::RS256);
                    let mut audiences:HashSet<String> = HashSet::new();
                    audiences.insert("cilogon:/client_id/3d38b53c9709489136c9b68c8f769c99".to_string());
                    validation.aud = Some(audiences);

                    let result: jsonwebtoken::errors::Result<TokenData<HashMap<String, Value>>>= decode(token, &key, &validation);
                    println!("Result: {:?}", &result);
                }
                _ => { println!("Error decoding key"); }
            }
        }
    }

//    Err("Unknown issuer!".to_string())
    Ok(Jwt{})
}

pub async fn get_public_key(pub_key_url: &Url) -> Option<String> {
    let client = reqwest::Client::new();
    let response = client.get(pub_key_url.as_str())
        .send()
        .await;
    match response {
        Ok(response) => {
            match response.text().await {
                Ok(text) => Some(text),
                _ => None
            }
        },
        Err(_) => { None },
    }
}
/*
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