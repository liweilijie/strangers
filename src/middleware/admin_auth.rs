use crate::error::AppError;
use crate::handler::backend::get_login_admin_by_cookie;
use crate::model::AppState;
use axum::async_trait;
use axum::extract::{FromRequest, RequestParts};
use std::sync::Arc;
use tower_cookies::Cookies;
use tracing::debug;

pub struct Auth {}

#[async_trait]
impl<B> FromRequest<B> for Auth
where
    B: Send,
{
    type Rejection = AppError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let state = req.extensions().unwrap().get::<Arc<AppState>>().unwrap();
        let ck = req.extensions().unwrap().get::<Cookies>().unwrap();
        // let headers = req.headers().unwrap();
        let admin_session = get_login_admin_by_cookie(&state, ck).await?;
        if let Some(_) = admin_session {
            Ok(Auth {})
        } else {
            Err(AppError::auth_error(
                "权限认证失败，请点击右上角重新登录页面!",
            ))
        }
    }
}
