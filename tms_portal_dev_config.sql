-- Add the globus identity provider (as login identity provider)
INSERT INTO identity_providers 
        (id, name, client_id, client_secret, identity_redirect_url, 
                oauth2_token_url, oauth2_jwks_url, oidc_user_info_url, 
                scope, provider_type, supports_login, 
                supports_resources)
VALUES 
        ('globus_idp', 'Globus IDP', '${GLOBUS_CLIENT_ID}',
                '${GLOBUS_CLIENT_SECRET}',
                'https://auth.globus.org/v2/oauth2/authorize', 
                'https://auth.globus.org/v2/oauth2/token', 
                'https://auth.globus.org/jwk.json', '',
                'openid profile email', 'globus', true, false);

-- Add the tacc identity provider (as resource provider)
INSERT INTO identity_providers (id, name, client_id, client_secret, identity_redirect_url, 
oauth2_token_url, oauth2_jwks_url, oidc_user_info_url, scope, provider_type, supports_login, 
supports_resources) VALUES ('tacc', 'TACC Resource Provider', '${TACC_RP_CLIENT_ID}',
'${TACC_RP_CLIENT_SECRET}',
'https://tacc.tapis.io/v3/oauth2/authorize', 'https://tacc.tapis.io/v3/oauth2/tokens', 
'https://tacc.tapis.io/v3/tokens/.well-known/jwks.json', '',
'openid profile email', 'tacc_tapis', false, true);

-- token signing key - tms
INSERT INTO keys (kid, jwt_public_key, jwt_private_key) 
VALUES('${TMS_TOKEN_KID}',
'${TMS_TOKEN_PUB_KEY}',
'${TMS_TOKEN_PRIV_KEY}'
);

-- state signing key - tms
INSERT INTO keys (kid, jwt_public_key, jwt_private_key) 
VALUES('${TMS_STATE_KID}',
'${TMS_STATE_PUB_KEY}',
'${TMS_STATE_PRIV_KEY}'
);

-- Add tms client
INSERT INTO clients (client_id, name, secret, enabled) 
        VALUES('tms', 'Tms Service', 'tms', true);
-- Add allowed redirects (with and without trailing slash)
INSERT INTO allowed_redirects (client_id, uri) 
        VALUES('tms', '${BASE_URL}');

INSERT INTO allowed_redirects (client_id, uri) 
        VALUES('tms', '${BASE_URL}/');

-- Add allowed redirects for resource provider oauth callback
INSERT INTO allowed_redirects (uri, client_id)
        VALUES ('${BASE_URL}/resources/providers/callback', 'tms');

-- Add configuration settings
INSERT INTO configuration (config_name, config_value) 
        VALUES ('state_key', '{"kid":"${TMS_STATE_KID}"}'::jsonb);
INSERT INTO configuration (config_name, config_value) 
        VALUES ('jwt_config', '{"default_expiration_minutes":"60", "signing_key_kid":"${TMS_TOKEN_KID}"}'::jsonb);
INSERT INTO configuration (config_name, config_value) 
        VALUES ('oauth_config', '{"login_oauth_provider":"globus_idp"}'::jsonb);
INSERT INTO configuration (config_name, config_value) 
VALUES 
        ('http_config', '{"base_url":"${BASE_URL}/", 
                "identity_provider_callback_endpoint":"login/callback", 
                "resource_provider_callback_endpoint":"resources/providers/callback", 
                "oauth_provider_callback_endpoint":"oauth2/callback", 
                "token_endpoint":"oauth2/token", "authorization_endpoint":"oauth2/authorization", 
                "revocation_endpoint":"oauth2/token/revoke", "jwks_endpoint":"jwks.json" }'::jsonb);

INSERT INTO configuration (config_name, config_value)
        VALUES ('runtime_config', '{"config_directory":"application/resources/config", "logging_config_file_name":"log4rs.yml"}'::jsonb);

-- Temporary for testing locally
-- Add tapisauth client for tms-test tenant
INSERT INTO clients (client_id, name, secret, enabled) 
        VALUES('3b684a8a-c974-4001-ab57-3f9e5190caf1', '${TAPIS_TEST_TENANT_CLIENT_ID}', '${TAPIS_TEST_TENANT_CLIENT_SECRET}', true);

-- Add allowed redirects for tapis tms-test tenant oauth callback
INSERT INTO allowed_redirects (uri, client_id)
        VALUES ('https://linktest.develop.tapis.io/v3/oauth2/extensions/oa2/callback', '3b684a8a-c974-4001-ab57-3f9e5190caf1');

