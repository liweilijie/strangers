use crate::error::AppError;
use crate::model::AppState;
use crate::Result;
use askama::Template;
use axum::http::header::COOKIE;
use axum::http::HeaderMap;
use axum::response::Html;
use deadpool_postgres::Client;
use tracing::debug;

pub fn render<T: Template>(tmpl: T, handler_name: &str) -> Result<Html<String>> {
    let out = tmpl.render().map_err(|err| {
        tracing::error!("模板渲染出错: {:?}, {}", err, handler_name);
        AppError::from(err)
    })?;
    Ok(Html(out))
}

pub fn get_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookie = headers
        .get(COOKIE)
        // Result.ok() 从 Result<T, E> 转换为 Option<T>。
        // 将 self 转换为 Option<T>，使用 self，并丢弃错误 (如果有)。
        .and_then(|value| {
            debug!("cookie.value: {:?}", value);
            value.to_str().ok()
        })
        .map(|value| value.to_string());
    debug!("cookie: {:?}", cookie);
    match cookie {
        Some(cookie) => {
            let cookie = cookie.as_str();
            let cs: Vec<&str> = cookie.split(';').collect();
            for item in cs {
                let item: Vec<&str> = item.split('=').collect();
                if item.len() != 2 {
                    continue;
                }
                let key = item[0];
                let val = item[1];
                let key = key.trim();
                let val = val.trim();
                if key == name {
                    return Some(val.to_string());
                }
            }
            None
        }
        None => None,
    }
}

pub async fn get_client(state: &AppState, handler_name: &str) -> Result<Client> {
    state.pool.get().await.map_err(|err| {
        tracing::error!("无法获取数据库连接: {:?}, {}", err, handler_name);
        AppError::from(err)
    })
}

// TODO:
pub fn log_error(handler_name: String) -> Box<dyn Fn(AppError) -> AppError> {
    Box::new(move |err| {
        tracing::error!("操作失败: {:?}, {}", err, handler_name);
        err
    })
}
