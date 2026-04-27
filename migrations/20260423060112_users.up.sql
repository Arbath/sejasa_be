-- Add up migration script here
-- Mengaktifkan ekstensi untuk generate UUID otomatis (jika belum ada)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 1. Table: Users
CREATE TABLE Users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password VARCHAR(255) NOT NULL,
    is_superuser BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    account_type VARCHAR(50)
);

-- 2. Table: Categories
CREATE TABLE Categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 3. Table: User Profile
-- Menggunakan user_id sebagai Primary Key sekaligus Foreign Key karena relasinya kemungkinan 1-to-1
CREATE TABLE User_Profile (
    user_id UUID PRIMARY KEY REFERENCES Users(id) ON DELETE CASCADE,
    gender BOOLEAN NULL, -- opt<bool> berarti boleh null
    rating DOUBLE PRECISION,
    contact VARCHAR(50),
    coordinate VARCHAR(255),
    image TEXT,
    address TEXT
);

-- 4. Table: Projects
CREATE TABLE Projects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES Users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    address TEXT,
    max_participan INTEGER,
    descriptions TEXT, 
    requiremets JSONB,
    slug VARCHAR(255) UNIQUE,
    coordinate VARCHAR(255),
    rating DOUBLE PRECISION,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    category_id INTEGER REFERENCES Categories(id) ON DELETE SET NULL
);

-- 5. Table: Hastags (Menyesuaikan penulisan di gambar)
CREATE TABLE Hastags (
    id SERIAL PRIMARY KEY,
    project_id UUID REFERENCES Projects(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 6. Table: Project Participant
CREATE TABLE Project_Participant (
    id SERIAL PRIMARY KEY,
    user_id UUID REFERENCES Users(id) ON DELETE CASCADE,
    project_id UUID REFERENCES Projects(id) ON DELETE CASCADE,
    status VARCHAR(50)
);

-- 7. Table: Chats
CREATE TABLE Chats (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES Users(id) ON DELETE CASCADE,
    project_id UUID REFERENCES Projects(id) ON DELETE CASCADE
);

-- 8. Table: Detail Chat
CREATE TABLE Detail_Chat (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    chat_id UUID REFERENCES Chats(id) ON DELETE CASCADE,
    content TEXT,
    file TEXT,
    is_read BOOLEAN DEFAULT FALSE
);