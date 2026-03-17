use crate::db::config_dao::jwt_private_key;
use crate::db::idp_dao::Idp;
use crate::models::tms_internal::TmsResult;
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{
    decode, decode_header, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData,
    Validation,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use url::Url;

pub struct JwtDecoderBuilder {
    jwks_url: Option<String>,
    public_key: Option<DecodingKey>,
    audience: Option<HashSet<String>>,
}

impl JwtDecoderBuilder {
    pub fn builder() -> JwtDecoderBuilder {
        JwtDecoderBuilder {
            jwks_url: None,
            public_key: None,
            audience: None,
        }
    }
    pub fn jwks_url(mut self, jwks_url: &str) -> Self {
        self.jwks_url = Some(String::from(jwks_url));
        self
    }

    pub fn public_key(mut self, public_key: DecodingKey) -> Self {
        self.public_key = Some(public_key);
        self
    }

    pub fn audience(mut self, audience: HashSet<String>) -> Self {
        self.audience = Some(audience);
        self
    }

    pub async fn decode<T>(&self, token: &String) -> TmsResult<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        // if jwks url is provided, use that.
        let decoding_key = match self.jwks_url {
            Some(ref jwks_url) => {
                let jwks_url = match jwks_url.parse() {
                    Ok(url) => url,
                    Err(error) => {
                        return Err(format!("Error parsing Url String:: {}", error));
                    }
                };

                let jwks = get_public_key(&jwks_url).await?;

                let token_header = match decode_header(token) {
                    Ok(header) => header,
                    Err(error) => return Err(error.to_string()),
                };

                let jwk = match token_header.kid {
                    Some(kid) => jwks.find(&kid),
                    None => jwks.keys.get(0),
                };

                let decoding_key_result = match jwk {
                    Some(jwk) => DecodingKey::from_jwk(jwk),
                    None => return Err("Unable to find the proper key".to_string()),
                };

                match decoding_key_result {
                    Ok(decoding_key) => decoding_key,
                    Err(error) => return Err(error.to_string()),
                }
            }

            None => match &self.public_key {
                Some(public_key) => public_key.to_owned(),
                None => return Err("No key available for decoding JWT".to_string()),
            },
        };
        // TODO: remember this trick for generics.  It helps a lot!!
        // println!("Type is: {}", std::any::type_name::<T>());
        let mut validation = Validation::new(Algorithm::RS256);
        if let Some(audience) = &self.audience {
            validation.aud = self.audience.to_owned();
        }

        let decoded: TokenData<T> = match decode(token, &decoding_key, &validation) {
            Ok(decoded) => decoded,
            Err(error) => return Err(error.to_string()),
        };

        Ok(decoded.claims)
    }
}

pub async fn get_public_key(pub_key_url: &Url) -> TmsResult<JwkSet> {
    let client = reqwest::Client::new();
    let jwks_string = match client.get(pub_key_url.as_str()).send().await {
        Ok(mut response) => match (response.text().await) {
            Ok(jwks_string) => jwks_string,
            Err(error) => return Err(error.to_string()),
        },
        Err(error) => return Err(error.to_string()),
    };
    serde_json::from_str(jwks_string.as_str()).map_err(|error| error.to_string())
}

pub async fn exchange_code_for_token<R>(idp: &Idp, code: &String) -> TmsResult<R>
where
    R: for<'a> Deserialize<'a>,
{
    // TODO: add client secret/id to config
    let form_params = [
        ("grant_type", "authorization_code"),
        (
            "client_id",
            "cilogon:/client_id/3d38b53c9709489136c9b68c8f769c99",
        ),
        (
            "client_secret",
            "_-9v-A023hVLquAlLjToWSQoF5XOwCKjG8i8QHtCN3K4c8fjF_ILSmf4ZekXafk0VC6q_T66WOntSUxgJLjN1Q",
        ),
        ("code", code),
    ];
    let client = reqwest::Client::new();
    let response = client
        .post(&idp.oauth2_token_url)
        .form(&form_params)
        .send()
        .await;

    match response {
        Ok(response) => {
            let token_string = match response.text().await {
                Ok(response_string) => response_string,
                Err(error) => return Err(format!("Error getting response body: {}", error)),
            };

            let token = match serde_json::from_str::<R>(&token_string) {
                Ok(token) => token,
                Err(error) => return Err(error.to_string()),
            };

            Ok(token)
        }
        Err(error) => return Err(error.to_string()),
    }
}

pub struct JwtEncoderBuilder<T> {
    header: Header,
    claims: T,
    encoding_key: EncodingKey,
}

impl<T> JwtEncoderBuilder<T>
where
    T: Serialize,
{
    pub fn builder(header: Header, claims: T, encoding_key: EncodingKey) -> JwtEncoderBuilder<T> {
        JwtEncoderBuilder {
            header,
            claims,
            encoding_key,
        }
    }
    pub async fn encode(&self) -> TmsResult<String> {
        match encode(&self.header, &self.claims, &self.encoding_key) {
            Ok(encoded) => Ok(encoded),
            Err(error) => Err(format!("Error encoding JWT: {}", error)),
        }
    }
}

pub async fn decode_access_token<T>(idp: &Idp, id_token: &String) -> TmsResult<T>
where
    T: for<'a> Deserialize<'a>,
{
    let audience =
        HashSet::from(["cilogon:/client_id/3d38b53c9709489136c9b68c8f769c99".to_string()]);
    match JwtDecoderBuilder::builder()
        .jwks_url(&idp.oauth2_jwks_url)
        .audience(audience)
        .decode(id_token)
        .await
    {
        Ok(decoded) => Ok(decoded),
        Err(error) => Err(format!("Error decoding JWT: {}", error)),
    }
}

pub async fn make_auth_token(claims: HashMap<String, Value>) -> TmsResult<String> {
    let header = Header::new(Algorithm::RS256);
    let encoding_key = match EncodingKey::from_rsa_pem(&jwt_private_key().into_bytes()) {
        Ok(key) => key,
        Err(error) => return Err(error.to_string()),
    };

    JwtEncoderBuilder::builder(header, claims, encoding_key)
        .encode()
        .await
}
