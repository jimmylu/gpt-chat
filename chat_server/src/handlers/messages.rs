use axum::response::IntoResponse;

#[allow(unused)]
pub(crate) async fn create_message_handler() -> impl IntoResponse {
    "create_message"
}

#[allow(unused)]
pub(crate) async fn update_message_handler() -> impl IntoResponse {
    "update_message"
}

#[allow(unused)]
pub(crate) async fn list_message_handler() -> impl IntoResponse {
    "list_message"
}
