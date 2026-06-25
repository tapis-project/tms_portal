-- ---------------------------------------
-- user_mfa table
-- ---------------------------------------
-- This table records when a user's MFA validation will expire.
-- CREATE TABLE IF NOT EXISTS user_mfa
-- changed name because it's not really user_mfa.  I don't have strong feelings about what we call it though.
CREATE TABLE IF NOT EXISTS resource_provider_account_logins
(
    id                          SERIAL PRIMARY KEY,
    tms_user_id                 TEXT NOT NULL,
    resource_provider_account   TEXT NOT NULL,
    resource_provider_id        TEXT NOT NULL,
    last_login                  TIMESTAMPTZ NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    enabled                     BOOLEAN NOT NULL,
    created                     TIMESTAMPTZ NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated                     TIMESTAMPTZ NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    UNIQUE (tms_user_id, resource_provider_id, resource_provider_account),
    FOREIGN KEY(resource_provider_id) REFERENCES identity_providers(id)
);
ALTER TABLE resource_provider_account_logins OWNER TO tms_portal_user;

-- -- ---------------------------------------
-- -- user_hosts table
-- -- ---------------------------------------
-- -- If both tms_user_id and host_account are set to "*",
-- -- then all users have their identity linked on host.
-- CREATE TABLE IF NOT EXISTS user_hosts
-- (
--     id                SERIAL PRIMARY KEY,
--     tms_user_id       TEXT NOT NULL,
--     host              TEXT NOT NULL,
--     host_account      TEXT NOT NULL,
--     expires_at        TIMESTAMPTZ NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
--     created           TIMESTAMPTZ NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
--     updated           TIMESTAMPTZ NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
--     UNIQUE (tms_user_id, host, host_account),
--     FOREIGN KEY(tms_user_id) REFERENCES resource_provider_account_logins(tms_user_id) ON UPDATE CASCADE ON DELETE CASCADE
-- );
-- ALTER TABLE user_hosts OWNER TO tms_portal_user;
--
-- -- ---------------------------------------
-- -- delegations table
-- -- ---------------------------------------
-- -- If client_user_id "*", then the delegation applies to all users.
-- CREATE TABLE IF NOT EXISTS delegations
-- (
--     id                SERIAL PRIMARY KEY,
--     client_id         TEXT NOT NULL,
--     client_user_id    TEXT NOT NULL,
--     expires_at        TIMESTAMPTZ NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
--     created           TIMESTAMPTZ NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
--     updated           TIMESTAMPTZ NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
--     UNIQUE (client_id, client_user_id),
--     FOREIGN KEY(client_user_id) REFERENCES resource_provider_account_logins(tms_user_id) ON UPDATE CASCADE ON DELETE CASCADE,
--     FOREIGN KEY(client_id) REFERENCES clients(id) ON UPDATE CASCADE ON DELETE CASCADE
-- );
-- ALTER TABLE delegations OWNER TO tms_portal_user;
--
-- CREATE UNIQUE INDEX IF NOT EXISTS delg_user_client_idx ON delegations (client_id, client_user_id);
