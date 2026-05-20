use axum::extract::{FromRef, FromRequestParts, ws::{Message, WebSocket}};
use tracing::{info, error};
use tokio::sync::mpsc::{self, error::TrySendError};
use futures_util::{SinkExt, stream::StreamExt};
use uuid::Uuid;

use crate::{models::{chat::{Chat, ChatMessage, WsEvent}, user::User}, repository::{chat::ChatRepository, project::ProjectRepository}, state::AppState, utils::response::AppError};

#[allow(dead_code)]
pub struct ChatService {
    pub chat_repo: ChatRepository,
    pub project_repo: ProjectRepository,
    state: AppState
}

impl ChatService {
    pub fn new(state: AppState) -> Self {
        let chat_repo = ChatRepository::new(state.database.clone());
        let project_repo = ProjectRepository::new(state.database.clone());

        Self { chat_repo, project_repo, state }
    }

    pub async fn chat_hand(&self, socket: WebSocket, user: User, chat_id: Uuid) {
        let (mut sender, mut receiver) = socket.split();

        // AUTO-LOAD HISTORY SEBELUM JOIN ROOM
        if let Ok(history) = self.chat_repo.find_all_chat_detail(chat_id).await {
            let event = WsEvent::History(history);
            if let Ok(json) = serde_json::to_string(&event) {
                let _ = sender.send(Message::Text(json.into())).await;
            }
        }

        // SETUP MPSC & JOIN KE DALAM ROOM
        let (tx, mut rx) = mpsc::channel(100);
        
        {
            let mut rooms = self.state.rooms.write().await;
            // Jika room belum ada, otomatis buat baru, lalu masukkan user
            rooms.entry(chat_id).or_default().insert(user.id, tx);
        }
        info!("User #{} bergabung ke chat_id #{}", user.id, chat_id);

        // Broadcast System Message (Optional)
        {
            let event = WsEvent::System(format!("User {} bergabung", user.id));
            let json = serde_json::to_string(&event).unwrap();
            
            let rooms = self.state.rooms.read().await;
            if let Some(room) = rooms.get(&chat_id) {
                for (&uid, user_tx) in room.iter() {
                    if uid != user.id {
                        let _ = user_tx.try_send(Message::Text(json.clone().into()));
                    }
                }
            }
        }

        // TASK SENDER (Mengirim pesan ke klien)
        let send_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if sender.send(msg).await.is_err() {
                    break;
                }
            }
        });

        // TASK RECEIVER & BROADCASTER
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                
                // Parse request dari frontend (Wajib JSON `ChatMessage`)
                match serde_json::from_str::<ChatMessage>(&text) {
                    Ok(chat_message) => {
                        
                        // LOGIKA IS_READ: 
                        // Cek apakah lawan bicara sedang online di room ini
                        let is_read = {
                            let rooms = self.state.rooms.read().await;
                            if let Some(room) = rooms.get(&chat_id) {
                                // Jika di room ada lebih dari 1 orang (artinya kamu dan lawan bicaramu), maka dibaca.
                                room.len() > 1 
                            } else {
                                false
                            }
                        };

                        // Simpan ke DB
                        let create_result = self.chat_repo
                            .create_chat_detail(chat_id, user.id, chat_message, is_read)
                            .await;

                        match create_result {
                            Ok(chat_detail) => {
                                // Format balasan menjadi JSON Event
                                let event = WsEvent::NewMessage(chat_detail);
                                let json_res = serde_json::to_string(&event).unwrap();

                                // Broadcast ke SEMUA ORANG DI DALAM ROOM INI SAJA
                                let rooms = self.state.rooms.read().await;
                                if let Some(room) = rooms.get(&chat_id) {
                                    for (&uid, tx) in room.iter() {
                                        match tx.try_send(Message::Text(json_res.clone().into())) {
                                            Ok(_) => {},
                                            Err(TrySendError::Full(_)) => {
                                                info!("Antrian pesan user {} penuh", uid);
                                            },
                                            Err(TrySendError::Closed(_)) => {}
                                        }
                                    }
                                }
                            }
                            Err(e) => error!("Gagal menyimpan chat ke DB: {:?}", e),
                        }
                    },
                    Err(_) => {
                        // Jika frontend mengirim data non-JSON, kita kirim error ke dia saja
                        let err_event = WsEvent::System("Format pesan tidak valid (Harus JSON)".to_string());
                        let json_err = serde_json::to_string(&err_event).unwrap();
                        
                        let rooms = self.state.rooms.read().await;
                        if let Some(room) = rooms.get(&chat_id) {
                            if let Some(my_tx) = room.get(&user.id) {
                                let _ = my_tx.try_send(Message::Text(json_err.into()));
                            }
                        }
                    }
                }
            }
        }

        // CLEANUP SAAT DISCONNECT
        {
            let mut rooms = self.state.rooms.write().await;
            if let Some(room) = rooms.get_mut(&chat_id) {
                room.remove(&user.id);
                // (Opsional) Hapus room dari RAM jika sudah kosong
                if room.is_empty() {
                    rooms.remove(&chat_id);
                }
            }
        }

        info!("User #{} disconnect dari chat_id #{}", user.id, chat_id);
        send_task.abort();
    }

    pub async fn get_chat_user(&self, user: User) -> Result<Vec<Chat>, AppError>{
        let q = self.chat_repo.find_chat_user(user.id).await?;
        Ok(q)
    }

    pub async fn get_chat_project(&self, project_id: Uuid, user: User) -> Result<Vec<Chat>, AppError>{
        let is_owner = self.project_repo.check_project_ownership(project_id, user.id).await?;
        if !is_owner {
            return Err(AppError::Forbidden("Anda tidak memiliki akses ke project ini".to_string()));
        }

        let q = self.chat_repo.find_chat_project(project_id).await?;
        Ok(q)
    }

}

impl<S> FromRequestParts<S> for ChatService
where  
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;
    async fn from_request_parts(
        _parts: &mut http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection>
    {
        let state = AppState::from_ref(state);
        Ok(ChatService::new(state))
    }
}