use axum::{
    extract::{FromRequestParts, Path, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{AppState, User};

#[allow(unused)]
pub async fn verify_chat(State(state): State<AppState>, mut req: Request, next: Next) -> Response {
    let (mut parts, body) = req.into_parts();
    let Path(chat_id) = Path::<u64>::from_request_parts(&mut parts, &state)
        .await
        .unwrap();
    // chat is exists
    let chat = state.chat_fetched_by_id(chat_id as _).await;
    match chat {
        Ok(Some(_)) => (),
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                format!("Chat not found with id: {}", chat_id),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal server error: {}", e),
            )
                .into_response();
        }
    }

    let user = parts.extensions.get::<User>().unwrap();
    // verify the user is a member of the chat with the id in the path
    let is_member = state.chat_is_member(chat_id, user.id as _).await;

    match is_member {
        Ok(true) => (),
        Ok(false) => {
            return (StatusCode::FORBIDDEN, "You are not a member of this chat").into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal server error: {}", e),
            )
                .into_response();
        }
    }

    let req = Request::from_parts(parts, body);
    next.run(req).await
}

#[cfg(test)]
mod tests {

    use anyhow::Result;
    use axum::{body::Body, middleware::from_fn_with_state, routing::get, Extension, Router};
    use chrono::Utc;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;
    async fn handler(
        State(_state): State<AppState>,
        Extension(_user): Extension<User>,
        Path(_chat_id): Path<u64>,
        _req: Request,
    ) -> impl IntoResponse {
        (StatusCode::OK, "").into_response()
    }

    #[tokio::test]
    async fn verify_chat_member_should_work() -> Result<()> {
        let (_pg, state) = AppState::new_for_test().await?;

        let user = User {
            id: 1,
            email: "test@test.com".to_string(),
            ws_id: 1,
            fullname: "test".to_string(),
            password_hash: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let app = Router::new().route(
            "/api/{chat_id}",
            get(handler)
                .layer(from_fn_with_state(state.clone(), verify_chat))
                .with_state(state),
        );

        let mut req = Request::builder().uri("/api/1").body(Body::empty())?;
        // 为handler添加user
        req.extensions_mut().insert(user);
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::OK);

        Ok(())
    }

    #[tokio::test]
    async fn verify_chat_member_should_return_403_if_user_is_not_a_member() -> Result<()> {
        let (_pg, state) = AppState::new_for_test().await?;
        let app = Router::new().route(
            "/api/{chat_id}",
            get(handler)
                .layer(from_fn_with_state(state.clone(), verify_chat))
                .with_state(state),
        );

        let user = User {
            id: 5,
            email: "test@test.com".to_string(),
            ws_id: 1,
            fullname: "test".to_string(),
            password_hash: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut req = Request::builder().uri("/api/1").body(Body::empty())?;
        // 为handler添加user
        req.extensions_mut().insert(user);
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
        let body = String::from_utf8(res.into_body().collect().await?.to_bytes().to_vec())?;
        assert_eq!(body, "You are not a member of this chat");

        Ok(())
    }

    #[tokio::test]
    async fn verify_chat_member_should_return_404_if_chat_not_found() -> Result<()> {
        let (_pg, state) = AppState::new_for_test().await?;
        let app = Router::new().route(
            "/api/{chat_id}",
            get(handler)
                .layer(from_fn_with_state(state.clone(), verify_chat))
                .with_state(state),
        );

        let user = User {
            id: 1,
            email: "test@test.com".to_string(),
            ws_id: 1,
            fullname: "test".to_string(),
            password_hash: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut req = Request::builder().uri("/api/20").body(Body::empty())?;
        // 为handler添加user
        req.extensions_mut().insert(user);
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        let body = String::from_utf8(res.into_body().collect().await?.to_bytes().to_vec())?;
        assert_eq!(body, "Chat not found with id: 20");

        Ok(())
    }
}
