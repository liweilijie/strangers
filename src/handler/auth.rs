use crate::db::admin;
use crate::error::AppError;
use crate::handler::backend::get_login_admin_by_cookie;
use crate::handler::helper::{get_client, log_error, render};
use crate::handler::redirect::{redirect, redirect_with_cookie};
use crate::html::auth::LoginTemplate;
use crate::model::{AdminSession, AppState};
use crate::session::gen_redis_key;
use crate::{form, hcaptcha, password, rdb, session, Result};
use axum::extract::{Extension, Form};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Html;
use serde_json::json;
use std::ops::Add;
use std::sync::Arc;
use tower_cookies::{Cookie, Cookies};
use tracing::{debug, error};

pub async fn admin_login_ui(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ck): Extension<Cookies>,
) -> Result<Html<String>> {
    let handler_name = "admin_login_ui";
    let site_key = state.hcap_cfg.site_key.clone();
    let tmpl = LoginTemplate { site_key };
    render(tmpl, handler_name)
}

pub async fn admin_login(
    Extension(state): Extension<Arc<AppState>>,
    Form(login): Form<form::AdminLogin>,
    Extension(ck): Extension<Cookies>,
) -> Result<(StatusCode, HeaderMap, ())> {
    // 查看是否已经登录,如果已经登录则直接跳转到管理页面
    if let Ok(admin_session) = get_login_admin_by_cookie(&state, &ck).await {
        if admin_session.is_some() {
            debug!("已经登录,直接进管理页面.");
            return redirect("/admin");
        }
    }

    let handler_name = "auth_login";
    // 不用去验证图片验证码
    // let is_valid = hcaptcha::verify(
    //     login.hcaptcha_response.clone(),
    //     state.hcap_cfg.secret_key.clone(),
    // )
    // .await?;
    // if !is_valid {
    //     debug!("hcaptcha verification failed");
    //     // return Err(AppError::auth_error("人机验证失败"));
    // }
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
    // data: "{\"dateline\":1647955504,\"id\":1,\"is_sys\":true,\"password\":\"$2b$12$QW8Lmf0gvsb1xtRJLxJxzea2M2p5Pxx1LrmPuVzria5obcY8u890C\",\"username\":\"wgr\"}"
    let session::GeneratedKey {
        id,
        cookie_key,
        redis_key,
    } = session::gen_key(&cfg);
    debug!(
        "id: {:?}, cookie_key: {:?}, redis_key: {:?}",
        id, cookie_key, redis_key
    );
    // id: "4d8e927e7dc44ef696351ec59438f5ee", cookie_key: "axumrs_session", redis_key: "axumrs:session:4d8e927e7dc44ef696351ec59438f5ee"
    rdb::set(&state.rdc, &redis_key, &data, cfg.expired).await?;
    // .map_err(AppError::form)?; // TODO:
    let cookie = format!("{}={}", cookie_key, id);
    debug!("to set cookie: {:?}", cookie);
    // let cookie_item = Cookie::new(cookie_key, id);
    redirect_with_cookie("/admin", Some(&cookie))
}

pub async fn admin_logout(
    Extension(state): Extension<Arc<AppState>>,
    Extension(ck): Extension<Cookies>,
) -> Result<(StatusCode, HeaderMap, ())> {
    let cfg = state.sess_cfg.clone();
    let cookie: Option<String> = ck
        .get(&cfg.id_name)
        .and_then(|c| Some(c.value().to_owned()));

    if let Some(val) = cookie {
        let redis_key = gen_redis_key(&cfg, &val);
        debug!("logout delete redis_key: {:?}", redis_key);
        rdb::del(&state.rdc, &redis_key).await.map_err(|e| {
            error!("logout delete redis_key: {:?} failed: {:?}", redis_key, e);
        });
    }
    let cookie_logout = format!("{}=", &cfg.id_name);
    redirect_with_cookie("/login", Some(&cookie_logout))
}
