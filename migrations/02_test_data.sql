INSERT INTO idps (id, name, client_id, client_secret, identity_redirect_url, 
oauth2_token_url, oauth2_jwks_url, oidc_user_info_url, scope) 
VALUES ('cilogon_idp', 'CILogon IDP', 'cilogon:/client_id/3d38b53c9709489136c9b68c8f769c99',
'_-9v-A023hVLquAlLjToWSQoF5XOwCKjG8i8QHtCN3K4c8fjF_ILSmf4ZekXafk0VC6q_T66WOntSUxgJLjN1Q',
'https://cilogon.org/authorize', 'https://cilogon.org/oauth2/token', 
'https://cilogon.org/oauth2/certs', 'https://cilogon.org/oauth2/userinfo',
'openid profile email org.cilogon.userinfo');

INSERT INTO idps (id, name, client_id, client_secret, identity_redirect_url, 
oauth2_token_url, oauth2_public_key, oidc_user_info_url, scope) 
VALUES ('cilogon_idp2', 'CILogon IDP', 'cilogon:/client_id/3d38b53c9709489136c9b68c8f769c99',
'_-9v-A023hVLquAlLjToWSQoF5XOwCKjG8i8QHtCN3K4c8fjF_ILSmf4ZekXafk0VC6q_T66WOntSUxgJLjN1Q',
'https://cilogon.org/authorize', 'https://cilogon.org/oauth2/token', 
'-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAhpJRWPJAv9yolz9ewiLJ
Mk1udGZC0JSWX1OwqaTUpWgcvDricRmS0TVxWeDaHig7lprkb7YowOdBv20TWeeN
NO1HxTO5nVDwg2jQ8IliR2Gscwi9pC6gektC9CXEBEEJYnl0rx9kazSvMTwXD/92
jZt3k8ixbuNzX6ZcfotXe/vmFu7Jtgxr9XYzZXTMMjXLC4qt02oURVHOMwjR0ziu
IIn0wZXvy7kGScnRZgYvyTBpIfvsRtVIEye5XPnk+DCBRo769qMFCDXD1Qhc9ePa
xLseqhtO00XEqv8SZ6MnvjWpuvlxW6GPmVHK6h2tvD2Lk4GUO9qsj8wrU1KrsUgf
sQIDAQAB
-----END PUBLIC KEY-----', 'https://cilogon.org/oauth2/userinfo',
'openid profile email org.cilogon.userinfo');

INSERT INTO idps (id, name, client_id, client_secret, identity_redirect_url, 
oauth2_token_url, oauth2_jwks_url, oidc_user_info_url, scope) 
VALUES ('globus_idp3', 'Globus IDP', 'aab240d2-2df2-42d4-a67b-eb268076fa9d',
'C1TBwDH0lj0zT33sVclDAPBzauG89JRazpE6DCDvkrM=',
'https://auth.globus.org/v2/oauth2/authorize', 'https://auth.globus.org/v2/oauth2/token', 
'https://auth.globus.org/jwk.json', '',
'openid profile email');

INSERT INTO clients (id, name, secret, jwt_public_key, jwt_private_key) 
VALUES('tms_test_client_id', 'Tms Test Client', 'tms_test_client_secret',
'-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAztiv4g36LrvgytSospsO
Fs2Q+zHuTofqYqotM2+/y/6n0v1klX0jMeBvtlX2vwPlKQ74EmlaOWglPL3ZAxdp
KiAJNIf9Bw4dpanZHCLrJTIO+KTIkRiyWsE7u9ydzt16grPDeutxPfE97M2V1wZj
9aEOo2zrPMvb9oxAFtZYPxEU0RkbuelKS2sxRMAHFCqOss7zjzjxXKwXw0NThcWj
6n/9qODEZgiiteOtpTCd52R5enzrsUcCf2H94lQ03W3qe4RXsGIEBnZV1wgrNjPY
tuZJFK72DDwKE+6TJ31BD932FbU8WIncmMJENK9WM1Ls82N8APAp0Y/GoqUT9dsh
/QIDAQAB
-----END PUBLIC KEY-----', 
'-----BEGIN PRIVATE KEY-----
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
-----END PRIVATE KEY-----'
);


