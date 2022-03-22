use axum::http::header::{LOCATION, SET_COOKIE};
use axum::http::{HeaderMap, StatusCode};

/// 重定向
pub fn redirect(url: &str) -> crate::Result<(StatusCode, HeaderMap, ())> {
    redirect_with_cookie(url, None)
}

/// 重定向
pub fn redirect_with_cookie(
    url: &str,
    cookie: Option<&str>,
) -> crate::Result<(StatusCode, HeaderMap, ())> {
    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, url.parse().unwrap());
    if let Some(cookie) = cookie {
        headers.insert(SET_COOKIE, cookie.parse().unwrap());
    }
    Ok((StatusCode::FOUND, headers, ()))
}
