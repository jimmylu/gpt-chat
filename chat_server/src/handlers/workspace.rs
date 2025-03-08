use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};

use crate::{models::Workspace, AppError, AppState};

pub(crate) async fn user_list_handler(
    State(state): State<AppState>,
    Path(ws_name): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let workspace = Workspace::get_by_name(&ws_name, &state.pool).await?;
    if workspace.is_none() {
        return Err(AppError::WorkspaceNotFound);
    }
    let users = workspace.unwrap().fetch_all_chat_users(&state.pool).await?;
    Ok(Json(users))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::CreateUserPayload, AppError, AppState, User};
    use axum::{extract::Path, http::StatusCode, response::IntoResponse};
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn user_list_handler_should_work() -> Result<(), AppError> {
        let (pg, _state) = AppState::new_for_test().await?;
        let pg_pool = pg.get_pool().await;

        let workspace = Workspace::create("my_ws", &pg_pool).await?;
        let _user = User::create(
            CreateUserPayload {
                email: "test@test.com".to_string(),
                fullname: "test".to_string(),
                password: "test".to_string(),
                workspace: workspace.name.clone(),
            },
            &pg_pool,
        )
        .await?;

        let res = user_list_handler(State(_state), Path("my_ws".to_string()))
            .await?
            .into_response();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap();
        let users = serde_json::from_slice::<Vec<User>>(&body.to_bytes()).unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].email, "test@test.com");
        Ok(())
    }
}
