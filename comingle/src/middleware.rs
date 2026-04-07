use axum::body::Body;
use axum::http::Request;
use axum::http::header::CACHE_CONTROL;
use axum::middleware::Next;
use axum::response::IntoResponse;

pub async fn cache_short(req: Request<Body>, next: Next) -> impl IntoResponse {
    let mut res = next.run(req).await;
    res.headers_mut()
        .insert(CACHE_CONTROL, "public, max-age=24".parse().unwrap());
    res
}

pub async fn cache_forever(req: Request<Body>, next: Next) -> impl IntoResponse {
    let mut res = next.run(req).await;
    res.headers_mut().insert(
        CACHE_CONTROL,
        "public, max-age=31536000, immutable".parse().unwrap(), // 1 year
    );
    res
}
