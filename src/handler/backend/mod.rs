pub mod admin;
pub mod excel;
pub mod index;
pub mod medicinal;

use crate::error::AppError;
use crate::model::{AdminSession, AppState};
use crate::session::gen_redis_key;
use crate::{rdb, Result};
use axum::http::HeaderMap;
use axum::routing::get;
use axum::Router;
use tower_cookies::Cookies;
use tracing::{debug, error};

pub fn routers() -> Router {
    Router::new()
        .route("/", get(index::index))
        .route("/medicinal", get(medicinal::index))
        .route(
            "/medicinal/add",
            get(medicinal::add).post(medicinal::add_action),
        ) // 添加药品
        .route(
            "/medicinal/edit/:id",
            get(medicinal::edit).post(medicinal::edit_action),
        ) // 编辑药品
        .route(
            "/medicinal/upload",
            get(medicinal::upload).post(medicinal::upload_action),
        ) // 批量上传药品
        .route("/medicinal/download", get(medicinal::download))
        .route("/medicinal/del/:id", get(medicinal::del)) // 删除药品
        .route("/medicinal/recover/:id", get(medicinal::recover)) // 恢复药品
        .route("/admin", get(admin::index))
        .route("/admin/add", get(admin::add).post(admin::add_action)) // 添加管理员
        .route("/admin/edit/:id", get(admin::edit).post(admin::edit_action)) // 编辑管理员
        .route("/admin/del/:id", get(admin::del)) // 删除管理员
        .route("/admin/recover/:id", get(admin::recover)) // 恢复管理员
}

// pub async fn get_logined_admin(
//     state: &AppState,
//     headers: &HeaderMap,
// ) -> Result<Option<AdminSession>> {
//     let sess_cfg = state.clone().sess_cfg.clone();
//     debug!("sess_cfg: {:#?}", sess_cfg);
//     let cookie = get_cookie(headers, &sess_cfg.id_name);
//     debug!("result cookie: {:?}", cookie);
//     if let Some(session_id) = cookie {
//         if !session_id.is_empty() {
//             let redis_key = gen_redis_key(&sess_cfg, &session_id);
//             debug!("redis_key: {}", redis_key);
//             let admin_session = rdb::get(&state.rdc, &redis_key).await.map_err(|err| {
//                 error!("get session failed: {:?}", err);
//                 AppError::auth_error("UNAUTHENTICATED")
//             })?;
//             debug!("admin_session: {:#?}", admin_session);
//             if let Some(admin_session) = admin_session {
//                 let admin_session: AdminSession =
//                     serde_json::from_str(&admin_session).map_err(|err| {
//                         error!("des parse session failed: {:?}", err);
//                         AppError::auth_error("UNAUTHENTICATED")
//                     })?;
//                 debug!(
//                     "deserialize success and admin_session: {:#?}",
//                     admin_session
//                 );
//                 return Ok(Some(admin_session));
//             }
//         }
//     }
//     Ok(None)
// }

pub async fn get_login_admin_by_cookie(
    state: &AppState,
    ck: &Cookies,
) -> Result<Option<AdminSession>> {
    let sess_cfg = state.clone().sess_cfg.clone();
    debug!("sess_cfg: {:#?}", sess_cfg);
    let cookie = ck
        .get(&sess_cfg.id_name)
        .and_then(|c| Some(c.value().to_owned()));
    debug!("result cookie: {:#?}", cookie);
    if let Some(cookie) = cookie {
        let redis_key = gen_redis_key(&sess_cfg, &cookie);
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

    Ok(None)
}
