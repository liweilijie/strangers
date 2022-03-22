use crate::db::admin;
use crate::error::AppError;
use crate::handler::helper::{get_client, log_error, render};
use crate::handler::redirect::redirect_with_cookie;
use crate::html::auth::LoginTemplate;
use crate::model::{AdminSession, AppState};
use crate::{form, hcaptcha, password, rdb, session, Result};
use axum::extract::{Extension, Form};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Html;
use serde_json::json;
use std::ops::Add;
use std::sync::Arc;
use tracing::debug;

pub async fn admin_login_ui(Extension(state): Extension<Arc<AppState>>) -> Result<Html<String>> {
    let handler_name = "admin_login_ui";
    let site_key = state.hcap_cfg.site_key.clone();
    let tmpl = LoginTemplate { site_key };
    render(tmpl, handler_name)
}

pub async fn admin_login(
    Extension(state): Extension<Arc<AppState>>,
    Form(login): Form<form::AdminLogin>,
) -> Result<(StatusCode, HeaderMap, ())> {
    let handler_name = "auth_login";
    let is_valid = hcaptcha::verify(
        login.hcaptcha_response.clone(),
        state.hcap_cfg.secret_key.clone(),
    )
    .await?;
    if !is_valid {
        return Err(AppError::auth_error("人机验证失败"));
    }
    let client = get_client(&state, handler_name).await?;
    let login_admin = admin::find(&client, &login.username)
        .await
        .map_err(log_error(handler_name.to_string()))?;
    debug!("passwd: {:?}", password::hash(&login.password));
    if !password::verify(&login.password, &login_admin.password)? {
        return Err(AppError::auth_error("用户名或密码错误"));
    }
    let cfg = state.sess_cfg.clone();
    let dateline = chrono::Local::now().add(chrono::Duration::seconds(cfg.expired as i64));
    debug!("dateline: {:?}", dateline);
    let data = json!(AdminSession {
        id: login_admin.id,
        username: login_admin.username,
        dateline: dateline.timestamp() as i32,
        password: login_admin.password,
        is_sys: login_admin.is_sys,
    });
    let data = data.to_string();
    debug!("data: {:?}", data);
    let session::GeneratedKey {
        id,
        cookie_key,
        redis_key,
    } = session::gen_key(&cfg);
    debug!(
        "id: {:?}, cookie_key: {:?}, redis_key: {:?}",
        id, cookie_key, redis_key
    );
    rdb::set(&state.rdc, &redis_key, &data, cfg.expired).await?;
    // .map_err(AppError::form)?; // TODO:
    let cookie = format!("{}={}", cookie_key, id);
    redirect_with_cookie("/admin", Some(&cookie))
}
