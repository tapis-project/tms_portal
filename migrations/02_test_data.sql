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

