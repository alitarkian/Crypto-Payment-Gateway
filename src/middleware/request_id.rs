use axum::{ extract::Request, middleware::Next, response::Response };
use uuid::Uuid;

pub const REQUEST_ID_HEADER: &str = "x-request-id";

pub async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    let request_id = req
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    tracing::Span::current().record("request_id", &request_id);

    req.headers_mut().insert(REQUEST_ID_HEADER, request_id.parse().unwrap());

    let mut response = next.run(req).await;

    response.headers_mut().insert(REQUEST_ID_HEADER, request_id.parse().unwrap());

    response
}
