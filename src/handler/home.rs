use crate::handler::backend::get_login_admin_by_cookie;
use crate::handler::redirect::redirect_with_cookie;
use crate::model::{AdminSession, AppState};
use crate::session::gen_redis_key;
use crate::{rdb, Result};
use axum::extract::Extension;
use axum::http::{HeaderMap, StatusCode};
use std::sync::Arc;
use tower_cookies::Cookies;
use tracing::debug;

// 首页如果已经login，则跳转到admin页面
// 如果没有login，则跳转到login页面
pub async fn admin_index(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ck): Extension<Cookies>,
) -> Result<(StatusCode, HeaderMap, ())> {
    let admin_session = get_login_admin_by_cookie(&state, &ck).await?;
    if admin_session.is_none() {
        redirect_with_cookie("/login", None)
    } else {
        redirect_with_cookie("/admin", None)
    }
    //
    // debug!("cookie: {:?}", cookie_kv);
    // if cookie_kv.is_some() && !cookie_kv.as_ref().unwrap().value.is_empty() {
    //     let redis_key = gen_redis_key(&cfg, &cookie_kv.as_ref().unwrap().value);
    //     debug!("redis_key: {:?}", redis_key);
    //     let data = rdb::get(&state.rdc, &redis_key).await?;
    //     if let Some(data) = data {
    //         let admin_session: AdminSession = serde_json::from_str(&data)?;
    //         if admin_session.is_sys {
    //             return redirect_with_cookie("/admin", None);
    //         }
    //     }
    // }
    // redirect_with_cookie("/login", None)
}
