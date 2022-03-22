use crate::error::AppError;
use crate::handler::backend::get_logined_admin;
use crate::model::AppState;
use axum::async_trait;
use axum::extract::{FromRequest, RequestParts};
use std::sync::Arc;

pub struct Auth {}

#[async_trait]
impl<B> FromRequest<B> for Auth
where
    B: Send,
{
    type Rejection = AppError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let state = req.extensions().unwrap().get::<Arc<AppState>>().unwrap();
        let headers = req.headers().unwrap();
        let admin_session = get_logined_admin(&state, headers).await?;
        if let Some(_) = admin_session {
            Ok(Auth {})
        } else {
            Err(AppError::auth_error("UNAUTHENTICATED"))
        }
    }
}
