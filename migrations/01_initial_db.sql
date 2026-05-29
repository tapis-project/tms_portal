CREATE TABLE IF NOT EXISTS idps
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
    provider              TEXT                        NOT NULL,
    created               TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated               TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

CREATE TABLE IF NOT EXISTS idp_providers
(
    provider TEXT PRIMARY KEY            NOT NULL,
    created  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

INSERT INTO idp_providers (provider)
VALUES ('globus');

ALTER TABLE idps
    ADD CONSTRAINT fk_provider FOREIGN KEY (provider) REFERENCES idp_providers (provider);

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

