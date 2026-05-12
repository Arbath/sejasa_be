-- Add up migration script here
CREATE TABLE user_skills (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    descriptions TEXT,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE
);