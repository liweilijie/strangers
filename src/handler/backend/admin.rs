use crate::db::admin;
use crate::error::{AppError, AppErrorType};
use crate::form::{CreateAdmin, UpdateAdmin};
use crate::handler::backend::get_login_admin_by_cookie;
use crate::handler::helper::{get_client, log_error, render};
use crate::handler::redirect::redirect;
use crate::html::backend::admin::{AddTemplate, EditTemplate, IndexTemplate};
use crate::model::AppState;
use crate::{arg, password, Result};
use axum::extract::{Extension, Form, Path, Query};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Html;
use std::sync::Arc;
use tower_cookies::Cookies;

pub async fn index(
    Extension(state): Extension<Arc<AppState>>,
    args: Option<Query<arg::BackendQueryArg>>,
) -> Result<Html<String>> {
    let handler_name = "backend_admin_index";
    let args = args.unwrap();
    let q_keyword = format!("%{}%", args.keyword());
    let client = get_client(&state, handler_name).await?;
    let admin_list = admin::select(
        &client,
        Some("is_del=$1 AND username ILIKE $2"),
        &[&args.is_del(), &q_keyword],
        args.page(),
    )
    .await
    .map_err(log_error(handler_name.to_string()))?;
    let tmpl = IndexTemplate {
        list: admin_list,
        arg: args.0,
    };
    render(tmpl, handler_name)
}

pub async fn add() -> Result<Html<String>> {
    let handler_name = "backend_admin_add";
    let tmpl = AddTemplate {};
    render(tmpl, handler_name)
}

pub async fn add_action(
    Extension(state): Extension<Arc<AppState>>,
    Form(ca): Form<CreateAdmin>,
) -> Result<(StatusCode, HeaderMap, ())> {
    let handler_name = "backend_admin_add_action";
    if ca.password.is_empty() {
        return Err(AppError::from_str("请输入密码", AppErrorType::Common));
    }

    if ca.password.len() < 6 {
        return Err(AppError::from_str(
            "密码长度不能小于6位",
            AppErrorType::Common,
        ));
    }

    if ca.username.contains(ca.password.as_str()) {
        return Err(AppError::from_str(
            "密码设置不安全,请重新设置",
            AppErrorType::Common,
        ));
    }

    if &ca.password != &ca.re_password {
        return Err(AppError::from_str(
            "两次输入的密码不一致",
            AppErrorType::Common,
        ));
    }
    let client = get_client(&state, handler_name).await?;
    let mut ca = CreateAdmin { ..ca };
    ca.password = password::hash(&ca.password)?;
    admin::create(&client, ca)
        .await
        .map_err(log_error(handler_name.to_string()))?;
    redirect("/admin/admin?msg=账号添加成功")
}

pub async fn edit(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Html<String>> {
    let handler_name = "backend_admin_edit";
    let client = get_client(&state, handler_name).await?;
    let item = admin::find_by_id(&client, id)
        .await
        .map_err(log_error(handler_name.to_string()))?;
    let tmpl = EditTemplate { admin: item };
    render(tmpl, handler_name)
}

pub async fn edit_action(
    Extension(state): Extension<Arc<AppState>>,
    Form(ua): Form<UpdateAdmin>,
    Extension(ck): Extension<Cookies>,
) -> Result<(StatusCode, HeaderMap, ())> {
    let handler_name = "backend_admin_edit_action";
    if ua.new_password.is_empty() {
        return Err(AppError::from_str("请输入密码", AppErrorType::Common));
    }
    if &ua.new_password != &ua.re_password {
        return Err(AppError::from_str(
            "两次输入的密码不一致",
            AppErrorType::Common,
        ));
    }

    let admin_session = get_login_admin_by_cookie(&state, &ck).await?;
    if admin_session.is_none() {
        return Err(AppError::auth_error("UNAUTHENTICATED"));
    }

    let admin_session = admin_session.unwrap();
    if !password::verify(&ua.password, &admin_session.password)? {
        return Err(AppError::auth_error("你输入的密码错误."));
    }
    let mut ua = UpdateAdmin { ..ua };
    ua.new_password = password::hash(&ua.new_password)?;
    let client = get_client(&state, handler_name).await?;
    admin::update(&client, ua)
        .await
        .map_err(log_error(handler_name.to_string()))?;
    redirect("/admin/admin?msg=账号修改成功")
}

pub async fn del(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<(StatusCode, HeaderMap, ())> {
    let handler_name = "backend_admin_del";
    let client = get_client(&state, handler_name).await?;
    admin::del_or_recover(&client, id, true)
        .await
        .map_err(log_error(handler_name.to_string()))?;
    redirect("/admin/admin?msg=账号删除成功")
}

pub async fn recover(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<(StatusCode, HeaderMap, ())> {
    let handler_name = "backend_admin_recover";
    let client = get_client(&state, handler_name).await?;
    admin::del_or_recover(&client, id, false)
        .await
        .map_err(log_error(handler_name.to_string()))?;
    redirect("/admin/admin?msg=账号恢复成功")
}
