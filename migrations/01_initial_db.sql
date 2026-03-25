CREATE TABLE IF NOT EXISTS idps (
        id                      TEXT    PRIMARY KEY     NOT NULL,
        name                    TEXT                    NOT NULL,
        client_id               TEXT                    NOT NULL,
        client_secret           TEXT                    NOT NULL,
        identity_redirect_url   TEXT                    NOT NULL,
        oauth2_token_url        TEXT                    NOT NULL,
        oauth2_jwks_url         TEXT,
        oauth2_public_key       TEXT,
        oidc_user_info_url      TEXT,
        scope                   TEXT,
        created                 TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
        updated                 TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

CREATE TABLE IF NOT EXISTS clients (
        id                      TEXT    PRIMARY KEY     NOT NULL,
        name                    TEXT                    NOT NULL,
        secret                  TEXT                    NOT NULL,
        created                 TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
        updated                 TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);
