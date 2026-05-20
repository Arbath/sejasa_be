use axum::{extract::{WebSocketUpgrade, ws::{CloseFrame, Message, Utf8Bytes, close_code}}, response::{IntoResponse}};
use http::Uri;
use uuid::Uuid;

use crate::{middleware::auth::AuthUser, service::chat::ChatService, utils::{request::ValidatedPath, response::{ApiError, AppError, WebResponse}}};

pub async fn ws_chat(
    ValidatedPath(chat_id): ValidatedPath<Uuid>,
    AuthUser(user): AuthUser,
    ws: WebSocketUpgrade,
    service: ChatService
) -> Result<impl IntoResponse, AppError> {
    Ok(ws.on_upgrade(move |mut socket| async move { 
        let chat_result = service.chat_repo.find_chat(chat_id).await;
        let chat = match chat_result {
            Ok(c) => c,
            Err(_) => {
                let _ = socket.send(Message::Close(Some(CloseFrame {
                    code: close_code::INVALID, // 1007 atau custom code
                    reason: Utf8Bytes::from("Chat room tidak ditemukan!"),
                }))).await;
                return;
            }
        };

        let is_project_owner_result: Result<bool, sqlx::Error> = service.project_repo.check_project_ownership(chat.project_id, user.id).await;
        
        if let Ok(is_project_owner) = is_project_owner_result {
            let is_participant = chat.user_id == user.id;
            if !is_project_owner && !is_participant {
                
                let _ = socket.send(Message::Close(Some(CloseFrame {
                    code: close_code::POLICY, 
                    reason: Utf8Bytes::from("Kamu tidak punya akses chat ini!"),
                }))).await;
                return;
            }
        } else {
             let _ = socket.send(Message::Close(Some(CloseFrame {
                    code: close_code::ERROR, // 1011 Internal Error
                    reason: Utf8Bytes::from("Gagal memverifikasi project!"),
                }))).await;
                return;
        }

        // Jika semua lolos, oper socket yang sudah valid ke logika chatting
        service.chat_hand(socket, user, chat_id).await;
    }))
}

pub async fn find_chat_user_hand(
    uri: Uri,
    AuthUser(user) : AuthUser,
    service: ChatService,
) -> Result<impl IntoResponse, ApiError> {
    let res = service.get_chat_user(user).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Success!", res))
}

pub async fn find_chat_project_hand(
    ValidatedPath(project_id): ValidatedPath<Uuid>,
    uri: Uri,
    AuthUser(user) : AuthUser,
    service: ChatService,
) -> Result<impl IntoResponse, ApiError> {
    let res = service.get_chat_project(project_id, user).await.map_err(|e|e.with_path(&uri))?;
    Ok(WebResponse::ok(&uri, "Success!", res))
}
