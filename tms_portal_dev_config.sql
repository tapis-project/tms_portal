-- Add the globus identity provider (as login identity provider)
INSERT INTO identity_providers 
        (id, name, client_id, client_secret, identity_redirect_url, 
                oauth2_token_url, oauth2_jwks_url, oidc_user_info_url, 
                scope, provider_type, supports_login, 
                supports_resources)
VALUES 
        ('globus_idp', 'Globus IDP', 
                '@globus-client-id.age@',
                '@globus-client-secret.age@',
                'https://auth.globus.org/v2/oauth2/authorize', 
                'https://auth.globus.org/v2/oauth2/token', 
                'https://auth.globus.org/jwk.json', '',
                'openid profile email', 'globus', true, false);

-- Add the tacc identity provider (as resource provider)
INSERT INTO identity_providers (id, name, client_id, client_secret, identity_redirect_url, 
oauth2_token_url, oauth2_jwks_url, oidc_user_info_url, scope, provider_type, supports_login, 
supports_resources) VALUES ('tacc', 'TACC Resource Provider', 
'@tacc-resource-provider-client-id.age@',
'@tacc-resource-provider-client-secret.age@',
'https://tacc.tapis.io/v3/oauth2/authorize', 
'https://tacc.tapis.io/v3/oauth2/tokens', 
'https://tacc.tapis.io/v3/tokens/.well-known/jwks.json', '',
'openid profile email', 'tacc_tapis', false, true);

-- token signing key - tms
INSERT INTO keys (kid, jwt_public_key, jwt_private_key) 
VALUES('@token-kid@', 
'@token-signing-public-key@', 
'@token-signing-key.age@'
);

-- state signing key - tms
INSERT INTO keys (kid, jwt_public_key, jwt_private_key) 
VALUES('@state-kid@',
'@state-signing-public-key@',
'@state-signing-key.age@'
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
        VALUES ('state_key', '{"kid":"@state-kid@"}'::jsonb);
INSERT INTO configuration (config_name, config_value) 
        VALUES ('jwt_config', 
        '{"default_expiration_minutes":"60", "signing_key_kid":"@signing-kid@"}'::jsonb);
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

INSERT INTO configuration (config_name, config_value)
        VALUES ('runtime_config', '{"config_directory":"application/resources/config", "logging_config_file_name":"log4rs.yml"}'::jsonb);


