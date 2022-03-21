use crate::error::AppError;
use crate::Result;
use askama::Template;
use axum::response::Html;

pub fn render<T: Template>(tmpl: T, handler_name: &str) -> Result<Html<String>> {
    let out = tmpl.render().map_err(|err| {
        tracing::error!("模板渲染出错: {:?}, {}", err, handler_name);
        AppError::from(err)
    })?;
    Ok(Html(out))
}
