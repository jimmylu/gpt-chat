use crate::{
    models::{Chat, CreateChat},
    AppError, AppState, User,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

pub(crate) async fn list_chat_handler(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("user: {:?}", user);
    let chats = Chat::fetch_all(user.ws_id as _, &state.pool).await?;

    Ok((StatusCode::OK, Json(chats)))
}

pub(crate) async fn create_chat_handler(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::create(user.ws_id as _, input, &state.pool).await?;

    Ok((StatusCode::CREATED, Json(chat)))
}

pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::get_by_id(id, &state.pool).await?;
    match chat {
        Some(chat) => Ok((StatusCode::OK, Json(chat))),
        None => Err(AppError::NotFound(format!("Chat with id {} not found", id))),
    }
}

pub(crate) async fn update_chat_handler() -> Result<impl IntoResponse, AppError> {
    Ok("update_chat")
}

pub(crate) async fn delete_chat_handler() -> Result<impl IntoResponse, AppError> {
    Ok("delete_chat")
}

pub(crate) async fn send_message_handler() -> Result<impl IntoResponse, AppError> {
    Ok("send_message")
}
