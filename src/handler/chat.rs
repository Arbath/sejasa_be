use std::sync::atomic::Ordering;

use axum::{extract::{State, WebSocketUpgrade, ws::{Message, WebSocket}}, response::{IntoResponse, Response}};
use futures_util::{SinkExt, stream::StreamExt};
use http::Uri;
use tokio::sync::mpsc::{self, error::TrySendError};
use tracing::info;

use crate::{middleware::auth::AuthUser, state::AppState, utils::response::{AppError, WebResponse}};

pub async fn test(
    uri: Uri,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let data = user;
    Ok(WebResponse::ok(&uri, "berhasil", data))
}

pub async fn ws_hand(
    State(_): State<AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(handle_ws)
}

async fn handle_ws(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            info!("recive : {:?}", &msg);
            msg
        } else {
            info!("Websocket disconnected");
            return ;
        };

        if socket.send(msg).await.is_err() {
            info!("Websocket error");
            return ;
        }
    }
}

pub async fn ws_chat(
    AuthUser(_): AuthUser,
    ws: WebSocketUpgrade, 
    State(state
): State<AppState>) -> Result<impl IntoResponse, AppError> {
    Ok(ws.on_upgrade(|socket| chat_hand(socket, state)))
}

async fn chat_hand(socket: WebSocket, state: AppState) {
    let my_id = state.n_user_id.fetch_add(1, Ordering::Relaxed);

    let (mut sender, mut reciver) = socket.split();
    let welcome_msg = format!("User #{}, Berhasil bergabung ke grup chat!", my_id);
    if let Err(e) = sender.send(Message::Text(welcome_msg.into())).await {
        info!("Gagal mengirim welcome message: {}", e);
        return; // Stop jika koneksi langsung putus
    }

    let (tx, mut rx) = mpsc::channel(100);
    {
        state.users.write().await.insert(my_id, tx);
    }
    info!("User #{} terhubung", my_id);

    {
        let users = state.users.read().await;
        let join_msg = format!("User #{} baru saja bergabung.", my_id);
        
        for (&uid, user_tx) in users.iter() {
            if uid != my_id {
                // Gunakan try_send agar user baru tidak menunggu user lama yg lemot
                let _ = user_tx.try_send(Message::Text(join_msg.clone().into()));
            }
        }
    }
    
    // Task sender
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Task read and boardcast
    while let Some(Ok(msg)) = reciver.next().await {
        if let Message::Text(text) = msg {
            info!("User #{} mengirim: {}", my_id, text);

            let users = state.users.read().await;
            for (&uid, tx) in users.iter() {
                if uid != my_id {
                    let res_msg = format!("User #{}: {}", my_id, text);
                    match tx.try_send(Message::Text(res_msg.into())) {
                        Ok(_) => {}, // Berhasil kirim
                        Err(TrySendError::Full(_)) => {
                            // Antrian user ini penuh (mungkin koneksi lambat)
                            info!("Antrian pesan user {} penuh, pesan di-drop", uid);
                        },
                        Err(TrySendError::Closed(_)) => {
                            // User sudah disconnect tapi belum terhapus dari map
                        }
                    }
                }
            }
        }
    }
    {
        state.users.write().await.remove(&my_id);
    }

    info!("User #{} disconnect", my_id);
    send_task.abort();
}