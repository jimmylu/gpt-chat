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
