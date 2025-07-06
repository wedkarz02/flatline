-- Add migration script here

CREATE TABLE refresh_tokens (
    jti UUID PRIMARY KEY,
    sub UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    exp BIGINT NOT NULL,
    iat BIGINT NOT NULL,
    token_hash TEXT NOT NULL
)
