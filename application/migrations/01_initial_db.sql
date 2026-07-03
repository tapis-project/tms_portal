CREATE TABLE IF NOT EXISTS identity_providers
(
    uuid                  UUID PRIMARY KEY  NOT NULL DEFAULT gen_random_uuid(),
    id                    TEXT              NOT NULL,
    name                  TEXT              NOT NULL,
    client_id             TEXT              NOT NULL,
    client_secret         TEXT              NOT NULL,
    identity_redirect_url TEXT              NOT NULL,
    oauth2_token_url      TEXT              NOT NULL,
    oauth2_jwks_url       TEXT,
    oauth2_public_key     TEXT,
    oidc_user_info_url    TEXT,
    scope                 TEXT,
    provider_type         TEXT              NOT NULL,
    supports_login        BOOLEAN           NOT NULL DEFAULT false,
    supports_resources    BOOLEAN           NOT NULL DEFAULT false,
    created               TIMESTAMPTZ       NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated               TIMESTAMPTZ       NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    UNIQUE (id)
);


CREATE TABLE IF NOT EXISTS identity_provider_types
(
    provider_type TEXT PRIMARY KEY            NOT NULL,
    created               TIMESTAMPTZ       NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated               TIMESTAMPTZ       NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

INSERT INTO identity_provider_types (provider_type)
VALUES ('globus');
INSERT INTO identity_provider_types (provider_type)
VALUES ('tacc_tapis');

ALTER TABLE identity_providers
    ADD CONSTRAINT fk_provider FOREIGN KEY (provider_type) REFERENCES identity_provider_types (provider_type);

CREATE TABLE IF NOT EXISTS keys
(
    kid             TEXT PRIMARY KEY            NOT NULL,
    jwt_public_key  TEXT                        NOT NULL,
    jwt_private_key TEXT                        NOT NULL,
    created               TIMESTAMPTZ       NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated               TIMESTAMPTZ       NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

CREATE TABLE IF NOT EXISTS clients
(
    id      TEXT PRIMARY KEY            NOT NULL,
    name    TEXT                        NOT NULL,
    secret  TEXT                        NOT NULL,
    kid     TEXT                        NOT NULL,
    created               TIMESTAMPTZ       NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated               TIMESTAMPTZ       NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    constraint fk_kid FOREIGN KEY (kid) REFERENCES keys (kid)
);

CREATE TABLE IF NOT EXISTS configuration
(
    config_name  TEXT PRIMARY KEY            NOT NULL,
    config_value JSONB                       NOT NULL,
    created               TIMESTAMPTZ       NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated               TIMESTAMPTZ       NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

CREATE TABLE IF NOT EXISTS allowed_redirects
(
    uri       TEXT                        NOT NULL,
    client_id TEXT                        NOT NULL,
    created               TIMESTAMPTZ       NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated               TIMESTAMPTZ       NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    constraint fk_client_id FOREIGN KEY (client_id) REFERENCES clients (id)
);

