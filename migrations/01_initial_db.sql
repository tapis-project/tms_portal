CREATE TABLE IF NOT EXISTS identity_providers
(
    id                    TEXT PRIMARY KEY            NOT NULL,
    name                  TEXT                        NOT NULL,
    client_id             TEXT                        NOT NULL,
    client_secret         TEXT                        NOT NULL,
    identity_redirect_url TEXT                        NOT NULL,
    oauth2_token_url      TEXT                        NOT NULL,
    oauth2_jwks_url       TEXT,
    oauth2_public_key     TEXT,
    oidc_user_info_url    TEXT,
    scope                 TEXT,
    provider_type         TEXT                        NOT NULL,
    created               TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated               TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

CREATE TABLE IF NOT EXISTS identity_provider_types
(
    provider_type TEXT PRIMARY KEY            NOT NULL,
    created       TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated       TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

INSERT INTO identity_provider_types (provider_type)
VALUES ('globus');

ALTER TABLE identity_providers
    ADD CONSTRAINT fk_provider FOREIGN KEY (provider_type) REFERENCES identity_provider_types (provider_type);

CREATE TABLE IF NOT EXISTS resource_providers
(
    id                    TEXT PRIMARY KEY            NOT NULL,
    name                  TEXT                        NOT NULL,
    client_id             TEXT                        NOT NULL,
    client_secret         TEXT                        NOT NULL,
    identity_redirect_url TEXT                        NOT NULL,
    oauth2_token_url      TEXT                        NOT NULL,
    oauth2_jwks_url       TEXT,
    oauth2_public_key     TEXT,
    oidc_user_info_url    TEXT,
    scope                 TEXT,
    provider_type         TEXT                        NOT NULL,
    created               TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated               TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

CREATE TABLE IF NOT EXISTS resource_provider_types
(
    provider_type TEXT PRIMARY KEY            NOT NULL,
    created       TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated       TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

INSERT INTO resource_provider_types (provider_type)
VALUES ('tacc_tapis');

ALTER TABLE resource_providers
    ADD CONSTRAINT fk_provider FOREIGN KEY (provider_type) REFERENCES resource_provider_types (provider_type);

CREATE TABLE IF NOT EXISTS keys
(
    kid             TEXT PRIMARY KEY            NOT NULL,
    jwt_public_key  TEXT                        NOT NULL,
    jwt_private_key TEXT                        NOT NULL,
    created         TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated         TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

CREATE TABLE IF NOT EXISTS clients
(
    id      TEXT PRIMARY KEY            NOT NULL,
    name    TEXT                        NOT NULL,
    secret  TEXT                        NOT NULL,
    kid     TEXT                        NOT NULL,
    created TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    constraint fk_kid FOREIGN KEY (kid) REFERENCES keys (kid)
);

CREATE TABLE IF NOT EXISTS configuration
(
    config_name  TEXT PRIMARY KEY            NOT NULL,
    config_value JSONB                       NOT NULL,
    created      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated      TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

CREATE TABLE IF NOT EXISTS allowed_redirects
(
    uri       TEXT                        NOT NULL,
    client_id TEXT                        NOT NULL,
    created   TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated   TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    constraint fk_client_id FOREIGN KEY (client_id) REFERENCES clients (id)
);

