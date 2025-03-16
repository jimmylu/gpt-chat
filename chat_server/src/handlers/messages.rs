use axum::{
    extract::{Multipart, Path, State},
    http::{
        header::{CONTENT_LENGTH, CONTENT_TYPE},
        HeaderMap,
    },
    response::IntoResponse,
    Extension, Json,
};
use tokio::fs;
use tracing::{info, warn};

use crate::{models::ChatFile, AppError, AppState, User};

#[allow(unused)]
pub(crate) async fn update_message_handler() -> impl IntoResponse {
    "update_message"
}

#[allow(unused)]
pub(crate) async fn list_message_handler() -> impl IntoResponse {
    "list_message"
}

#[allow(unused)]
pub(crate) async fn upload_handler(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id.to_string();
    let base_dir = state.config.server.base_dir.join(ws_id.clone());
    let mut files: Vec<String> = vec![];
    while let Some(mut field) = multipart.next_field().await? {
        let filename = field.file_name().map(|name| name.to_string());
        let data = field.bytes().await;
        let (Some(filename), Ok(data)) = (filename, data) else {
            warn!("Failed to read multipart field");
            continue;
        };
        let file = ChatFile::new(ws_id.parse().unwrap(), filename, &data);
        let path = file.path(&base_dir);
        if path.exists() {
            info!("File already exists: {}", path.display());
        } else {
            fs::create_dir_all(path.parent().expect("file path parent should exist")).await?;
            fs::write(path, data).await?;
        }
        files.push(file.url());
    }
    Ok(Json(files))
}

#[allow(unused)]
pub(crate) async fn download_handler(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path((ws_id, file_url)): Path<(i64, String)>,
) -> Result<impl IntoResponse, AppError> {
    if user.ws_id != ws_id {
        return Err(AppError::NotFound(
            "File doesn't exist or you don't have permission".to_string(),
        ));
    }

    let base_dir = state.config.server.base_dir.join(ws_id.to_string());

    let file_path = base_dir.join(file_url);
    if !file_path.exists() {
        return Err(AppError::NotFound("File doesn't exist".to_string()));
    }
    let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
    let body = fs::read(file_path).await?;
    let mut header = HeaderMap::new();
    header.insert(CONTENT_TYPE, mime.to_string().parse().unwrap());
    header.insert(CONTENT_LENGTH, body.len().to_string().parse().unwrap());

    Ok((header, body))
}
