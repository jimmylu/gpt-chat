use axum::http::HeaderValue;
use axum::{extract::Request, middleware::Next, response::Response};
use uuid::Uuid;

use super::REQUEST_ID_HEADER;

#[allow(unused)]
pub async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    // if x-request-id exists, do nothing, otherwise generate a new one
    let request_id = match req.headers().get(REQUEST_ID_HEADER) {
        Some(v) => Some(v.clone()),
        None => {
            // 使用 v7 UUID，它包含时间戳信息，便于请求追踪和排序
            let request_id = Uuid::now_v7().to_string();
            let header_value = HeaderValue::from_str(&request_id).unwrap();
            req.headers_mut()
                .insert(REQUEST_ID_HEADER, header_value.clone());
            Some(header_value)
        }
    };

    let mut res = next.run(req).await;

    let Some(request_id) = request_id else {
        return res;
    };

    res.headers_mut()
        .insert(REQUEST_ID_HEADER, request_id.clone());

    res
}
