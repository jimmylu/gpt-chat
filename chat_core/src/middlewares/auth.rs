use axum::{
    extract::{FromRequestParts, Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use serde::Deserialize;
use tracing::warn;

use super::TokenVerify;

#[derive(Debug, Deserialize)]
struct Params {
    pub token: String,
}

pub async fn verify_token<T>(State(state): State<T>, req: Request, next: Next) -> Response
where
    T: TokenVerify + Clone + Send + Sync + 'static,
{
    let (mut parts, body) = req.into_parts();

    let token =
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
            Ok(TypedHeader(Authorization(bearer))) => bearer.token().to_string(),
            Err(e) => {
                if e.is_missing() {
                    match Query::<Params>::from_request_parts(&mut parts, &state).await {
                        Ok(params) => params.token.clone(),
                        Err(e) => {
                            let msg = format!("parse authorization param failed: {}", e);
                            warn!("{}", msg);
                            return (StatusCode::UNAUTHORIZED, msg).into_response();
                        }
                    }
                } else {
                    let msg = format!("parse authorization header failed: {}", e);
                    warn!("{}", msg);
                    return (StatusCode::UNAUTHORIZED, msg).into_response();
                }
            }
        };
    dbg!(format!("auth token={:?}", &token));

    let Ok(user) = state.verify(&token) else {
        let msg = format!("token verification failed: {}", token);
        warn!("{}", msg);
        return (StatusCode::FORBIDDEN, msg).into_response();
    };

    dbg!(format!("auth user:{:?}", user.clone()));

    let mut req = Request::from_parts(parts, body);
    req.extensions_mut().insert(user);

    next.run(req).await
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use super::*;
    use crate::{DecodingKey, EncodingKey, User};
    use anyhow::Result;
    use axum::{
        Extension, Router, body::Body, http::HeaderValue, middleware::from_fn_with_state,
        routing::get,
    };
    use http_body_util::BodyExt as _;
    use tower::ServiceExt;

    #[derive(Clone)]
    struct AppState(Arc<AppStateInner>);

    struct AppStateInner {
        sk: EncodingKey,
        pk: DecodingKey,
    }

    impl TokenVerify for AppState {
        type Error = ();
        fn verify(&self, token: &str) -> Result<User, Self::Error> {
            self.0.pk.verify(token).map_err(|_| ())
        }
    }

    async fn handler(Extension(user): Extension<User>, _req: Request) -> impl IntoResponse {
        (StatusCode::OK, user.email).into_response()
    }

    #[tokio::test]
    async fn verify_token_none_bearer_header_should_return_unauthorized() -> Result<()> {
        let state = AppState(Arc::new(AppStateInner {
            sk: EncodingKey::load(include_str!("../../fixtures/encoding.pem"))?,
            pk: DecodingKey::load(include_str!("../../fixtures/decoding.pem"))?,
        }));
        let app = Router::new().route(
            "/api",
            get(handler)
                .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
                .with_state(state),
        );

        let req = Request::builder().uri("/api").body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        Ok(())
    }

    #[tokio::test]
    async fn verify_token_invalid_bearer_header_should_return_forbidden() -> Result<()> {
        let state = AppState(Arc::new(AppStateInner {
            sk: EncodingKey::load(include_str!("../../fixtures/encoding.pem"))?,
            pk: DecodingKey::load(include_str!("../../fixtures/decoding.pem"))?,
        }));
        let app = Router::new().route(
            "/api",
            get(handler)
                .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
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
        let state = AppState(Arc::new(AppStateInner {
            sk: EncodingKey::load(include_str!("../../fixtures/encoding.pem"))?,
            pk: DecodingKey::load(include_str!("../../fixtures/decoding.pem"))?,
        }));

        let user = User::new(1, 1, "test".to_string(), "test@test.com".to_string());
        let token = state.0.sk.sign(user)?;

        let app = Router::new().route(
            "/api",
            get(handler)
                .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
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
