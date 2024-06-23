-- Add migration script here
CREATE TABLE IF NOT EXISTS fenster.public.users
(
    user_id            TEXT NOT NULL
        CONSTRAINT users_pk
            PRIMARY KEY,
    user_name          TEXT NOT NULL,
    user_email         TEXT NOT NULL,
    user_author        BOOL DEFAULT FALSE,
    user_password_hash TEXT NOT NULL
);
