use crate::services::service_error::ServiceError::Internal;
use anyhow::{Context, Result};
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{
    decode, decode_header, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData,
    Validation,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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
    pub fn jwks_url(mut self, jwks_url: &Option<String>) -> Self {
        if let Some(jwks_url) = jwks_url {
            self.jwks_url = Some(String::from(jwks_url));
        }
        self
    }

    pub fn public_key(mut self, public_key: &Option<DecodingKey>) -> Self {
        if let Some(public_key) = public_key {
            self.public_key = Some(public_key.clone());
        }
        self
    }

    pub fn audience(mut self, audience: HashSet<String>) -> Self {
        self.audience = Some(audience);
        self
    }

    pub async fn decode<T>(&self, token: &String) -> Result<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        // if jwks url is provided, use that.
        let decoding_key = match self.jwks_url {
            Some(ref jwks_url) => {
                let jwks = get_public_key(&jwks_url).await?;
                let token_header = decode_header(token)?;

                let jwk = match token_header.kid {
                    Some(kid) => jwks.find(&kid),
                    None => jwks.keys.get(0),
                };

                match jwk {
                    Some(jwk) => DecodingKey::from_jwk(jwk)?,
                    None => {
                        return Err(Internal("Unable to find the proper key".to_string()).into());
                    }
                }
            }

            None => match &self.public_key {
                Some(public_key) => public_key.to_owned(),
                None => {
                    return Err(Internal("No key available for decoding JWT".to_string()).into());
                }
            },
        };

        // TODO: remember this trick for generics.  It helps a lot!!
        // println!("Type is: {}", std::any::type_name::<T>());
        let mut validation = Validation::new(Algorithm::RS256);
        if let Some(_) = &self.audience {
            validation.aud = self.audience.to_owned();
        }

        let decoded: TokenData<T> = decode(token, &decoding_key, &validation)?;
        Ok(decoded.claims)
    }
}

pub async fn get_public_key(pub_key_url: &String) -> Result<JwkSet> {
    let client = reqwest::Client::new();
    let response = client.get(pub_key_url.as_str()).send().await?;
    let jwks_string = response.text().await?;
    serde_json::from_str(jwks_string.as_str()).map_err(|error| error.into())
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
    pub async fn encode(&self) -> Result<String> {
        encode(&self.header, &self.claims, &self.encoding_key).context("Error encodeing JWT")
    }
}
