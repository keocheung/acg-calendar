#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
use axum::{
    body::Body,
    extract::OriginalUri,
    http::{header, HeaderValue, Response, StatusCode},
    routing::get,
    Router,
};
#[cfg(not(target_arch = "wasm32"))]
use calendar::{handle_calendar_url, AppError};
#[cfg(not(target_arch = "wasm32"))]
use std::{env, net::SocketAddr};
#[cfg(not(target_arch = "wasm32"))]
use url::Url;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    let addr = env::var("BIND")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_owned())
        .parse::<SocketAddr>()
        .expect("BIND must be a socket address");
    eprintln!("[calendar] listening on http://{addr}");
    let app = Router::new().route("/{*path}", get(handle_calendar));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");

    axum::serve(listener, app).await.expect("server failed");
}

#[cfg(not(target_arch = "wasm32"))]
async fn handle_calendar(OriginalUri(uri): OriginalUri) -> Response<Body> {
    let url = match Url::parse(&format!("http://localhost{uri}")) {
        Ok(url) => url,
        Err(err) => {
            return text_response(
                StatusCode::BAD_REQUEST,
                &format!("invalid request URI: {err}"),
            )
        }
    };
    match handle_calendar_url(&url).await {
        Ok(Some(calendar)) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, calendar.content_type)
            .header(header::CACHE_CONTROL, "public, max-age=1800")
            .body(Body::from(calendar.body))
            .expect("response should build"),
        Ok(None) => text_response(StatusCode::NOT_FOUND, "not found"),
        Err(err) => error_response(err),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn error_response(err: AppError) -> Response<Body> {
    let status =
        StatusCode::from_u16(err.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    text_response(status, &err.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn text_response(status: StatusCode, body: &str) -> Response<Body> {
    let mut response = Response::new(Body::from(body.to_owned()));
    *response.status_mut() = status;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    response
}
