-- Add up migration script here
CREATE TABLE chats (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    project_id UUID REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE detail_chat (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    chat_id UUID REFERENCES chats(id) ON DELETE CASCADE,
    sender_id UUID REFERENCES users(id) ON DELETE CASCADE,
    content TEXT,
    file TEXT,
    is_read BOOLEAN DEFAULT FALSE,
    send_at TIMESTAMPTZ DEFAULT now()
);