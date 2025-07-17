use axum::{
    extract::Request,
    http::{header, StatusCode},
    response::Response,
};

use crate::auth::jwt::decode_token;

static AUTH_COOKIE: &str = "token";

pub(crate) static REMOVE_COOKIE: &str = "token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT";

pub async fn auth_middleware(
    req: Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Response {
    match get_username_from_headers(req.headers()) {
        Some(username) => {
            let Ok(_) = crate::models::User::get(username).await else {
                tracing::info!("no user associated with this token");
                return redirect(req, next).await;
            };

            let path = req.uri().path();
            if path.starts_with("/login") || path.starts_with("/signup") {
                // If the user is authenticated, we don't want to show the login or signup pages
                return Response::builder()
                    .status(StatusCode::FOUND)
                    .header(header::LOCATION, "/")
                    .body(axum::body::Body::empty())
                    .unwrap();
            }
            next.run(req).await
        }
        None => redirect(req, next).await,
    }
}

async fn redirect(req: Request<axum::body::Body>, next: axum::middleware::Next) -> Response {
    let path = req.uri().path();

    if path.starts_with("/settings") || path.starts_with("/find") {
        // authenticated routes
        Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/login")
            .header(header::SET_COOKIE, REMOVE_COOKIE)
            .body(axum::body::Body::empty())
            .unwrap()
    } else {
        next.run(req).await
    }
}

#[tracing::instrument]
pub fn get_username_from_headers(headers: &axum::http::HeaderMap) -> Option<String> {
    headers.get(header::COOKIE).and_then(|x| {
        x.to_str()
            .unwrap()
            .split("; ")
            .find(|&x| x.starts_with(AUTH_COOKIE))
            .and_then(|x| x.split('=').last())
            .and_then(|x| decode_token(x).map(|jwt| jwt.claims.sub).ok())
    })
}

#[tracing::instrument]
pub fn get_username_from_request(request: &Request) -> Option<String> {
    get_username_from_headers(request.headers())
}