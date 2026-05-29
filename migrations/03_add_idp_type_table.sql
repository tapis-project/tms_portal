ALTER TABLE idps
    ADD COLUMN provider TEXT NOT NULL DEFAULT 'globus';

CREATE TABLE IF NOT EXISTS idp_types
(
    provider TEXT PRIMARY KEY            NOT NULL,
    created  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated  TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

INSERT INTO idp_types (provider)
VALUES ('globus');

ALTER TABLE idps
    ADD CONSTRAINT fk_provider FOREIGN KEY (provider) REFERENCES idp_types (provider);

