-- Add migration script here
CREATE TABLE IF NOT EXISTS fenster.public.articles
(
    article_slug      TEXT      NOT NULL
        CONSTRAINT articles_pk
            PRIMARY KEY,
    article_title     TEXT      NOT NULL,
    article_content   TEXT      NOT NULL,
    article_author    TEXT      NOT NULL REFERENCES fenster.public.users,
    article_published BOOL      NOT NULL DEFAULT FALSE,
    creation_date     TIMESTAMP NOT NULL DEFAULT NOW(),
    editing_date      TIMESTAMP NOT NULL DEFAULT NOW()
);