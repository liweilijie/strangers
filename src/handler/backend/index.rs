use crate::handler::helper::render;
use crate::html::backend::index::IndexTemplate;
use crate::Result;
use axum::response::Html;

pub async fn index() -> Result<Html<String>> {
    let handler_name = "backend_index";
    let tmpl = IndexTemplate {};
    render(tmpl, handler_name)
}
