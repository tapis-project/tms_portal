CREATE TABLE IF NOT EXISTS allowed_redirects (
        uri                     TEXT                    NOT NULL,
        client_id               TEXT                    NOT NULL,
        created                 TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
        updated                 TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
        constraint              fk_client_id  FOREIGN KEY (client_id) REFERENCES clients(id)
);

