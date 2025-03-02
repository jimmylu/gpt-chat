use axum::{
    extract::{FromRequestParts, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use tracing::warn;

use crate::AppState;

pub async fn verify_token(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let (mut parts, body) = req.into_parts();

    let req =
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
            Ok(TypedHeader(Authorization(bearer))) => {
                let token = bearer.token();
                let Ok(user) = state.pk.verify(token) else {
                    let msg = format!("token verification failed: {}", token);
                    warn!("{}", msg);
                    return (StatusCode::FORBIDDEN, msg).into_response();
                };

                let mut req = Request::from_parts(parts, body);
                req.extensions_mut().insert(user);
                req
            }
            Err(e) => {
                let msg = format!("parse authorization header failed: {}", e);
                warn!("{}", msg);
                return (StatusCode::UNAUTHORIZED, msg).into_response();
            }
        };

    next.run(req).await
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{AppState, User};
    use anyhow::Result;
    use axum::{
        body::Body, http::HeaderValue, middleware::from_fn_with_state, routing::get, Extension,
        Router,
    };
    use http_body_util::BodyExt as _;
    use tower::ServiceExt;

    async fn handler(Extension(user): Extension<User>, _req: Request) -> impl IntoResponse {
        (StatusCode::OK, user.email).into_response()
    }

    #[tokio::test]
    async fn verify_token_none_bearer_header_should_return_unauthorized() -> Result<()> {
        let (_pg, state) = AppState::new_for_test().await?;
        let app = Router::new().route(
            "/api",
            get(handler)
                .layer(from_fn_with_state(state.clone(), verify_token))
                .with_state(state),
        );

        let req = Request::builder().uri("/api").body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        Ok(())
    }

    #[tokio::test]
    async fn verify_token_invalid_bearer_header_should_return_forbidden() -> Result<()> {
        let (_pg, state) = AppState::new_for_test().await?;
        let app = Router::new().route(
            "/api",
            get(handler)
                .layer(from_fn_with_state(state.clone(), verify_token))
                .with_state(state),
        );

        let mut req = Request::builder().uri("/api").body(Body::empty())?;
        req.headers_mut()
            .insert("Authorization", HeaderValue::from_str("Bearer invalid")?);
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::FORBIDDEN);

        Ok(())
    }

    #[tokio::test]
    async fn verify_token_signed_should_work() -> Result<()> {
        let (_pg, state) = AppState::new_for_test().await?;

        let user = User::new(1, "test".to_string(), "test@test.com".to_string());
        let token = state.sk.sign(user)?;

        let app = Router::new().route(
            "/api",
            get(handler)
                .layer(from_fn_with_state(state.clone(), verify_token))
                .with_state(state),
        );

        let mut req = Request::builder().uri("/api").body(Body::empty())?;
        req.headers_mut().insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", token))?,
        );
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::OK);
        let body = String::from_utf8(res.into_body().collect().await?.to_bytes().to_vec())?;
        assert_eq!(body, "test@test.com");

        Ok(())
    }
}
