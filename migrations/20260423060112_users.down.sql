-- Add down migration script here
CREATE TYPE account_type AS ENUM {
    'ORGANIZATION',
    'VOLUNTIER'
}

CREATE TABLE users {
    user_id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255),
    rating DOUBLE DEFAULT 0 CHECK rating(<= 5.0),
    address VARCHAR(255),
    coordinate VARCHAR(255),
    account_type VARCHAR(255),
    created_at TIMESTAMPZ NOW()
}