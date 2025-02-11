CREATE TABLE users (
                       id UUID PRIMARY KEY,
                       name VARCHAR(32) NOT NULL,
                       email VARCHAR(320) NOT NULL,
                       password_hash VARCHAR(255) NOT NULL
);

CREATE TABLE avatars (
                         id UUID PRIMARY KEY,
                         image_data BYTEA NOT NULL,
                         content_type VARCHAR(255) NOT NULL
);