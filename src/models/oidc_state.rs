use std::collections::HashMap;
use jsonwebtoken::{decode, encode, jws, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation};
use jsonwebtoken::jws::Jws;
use serde::{Deserialize, Serialize};
use serde::de::Unexpected::Option;
use serde_json::Value;
use crate::models::responses::TmsApiErrorResult;

#[derive(Debug, Deserialize, Serialize)]
pub struct OAuthState {
    // TODO: generate crypto random nonce (or something)
    // TODO: can the expiration work better?
    pub idp_id: String,
    pub exp: u64,
}

impl TryFrom<&String> for OAuthState {
    type Error = (TmsApiErrorResult);

    fn try_from(state_string: &String) -> Result<Self, TmsApiErrorResult> {
        let key = match DecodingKey::from_rsa_pem(public_key().as_bytes()) {
            Ok(key) => key,
            Err(err) => return Err(TmsApiErrorResult { message: err.to_string() })
        };

        let token_data:TokenData<OAuthState> = match decode(state_string, &key, &Validation::new(Algorithm::RS256)) {
            Ok(data) => data,
            Err(err) => return Err(TmsApiErrorResult { message: err.to_string() } )
        };

        Ok(token_data.claims)
    }
}

impl OAuthState {
    pub fn encode(&self) -> Result<String, TmsApiErrorResult> {
        let header = Header::new(Algorithm::RS256);
        let key = match EncodingKey::from_rsa_pem(private_key().as_bytes()) {
            Ok(key) => key,
            Err(err) => return Err(TmsApiErrorResult { message: err.to_string() } ),
        };
/*
        let mut claims:HashMap<String, Value> = HashMap::new();
        claims.insert("idp_id".to_string(), self.idp_id.clone().into());
*/
        // TODO: read a real key from somewhere
        match encode(&header, &self, &key) {
            Ok(state) => Ok(state),
            Err(err) => Err(TmsApiErrorResult { message: err.to_string() } ),
        }
    }
//     pub fn encode_jws(&self) -> Result<Jws<OAuthState>, TmsApiErrorResult> {
//         let header = Header::new(Algorithm::RS256);
//         // let header = Header::default();
//         let key = match EncodingKey::from_rsa_pem(private_key().as_bytes()) {
//             Ok(key) => key,
//             Err(err) => return Err(TmsApiErrorResult { message: err.to_string() } ),
//         };
//
//         // let mut claims:HashMap<String, Value> = HashMap::new();
//         // claims.insert("idp_id".to_string(), self.idp_id.clone().into());
//
//         // TODO: read a real key from somewhere
//         match jws::encode(&header, Some(self), &key) {
//             Ok(state) => Ok(state),
//             Err(err) => Err(TmsApiErrorResult { message: err.to_string() } ),
//         }
//     }
//
}
//
// pub fn decode_jws(jws:&Jws<OAuthState>) -> Result<OAuthState, TmsApiErrorResult> {
//     let status = jws::decode(jws, &key, &validation);
//     Ok(OAuthState {idp_id: "DUMMY".to_string()})
// }

fn public_key() -> String {
    "-----BEGIN PUBLIC KEY-----
        MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAztiv4g36LrvgytSospsO
        Fs2Q+zHuTofqYqotM2+/y/6n0v1klX0jMeBvtlX2vwPlKQ74EmlaOWglPL3ZAxdp
        KiAJNIf9Bw4dpanZHCLrJTIO+KTIkRiyWsE7u9ydzt16grPDeutxPfE97M2V1wZj
        9aEOo2zrPMvb9oxAFtZYPxEU0RkbuelKS2sxRMAHFCqOss7zjzjxXKwXw0NThcWj
        6n/9qODEZgiiteOtpTCd52R5enzrsUcCf2H94lQ03W3qe4RXsGIEBnZV1wgrNjPY
        tuZJFK72DDwKE+6TJ31BD932FbU8WIncmMJENK9WM1Ls82N8APAp0Y/GoqUT9dsh
        /QIDAQAB
        -----END PUBLIC KEY-----".to_string()
}

