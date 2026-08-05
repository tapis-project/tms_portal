CREATE TABLE IF NOT EXISTS issued_tokens
(
    access_token  TEXT PRIMARY KEY  NOT NULL,
    expiration    TIMESTAMPTZ       NOT NULL,
    revoked       BOOLEAN           NOT NULL DEFAULT false,
    created       TIMESTAMPTZ       NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated       TIMESTAMPTZ       NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);
