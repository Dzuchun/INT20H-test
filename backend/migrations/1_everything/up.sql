CREATE TABLE IF NOT EXISTS users
(
    id            UUID PRIMARY KEY,
    name          VARCHAR(32)  NOT NULL,
    email         VARCHAR(320) NOT NULL,
    password_hash VARCHAR(255) NOT NULL
);

CREATE TABLE IF NOT EXISTS avatars
(
    id           UUID PRIMARY KEY,
    image_data   BYTEA        NOT NULL,
    content_type VARCHAR(255) NOT NULL
);

CREATE TABLE IF NOT EXISTS quests
(
    id          UUID PRIMARY KEY,
    owner       UUID    NOT NULL,
    title       TEXT,
    description TEXT,
    pages       INTEGER NOT NULL CHECK (pages >= 0),
    published   BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS quests_pages
(
    id                 UUID UNIQUE NOT NULL,
    page               INTEGER     NOT NULL CHECK (page >= 0),
    source             TEXT        NOT NULL,
    time_limit_seconds INTEGER CHECK (time_limit_seconds >= 0),
    PRIMARY KEY (id, page)
);

CREATE TABLE IF NOT EXISTS quests_applied
(
    user_id         UUID      NOT NULL,
    quest_id        UUID      NOT NULL,
    started_at      TIMESTAMP NOT NULL,
    finished_at     TIMESTAMP NULL,
    completed_pages INTEGER   NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, quest_id)
);
