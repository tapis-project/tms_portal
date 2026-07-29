-- Add the globus identity provider (as login identity provider)
INSERT INTO identity_providers 
        (id, name, client_id, client_secret, identity_redirect_url, 
                oauth2_token_url, oauth2_jwks_url, oidc_user_info_url, 
                scope, provider_type, supports_login, 
                supports_resources)
VALUES 
        ('globus_idp', 'Globus IDP', '<put your globus client id here>',
                '<put your globus client secret here>',
                'https://auth.globus.org/v2/oauth2/authorize', 
                'https://auth.globus.org/v2/oauth2/token', 
                'https://auth.globus.org/jwk.json', '',
                'openid profile email', 'globus', true, false);

-- Add the tacc identity provider (as resource provider)
INSERT INTO identity_providers (id, name, client_id, client_secret, identity_redirect_url, 
oauth2_token_url, oauth2_jwks_url, oidc_user_info_url, scope, provider_type, supports_login, 
supports_resources) VALUES ('tacc', 'TACC Resource Provider', '<put your tacc resource provider client id here>',
'<put your tacc resource provider client secret here>',
'https://tacc.tapis.io/v3/oauth2/authorize', 'https://tacc.tapis.io/v3/oauth2/tokens', 
'https://tacc.tapis.io/v3/tokens/.well-known/jwks.json', '',
'openid profile email', 'tacc_tapis', false, true);

-- token signing key - tms
INSERT INTO keys (kid, jwt_public_key, jwt_private_key) 
VALUES('<put your token signing kid here>', 
'-----BEGIN PUBLIC KEY-----
<put your token signing public key here>
-----END PUBLIC KEY-----', 
'-----BEGIN PRIVATE KEY-----
<put your token signing private key here>
-----END PRIVATE KEY-----'
);

-- state signing key - tms
INSERT INTO keys (kid, jwt_public_key, jwt_private_key) 
VALUES('<put your state signing kid here>',
'-----BEGIN PUBLIC KEY-----
<put your state signing public key here>
-----END PUBLIC KEY-----',
'-----BEGIN PRIVATE KEY-----
<put your state signing private key here>
-----END PRIVATE KEY-----'
);

-- Add tms client
INSERT INTO clients (id, name, secret) 
        VALUES('tms', 'Tms Service', 'tms');

-- Add allowed redirects (with and without trailing slash)
INSERT INTO allowed_redirects (client_id, uri) 
        VALUES('tms', 'http://localhost:8080');

INSERT INTO allowed_redirects (client_id, uri) 
        VALUES('tms', 'http://localhost:8080/');

-- Add allowed redirects for resource provider oauth callback
INSERT INTO allowed_redirects (uri, client_id)
        VALUES ('http://localhost:8080/resources/providers/callback', 'tms');

-- Add configuration settings
INSERT INTO configuration (config_name, config_value) 
        VALUES ('state_key', '{"kid":"<put your state signing kid here>"}'::jsonb);
INSERT INTO configuration (config_name, config_value) 
        VALUES ('jwt_config', '{"default_expiration_minutes":"60", "signing_key_kid":"<put your token signing kid here>"}'::jsonb);
INSERT INTO configuration (config_name, config_value) 
        VALUES ('oauth_config', '{"login_oauth_provider":"globus_idp"}'::jsonb);
INSERT INTO configuration (config_name, config_value) 
VALUES 
        ('http_config', '{"base_url":"http://localhost:8080/", 
                "identity_provider_callback_endpoint":"login/callback", 
                "resource_provider_callback_endpoint":"resources/providers/callback", 
                "oauth_provider_callback_endpoint":"oauth2/callback", 
                "token_endpoint":"oauth2/token", "authorization_endpoint":"oauth2/authorization", 
                "revocation_endpoint":"oauth2/token/revoke", "jwks_endpoint":"jwks.json" }'::jsonb);