insert into configuration (config_name, config_value) VALUES ('state_key', '{"current":{"public_key":"-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAztiv4g36LrvgytSospsO\nFs2Q+zHuTofqYqotM2+/y/6n0v1klX0jMeBvtlX2vwPlKQ74EmlaOWglPL3ZAxdp\nKiAJNIf9Bw4dpanZHCLrJTIO+KTIkRiyWsE7u9ydzt16grPDeutxPfE97M2V1wZj\n9aEOo2zrPMvb9oxAFtZYPxEU0RkbuelKS2sxRMAHFCqOss7zjzjxXKwXw0NThcWj\n6n/9qODEZgiiteOtpTCd52R5enzrsUcCf2H94lQ03W3qe4RXsGIEBnZV1wgrNjPY\ntuZJFK72DDwKE+6TJ31BD932FbU8WIncmMJENK9WM1Ls82N8APAp0Y/GoqUT9dsh\n/QIDAQAB\n-----END PUBLIC KEY-----", "private_key":"-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDO2K/iDfouu+DK\n1Kiymw4WzZD7Me5Oh+piqi0zb7/L/qfS/WSVfSMx4G+2Vfa/A+UpDvgSaVo5aCU8\nvdkDF2kqIAk0h/0HDh2lqdkcIuslMg74pMiRGLJawTu73J3O3XqCs8N663E98T3s\nzZXXBmP1oQ6jbOs8y9v2jEAW1lg/ERTRGRu56UpLazFEwAcUKo6yzvOPOPFcrBfD\nQ1OFxaPqf/2o4MRmCKK1462lMJ3nZHl6fOuxRwJ/Yf3iVDTdbep7hFewYgQGdlXX\nCCs2M9i25kkUrvYMPAoT7pMnfUEP3fYVtTxYidyYwkQ0r1YzUuzzY3wA8CnRj8ai\npRP12yH9AgMBAAECggEAIro4lVBZb501eXhItmvX6rYqoDHa265wCI6ftiIN5nbj\nwEWwrHRtA0O2HbvDCIDj6YfM2HJ9pmO41MDe/WjhzCPCx+II9jVFfvnMLLAkIIOO\nBerMjafC0f/dQoYgrIl8txLtP+blhUvKZMaDYK1+/M9EgOWZfQSQ8ozLecoU2Ml9\nEmRscfBQ38WJCwANuKia8IqoquFHcigTkE9W19aOX+u2r4Riq9xwGCkDu/ceIZn9\nhCbKoeGLo4XTiZsAO7GfbaxvBnj8I+MFk6nCjXULkBmboxMkZy6Mk0K5RHeHz8hy\nPG0hyx99CDyKDUkfAEfNkHer9EiSDg+vB/zfV9iXlQKBgQDfSTVSb1xCAGpErW1R\n2xq5YDwpqFf574TkS8QcxZkemXbeY1nihnblTkAnA0gS5xtcN6UdbTA7uTggJ8DY\n+KIsU0SSAQXYO54a9gSwhDJszgdYSgUAdcXjCnwK/mvu9Ikr0mX8M+fF0nTQq66P\nTZ1U0+wCd/wA3oaga52L0L5WIwKBgQDtJuF2o/4QmNfo7QLLWx3BhPpLwySSLyjK\nJ3Z8RDl/mu13s6UfzX0ge4F3ZLiin36fzjOSyA0/mEV7Xyw5DDxZaCpB/9yKa3Le\n8AM35HrNQLiCuc0IrpEN2AXLuLgQfP9bAYza1di88xO4gDROKKDGQPzxtPYnKpoY\nwFmPto1ZXwKBgGu+X8SPh+0xVhYduYquN48MKPvRB+LK+U1QYimgD+r8EqftOQpd\n6DFuOPaaVsUIT+OH1l0EuymWjsa1aBFKqLbK12O8qp1U504LOOgUYmCuakzoKtG2\nAu8zt/d2HY8I4MgMlrnEMir7CvNGZM0xnqG4QUJPs4KX3k66nyNAbxgRAoGAZ1Q4\niqP7kCm4cYHLZOWHeolBMX+OUK+Bm0tEgfKMBwBvFWfNu6SiL2QAzg+xrxHFb0W+\nDOdjdmEXbSDTuOuO4/nR573ezuTwQRjcnh7MLuBTRIpGPtEo3JpSNCiA8pY0AOgV\nAkiIzhMvdYVOxPtIyfkI/Ru24OvcBorQuyB0SAsCgYEApG475mdgwjgh3DWBPANF\nVub+UwbDQCykXM+seNKGm9P6ld2aQQpbG3y1SWYR+4YUXe6IH8Ge8ZHBFOg0j3zF\nXWqXEMOVErQefgf66bbYe0Q7Dt45RJTAeBMUd1J0h4yCihJjVxzT6+PGrsiaTWQ/\nFggAdDQqkBVYkMRsNNXmCP4=\n-----END PRIVATE KEY-----"} }'::jsonb);










