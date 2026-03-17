CREATE TABLE IF NOT EXISTS idps (
        id TEXT PRIMARY KEY,
        name TEXT,
        client_id TEXT,
        client_secret TEXT,
        identity_redirect_url TEXT,
        oauth2_token_url TEXT,
        oauth2_jwks_url TEXT,
        oauth2_info_url TEXT,
        user_info_url TEXT,
        scope TEXT,
        created TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
        updated TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

INSERT INTO idps (id, name, client_id, client_secret, identity_redirect_url, 
oauth2_token_url, oauth2_jwks_url, user_info_url, scope) 
VALUES ('cilogon_idp', 'CILogon IDP', 'cilogon:/client_id/3d38b53c9709489136c9b68c8f769c99',
'_-9v-A023hVLquAlLjToWSQoF5XOwCKjG8i8QHtCN3K4c8fjF_ILSmf4ZekXafk0VC6q_T66WOntSUxgJLjN1Q',
'https://cilogon.org/authorize', 'https://cilogon.org/oauth2/token', 
'https://cilogon.org/oauth2/certs', 'https://cilogon.org/oauth2/userinfo',
'openid profile email org.cilogon.userinfo');
