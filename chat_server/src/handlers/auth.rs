use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::models::{CreateUserPayload, SignInPayload};
use crate::{AppError, AppState};

pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.user_create(payload.clone()).await?;

    let token = state.sk.sign(user)?;

    Ok((
        StatusCode::CREATED,
        [(header::AUTHORIZATION, format!("Bearer {}", token))],
        (),
    ))
}

/// Sign in handler
pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(payload): Json<SignInPayload>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.user_verify(&payload.email, &payload.password).await?;

    match user {
        Some(user) => {
            let token = state.sk.sign(user)?;
            Ok((
                StatusCode::CREATED,
                [(header::AUTHORIZATION, format!("Bearer {}", token))],
                "",
            ))
        }
        None => Err(AppError::NotFound(format!(
            "User with email {} not found",
            payload.email
        ))),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn signin_handler_should_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let create_user = CreateUserPayload {
            email: "test@test.com".to_string(),
            fullname: "test".to_string(),
            password: "test".to_string(),
            workspace: "Default".to_string(),
        };
        state.user_create(create_user).await?;

        let user = SignInPayload {
            email: "test@test.com".to_string(),
            password: "test".to_string(),
        };
        let res = signin_handler(State(state), Json(user))
            .await?
            .into_response();
        assert_eq!(res.status(), axum::http::StatusCode::CREATED);
        assert!(res.headers().get(header::AUTHORIZATION).is_some());
        Ok(())
    }
    #[tokio::test]
    async fn signin_handler_should_fail_when_user_not_found() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let create_user = CreateUserPayload {
            email: "test@test.com".to_string(),
            fullname: "test".to_string(),
            password: "test".to_string(),
            workspace: "Default".to_string(),
        };
        state.user_create(create_user).await?;

        let user = SignInPayload {
            email: "test1@test.com".to_string(),
            password: "test".to_string(),
        };
        let res = signin_handler(State(state), Json(user)).await;
        assert!(matches!(
            res,
            Err(AppError::NotFound(msg)) if msg == "User with email test1@test.com not found"
        ));
        Ok(())
    }

    #[tokio::test]
    async fn signin_handler_should_fail_when_password_is_incorrect() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let create_user = CreateUserPayload {
            email: "test@test.com".to_string(),
            fullname: "test".to_string(),
            password: "test".to_string(),
            workspace: "Default".to_string(),
        };
        state.user_create(create_user).await?;

        let user = SignInPayload {
            email: "test@test.com".to_string(),
            password: "test1".to_string(),
        };
        let res = signin_handler(State(state), Json(user)).await;
        matches!(res, Err(AppError::InvalidCredentials));

        // assert_eq!(
        //     res.headers().get(header::AUTHORIZATION).unwrap(),
        //     "None"
        // );
        Ok(())
    }

    #[tokio::test]
    async fn signup_handler_should_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let user = CreateUserPayload {
            email: "test@test.com".to_string(),
            fullname: "test".to_string(),
            password: "test".to_string(),
            workspace: "Default".to_string(),
        };
        let res = signup_handler(State(state), Json(user.clone()))
            .await?
            .into_response();
        assert_eq!(res.status(), axum::http::StatusCode::CREATED);
        assert!(res.headers().get(header::AUTHORIZATION).is_some());

        Ok(())
    }

    #[tokio::test]
    async fn signup_handler_should_fail_when_email_already_exists() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let user = CreateUserPayload {
            email: "test@test.com".to_string(),
            fullname: "test".to_string(),
            password: "test".to_string(),
            workspace: "Default".to_string(),
        };
        state.user_create(user.clone()).await?;

        let res = signup_handler(State(state), Json(user)).await;
        matches!(res, Err(AppError::UserAlreadyExists));
        Ok(())
    }
}
