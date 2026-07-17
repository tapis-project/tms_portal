CREATE TABLE IF NOT EXISTS auth_code_data
(
    auth_code       TEXT PRIMARY KEY            NOT NULL,
    client_id       TEXT                        NOT NULL,
    redirect_uri    TEXT                        NOT NULL,
    created               TIMESTAMPTZ       NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated               TIMESTAMPTZ       NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);
