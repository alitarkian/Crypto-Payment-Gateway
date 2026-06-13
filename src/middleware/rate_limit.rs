use axum::{ extract::Request, middleware::Next, response::Response };
use tracing::debug;

#[allow(dead_code)]
pub async fn ip_rate_limit(req: Request, next: Next) -> Response {
    debug!(
        method = %req.method(),
        uri = %req.uri(),
        "Rate limit check"
    );
    next.run(req).await
}
