pub mod index;
pub mod medicinal;

use crate::error::AppError;
use crate::handler::helper::get_cookie;
use crate::model::{AdminSession, AppState};
use crate::session::gen_redis_key;
use crate::{rdb, Result};
use axum::http::HeaderMap;
use axum::routing::get;
use axum::Router;
use tracing::{debug, error};

pub fn routers() -> Router {
    Router::new()
        .route("/", get(index::index))
        .route("/medicinal", get(medicinal::index))
        .route(
            "/medicinal/add",
            get(medicinal::add).post(medicinal::add_action),
        )
        .route(
            "/medicinal/upload",
            get(medicinal::upload).post(medicinal::upload_action),
        )
}

pub async fn get_logined_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<AdminSession>> {
    let sess_cfg = state.clone().sess_cfg.clone();
    debug!("sess_cfg: {:#?}", sess_cfg);
    let cookie = get_cookie(headers, &sess_cfg.id_name);
    debug!("result cookie: {:?}", cookie);
    if let Some(session_id) = cookie {
        if !session_id.is_empty() {
            let redis_key = gen_redis_key(&sess_cfg, &session_id);
            debug!("redis_key: {}", redis_key);
            let admin_session = rdb::get(&state.rdc, &redis_key).await.map_err(|err| {
                error!("get session failed: {:?}", err);
                AppError::auth_error("UNAUTHENTICATED")
            })?;
            debug!("admin_session: {:#?}", admin_session);
            if let Some(admin_session) = admin_session {
                let admin_session: AdminSession =
                    serde_json::from_str(&admin_session).map_err(|err| {
                        error!("des parse session failed: {:?}", err);
                        AppError::auth_error("UNAUTHENTICATED")
                    })?;
                debug!(
                    "deserialize success and admin_session: {:#?}",
                    admin_session
                );
                return Ok(Some(admin_session));
            }
        }
    }
    Ok(None)
}
