-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE OR REPLACE FUNCTION uuid_generate_v7()
RETURNS uuid
AS $$
DECLARE
  unix_ts_ms bytea;
  uuid_bytes bytea;
BEGIN
  unix_ts_ms = substring(int8send(floor(extract(epoch from clock_timestamp()) * 1000)::bigint) from 3);
  uuid_bytes = unix_ts_ms || gen_random_bytes(10);
  uuid_bytes = set_byte(uuid_bytes, 6, (get_byte(uuid_bytes, 6) & x'0f'::int) | x'70'::int);
  uuid_bytes = set_byte(uuid_bytes, 8, (get_byte(uuid_bytes, 8) & x'3f'::int) | x'80'::int);
  RETURN encode(uuid_bytes, 'hex')::uuid;
END
$$ LANGUAGE plpgsql;


CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password VARCHAR(255) NOT NULL,
    is_superuser BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT now(),
    account_type VARCHAR(50)
);

CREATE TABLE user_profile (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    gender VARCHAR(20),
    rating DOUBLE PRECISION DEFAULT 0.0,
    contact VARCHAR(50),
    latitude DOUBLE PRECISION DEFAULT 0.0,
    longitude DOUBLE PRECISION DEFAULT 0.0,
    image TEXT,
    address TEXT
);

CREATE TABLE refresh_tokens (
    token TEXT PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);