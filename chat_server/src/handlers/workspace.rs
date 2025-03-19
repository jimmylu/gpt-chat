use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};

use crate::{AppError, AppState};

pub(crate) async fn user_list_handler(
    State(state): State<AppState>,
    Path(ws_name): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let workspace = state.workspace_fetch_by_name(&ws_name).await?;
    if workspace.is_none() {
        return Err(AppError::WorkspaceNotFound);
    }
    let users = state
        .user_fetched_all_by_ws_id(workspace.unwrap().id as u64)
        .await?;
    Ok(Json(users))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CreateUserPayload;
    use crate::{AppError, AppState, ChatUser};
    use axum::{extract::Path, http::StatusCode, response::IntoResponse};
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn user_list_handler_should_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let workspace = state.workspace_create("my_ws").await?;
        let _user = state
            .user_create(CreateUserPayload {
                email: "test@test.com".to_string(),
                fullname: "test".to_string(),
                password: "test".to_string(),
                workspace: workspace.name.clone(),
            })
            .await?;

        let res = user_list_handler(State(state), Path("my_ws".to_string()))
            .await?
            .into_response();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap();
        let users = serde_json::from_slice::<Vec<ChatUser>>(&body.to_bytes()).unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].email, "test@test.com");
        Ok(())
    }
}
