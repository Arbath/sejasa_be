use axum::{Router, extract::DefaultBodyLimit, routing::{get, post}};
use crate::{handler::chat::{test, ws_chat, ws_hand}, middleware::auth::AuthUser, state::AppState, utils::response::{ApiError, AppError, WebResponse}};

use axum::{extract::{Multipart}, response::IntoResponse};
use http::Uri;
use tokio::{fs::{self}, io::AsyncWriteExt};
use uuid::Uuid;
use serde_json::json;


async fn home() -> &'static str {
    "Sejasa rust backend is running..."
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(home))
        .route("/user", get(test))
        .route("/ws", get(ws_hand))
        .route("/ws/chat", get(ws_chat))
        .route("/api/upload", post(upload_file_handler)).layer(DefaultBodyLimit::max(10 * 1024 * 1024))
}

pub async fn upload_file_handler(uri: Uri, AuthUser(user): AuthUser, mut multipart: Multipart) -> Result<impl IntoResponse, ApiError> {
    let upload_dir = "./public/uploads";
    let mut file_url = String::new();

    while let Some(mut field) = multipart.next_field().await.map_err(|e|ApiError { error:AppError::BadRequest(format!("Failed read multipart: {}", e)), path: uri.to_string() })? {
        let field_name = field.name().unwrap_or("").to_string();
        
        if field_name == "document" || field_name == "image" {
            let original_name = field.file_name().unwrap_or("unknown.bin").to_string();
            let extension = std::path::Path::new(&original_name)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("bin")
                .to_lowercase();

            let allowed_extensions = vec!["jpg", "jpeg", "png", "webp", "pdf"];
            if !allowed_extensions.contains(&extension.as_str()) {
                return Err(ApiError { error:AppError::BadRequest(format!("File extension not allowed!")), path: uri.to_string() })?
            }

            let unique_filename = format!("{}.{}", Uuid::new_v4(), extension);
            
            let user_dir = format!("{}/{}", upload_dir, user.id);
            
            fs::create_dir_all(&user_dir).await.map_err(|e|ApiError { 
                error:AppError::InternalError(format!("Failed create user directory: {}", e)), 
                path: uri.to_string() 
            })?;

            let save_path = format!("{}/{}", user_dir, unique_filename);
            let mut file = fs::File::create(&save_path).await.map_err(|e|ApiError { error:AppError::InternalError(format!("Failed create file: {}", e)), path: uri.to_string() })?;

            while let Some(chunk) = field.chunk().await.map_err(|e|ApiError { error:AppError::BadRequest(format!("Failed read chunk file: {}", e)), path: uri.to_string() })? {
                file.write_all(&chunk).await.map_err(|e|ApiError { error:AppError::InternalError(format!("Failed write file: {}", e)), path: uri.to_string() })?;
            }

            file_url = format!("/uploads/{}/{}", user.id, unique_filename);
            
            break; 
        }
    }

    if file_url.is_empty() {
        return Err(ApiError { error:AppError::BadRequest(format!("File is empty!")), path: uri.to_string() })?
    }

    let res = json!({
        "file_url" : file_url
    });
    Ok(WebResponse::ok(&uri, "File uploaded!", res))
}