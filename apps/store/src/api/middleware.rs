//! API Middleware
//!
//! CORS and other middleware for HTTP requests.

use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, Method},
    middleware::Next,
    response::Response,
};
use crate::api::AppState;

pub fn cors_middleware() -> impl Clone + Send + Sync + 'static + tower_service::Service<axum::extract::Request> {
    use tower_service::Service;
    use axum::body::Body;

    axum::middleware::from_fn(
        |State(state): State<AppState>,
         request: axum::extract::Request,
         next: Next| async move {
            let method = request.method().clone();
            let headers = request.headers().clone();

            let mut response = next.run(request).await;

            if state.cors_enabled {
                if let Some(origin) = &state.cors_origin {
                    let headers = response.headers_mut();

                    let origin_value = HeaderValue::from_str(origin)
                        .expect("invariant violation: CORS origin must be a valid header value; this indicates a bug in configuration validation");

                    headers.insert(
                        "access-control-allow-origin",
                        origin_value,
                    );
                    headers.insert(
                        "access-control-allow-methods",
                        HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
                    );
                    headers.insert(
                        "access-control-allow-headers",
                        HeaderValue::from_static("Content-Type, Authorization"),
                    );
                    headers.insert(
                        "access-control-max-age",
                        HeaderValue::from_static("86400"),
                    );
                }
            }

            response
        }
    )
}
