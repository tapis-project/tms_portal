use crate::services::service_error::ServiceError::Internal;
use anyhow::{Context, Result};
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{
    decode, decode_header, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData,
    Validation,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;

pub struct JwtDecoderBuilder {
    jwks_url: Option<String>,
    public_key_bytes: Option<Vec<u8>>,
    audience: Option<HashSet<String>>,
}

impl JwtDecoderBuilder {
    pub fn builder() -> JwtDecoderBuilder {
        JwtDecoderBuilder {
            jwks_url: None,
            public_key_bytes: None,
            audience: None,
        }
    }
    pub fn jwks_url(mut self, jwks_url: &Option<String>) -> Self {
        if let Some(jwks_url) = jwks_url {
            self.jwks_url = Some(String::from(jwks_url));
        }
        self
    }

    pub fn public_key(mut self, key_bytes: &[u8]) -> Self {
        self.public_key_bytes = Some(key_bytes.to_vec());
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
        let mut algorithm = Algorithm::RS256;
        // if jwks url is provided, use that.
        let decoding_key = match self.jwks_url {
            Some(ref jwks_url) => {
                let jwks = get_public_key(&jwks_url).await?;
                let token_header = decode_header(token)?;
                algorithm = token_header.alg;

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

            None => match &self.public_key_bytes {
                Some(public_key) => DecodingKey::from_rsa_pem(public_key.as_ref())?,
                None => {
                    return Err(Internal("No key available for decoding JWT".to_string()).into());
                }
            },
        };

        // TODO: remember this trick for generics.  It helps a lot!!
        // println!("Type is: {}", std::any::type_name::<T>());
        let mut validation = Validation::new(algorithm);
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
    claims: T,
    algorithm_name: String,
    kid: String,
    encoding_key_bytes: Vec<u8>,
}

impl<T> JwtEncoderBuilder<T>
where
    T: Serialize,
{
    pub fn builder(
        claims: T,
        encoding_key_bytes: &[u8],
        algorithm_name: &str,
        kid: &str,
    ) -> JwtEncoderBuilder<T> {
        JwtEncoderBuilder {
            algorithm_name: String::from(algorithm_name),
            kid: String::from(kid),
            claims,
            encoding_key_bytes: encoding_key_bytes.to_vec(),
        }
    }
    pub async fn encode(&self) -> Result<String> {
        let encoding_key = EncodingKey::from_rsa_pem(&self.encoding_key_bytes)?;
        let alg = Algorithm::from_str(&self.algorithm_name)?;
        let header = Header {
            typ: Some(String::from("JWT")),
            alg,
            kid: Some(self.kid.clone()),
            cty: None,
            jku: None,
            jwk: None,
            x5u: None,
            x5c: None,
            x5t: None,
            x5t_s256: None,
            crit: None,
            enc: None,
            zip: None,
            url: None,
            nonce: None,
            extras: Default::default(),
        };
        encode(&header, &self.claims, &encoding_key).context("Error encoding JWT")
    }
}
