use crate::db::medicinal;
use crate::handler::helper::{get_client, log_error, render};
use crate::handler::redirect::redirect;
use crate::html::backend::medicinal::AddTemplate;
use crate::html::backend::medicinal::IndexTemplate;
use crate::model::AppState;
use crate::{arg, Result};
use axum::extract::{Extension, Query};
use axum::http::StatusCode;
use axum::response::Html;
use reqwest::header::HeaderMap;
use std::sync::Arc;

pub async fn index(
    Extension(state): Extension<Arc<AppState>>,
    args: Option<Query<arg::MedicinalBackendQueryArg>>,
) -> Result<Html<String>> {
    let handler_name = "backend_medicinal_index";
    let client = get_client(&state, handler_name).await?;
    let args = args.unwrap().0;
    let q_keyword = format!("%{}%", args.keyword());
    let medicinal_list = medicinal::select(
        &client,
        "is_del=$1 AND name LIKE $2",
        &[&args.is_del(), &q_keyword],
        args.page.unwrap_or(0),
    )
    .await
    .map_err(log_error(handler_name.to_string()))?;
    let tmpl = IndexTemplate {
        arg: args,
        list: medicinal_list,
    };
    render(tmpl, handler_name)
}

pub async fn add() -> Result<Html<String>> {
    let tmpl = AddTemplate {};
    render(tmpl, "backend_medicinal_add")
}

pub async fn add_action(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<(StatusCode, HeaderMap, ())> {
    let handler_name = "backend_medicinal_add_action";
    let client = get_client(&state, handler_name).await?;
    // todo: add_action
    redirect("/admin/medicinal?msg=药品添加成功")
}
