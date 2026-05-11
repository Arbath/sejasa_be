-- Add up migration script here
CREATE TABLE review (
    id SERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    project_id UUID REFERENCES projects(id) ON DELETE CASCADE,
    reviewer_id UUID REFERENCES users(id) ON DELETE SET NULL,
    rating DOUBLE PRECISION DEFAULT 0.0,
    review TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);