fn private_key() -> String {
    "-----BEGIN PRIVATE KEY-----
        MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDO2K/iDfouu+DK
        1Kiymw4WzZD7Me5Oh+piqi0zb7/L/qfS/WSVfSMx4G+2Vfa/A+UpDvgSaVo5aCU8
        vdkDF2kqIAk0h/0HDh2lqdkcIuslMg74pMiRGLJawTu73J3O3XqCs8N663E98T3s
        zZXXBmP1oQ6jbOs8y9v2jEAW1lg/ERTRGRu56UpLazFEwAcUKo6yzvOPOPFcrBfD
        Q1OFxaPqf/2o4MRmCKK1462lMJ3nZHl6fOuxRwJ/Yf3iVDTdbep7hFewYgQGdlXX
        CCs2M9i25kkUrvYMPAoT7pMnfUEP3fYVtTxYidyYwkQ0r1YzUuzzY3wA8CnRj8ai
        pRP12yH9AgMBAAECggEAIro4lVBZb501eXhItmvX6rYqoDHa265wCI6ftiIN5nbj
        wEWwrHRtA0O2HbvDCIDj6YfM2HJ9pmO41MDe/WjhzCPCx+II9jVFfvnMLLAkIIOO
        BerMjafC0f/dQoYgrIl8txLtP+blhUvKZMaDYK1+/M9EgOWZfQSQ8ozLecoU2Ml9
        EmRscfBQ38WJCwANuKia8IqoquFHcigTkE9W19aOX+u2r4Riq9xwGCkDu/ceIZn9
        hCbKoeGLo4XTiZsAO7GfbaxvBnj8I+MFk6nCjXULkBmboxMkZy6Mk0K5RHeHz8hy
        PG0hyx99CDyKDUkfAEfNkHer9EiSDg+vB/zfV9iXlQKBgQDfSTVSb1xCAGpErW1R
        2xq5YDwpqFf574TkS8QcxZkemXbeY1nihnblTkAnA0gS5xtcN6UdbTA7uTggJ8DY
        +KIsU0SSAQXYO54a9gSwhDJszgdYSgUAdcXjCnwK/mvu9Ikr0mX8M+fF0nTQq66P
        TZ1U0+wCd/wA3oaga52L0L5WIwKBgQDtJuF2o/4QmNfo7QLLWx3BhPpLwySSLyjK
        J3Z8RDl/mu13s6UfzX0ge4F3ZLiin36fzjOSyA0/mEV7Xyw5DDxZaCpB/9yKa3Le
        8AM35HrNQLiCuc0IrpEN2AXLuLgQfP9bAYza1di88xO4gDROKKDGQPzxtPYnKpoY
        wFmPto1ZXwKBgGu+X8SPh+0xVhYduYquN48MKPvRB+LK+U1QYimgD+r8EqftOQpd
        6DFuOPaaVsUIT+OH1l0EuymWjsa1aBFKqLbK12O8qp1U504LOOgUYmCuakzoKtG2
        Au8zt/d2HY8I4MgMlrnEMir7CvNGZM0xnqG4QUJPs4KX3k66nyNAbxgRAoGAZ1Q4
        iqP7kCm4cYHLZOWHeolBMX+OUK+Bm0tEgfKMBwBvFWfNu6SiL2QAzg+xrxHFb0W+
        DOdjdmEXbSDTuOuO4/nR573ezuTwQRjcnh7MLuBTRIpGPtEo3JpSNCiA8pY0AOgV
        AkiIzhMvdYVOxPtIyfkI/Ru24OvcBorQuyB0SAsCgYEApG475mdgwjgh3DWBPANF
        Vub+UwbDQCykXM+seNKGm9P6ld2aQQpbG3y1SWYR+4YUXe6IH8Ge8ZHBFOg0j3zF
        XWqXEMOVErQefgf66bbYe0Q7Dt45RJTAeBMUd1J0h4yCihJjVxzT6+PGrsiaTWQ/
        FggAdDQqkBVYkMRsNNXmCP4=
        -----END PRIVATE KEY-----".to_string()
}

