use anyhow::Context;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;
use std::str::FromStr;

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
    pub async fn encode(&self) -> anyhow::Result<String> {
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
