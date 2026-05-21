use sqlx::PgPool;
use uuid::Uuid;

use crate::models::chat::*;

pub struct ChatRepository {
    pool: PgPool
}

impl ChatRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_chat_detail(&self, detail_id: Uuid) -> Result<ChatDetail, sqlx::Error> {
        sqlx::query_as::<_, ChatDetail>(
            r#"SELECT * FROM detail_chat WHERE id = $1"#
        )
        .bind(detail_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_all_chat_detail(&self, chat_id: Uuid) -> Result<Vec<ChatDetail>, sqlx::Error> {
        sqlx::query_as::<_, ChatDetail>(
            r#"SELECT * FROM detail_chat WHERE chat_id = $1 ORDER BY send_at ASC"#
        )
        .bind(chat_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create_chat_detail(&self, chat_id: Uuid, user_id: Uuid, message: ChatMessage, is_read: bool) -> Result<ChatDetail, sqlx::Error> {
        sqlx::query_as::<_, ChatDetail>(
            r#"INSERT INTO detail_chat(chat_id, sender_id, content, file, is_read) VALUES($1, $2, $3, $4, $5) RETURNING * "#
        )
        .bind(chat_id)
        .bind(user_id)
        .bind(message.content)
        .bind(message.file)
        .bind(is_read)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_chat(&self, chat_id: Uuid) -> Result<Chat, sqlx::Error> {
        sqlx::query_as::<_, Chat>(
            r#"SELECT * FROM chats WHERE id = $1"#
        )
        .bind(chat_id)
        .fetch_one(&self.pool)
        .await
    }
    
    pub async fn find_chat_project(&self, project_id: Uuid) -> Result<Vec<Chat>, sqlx::Error> {
        sqlx::query_as::<_, Chat>(
            r#"SELECT * FROM chats WHERE project_id = $1"#
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_chat_user(&self, user_id: Uuid) -> Result<Vec<Chat>, sqlx::Error> {
        sqlx::query_as::<_, Chat>(
            r#"SELECT * FROM chats WHERE user_id = $1"#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }
    
    pub async fn create_chats(&self, user_id: Uuid, project_id: Uuid) -> Result<Chat, sqlx::Error> {
        sqlx::query_as::<_, Chat>(
            r#"INSERT INTO chats(user_id, project_id) VALUES($1, $2) RETURNING * "#
        )
        .bind(user_id)
        .bind(project_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn chat_user_preview(&self, user_id: Uuid) -> Result<Vec<ChatPreview>, sqlx::Error> {
        sqlx::query_as::<_,ChatPreview>(
            r#"SELECT DISTINCT ON (c.id) 
            c.id,
            c.project_id,
            jsonb_build_object(
                'id', c.user_id,
                'name', u.name,
                'image', p.image
            ) AS user,
            COALESCE(u.name, '') AS title,
            COALESCE(cd.content, '') AS body,
            (
                SELECT COALESCE(COUNT(*), 0)::int4 
                FROM detail_chat 
                WHERE chat_id = c.id 
                AND is_read = false 
                AND sender_id != $1
            ) AS unread_msg,
            cd.send_at AS timestamp
            FROM chats c
            LEFT JOIN users u ON c.user_id = u.id
            LEFT JOIN user_profile p ON c.user_id = p.user_id
            LEFT JOIN detail_chat cd ON c.id = cd.chat_id
            WHERE c.user_id = $1 ORDER BY c.id, cd.send_at DESC"#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }
    
    pub async fn chat_project_preview(&self, project_id: Uuid) -> Result<Vec<ChatPreview>, sqlx::Error> {
        sqlx::query_as::<_,ChatPreview>(
            r#"SELECT DISTINCT ON (c.id) 
            c.id,
            c.project_id,
            jsonb_build_object(
                'id', c.user_id,
                'name', u.name,
                'image', p.image
            ) AS user,
            COALESCE(u.name, '') AS title,
            COALESCE(cd.content, '') AS body,
            (
                SELECT COALESCE(COUNT(*), 0)::int4 
                FROM detail_chat 
                WHERE chat_id = c.id 
                AND is_read = false 
                AND sender_id != $1
            ) AS unread_msg,
            cd.send_at AS timestamp
            FROM chats c
            LEFT JOIN users u ON c.user_id = u.id
            LEFT JOIN user_profile p ON c.user_id = p.user_id
            LEFT JOIN detail_chat cd ON c.id = cd.chat_id
            WHERE c.project_id = $1 ORDER BY c.id, cd.send_at DESC"#
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_read_bulk(&self, chat_id: Uuid, current_user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE detail_chat 
            SET is_read = true 
            WHERE chat_id = $1 
              AND sender_id != $2
              AND is_read = false
            "#
        )
        .bind(chat_id)
        .bind(current_user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}