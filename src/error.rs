use crate::html::err::ErrTemplate;
use askama::Error;
use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use std::fmt::Formatter;

/// 应用错误类型
#[derive(Debug)]
pub enum AppErrorType {
    /// Template错误
    Template,
    /// 通用错误
    Common,
}

/// 应用错误
#[derive(Debug)]
pub struct AppError {
    /// 错误类型
    pub error_type: AppErrorType,
    /// 错误信息
    pub message: Option<String>,
    /// 错误原因
    pub cause: Option<String>,
}

impl AppError {
    /// 从其他错误中实例化
    pub fn from_err(err: impl ToString, error_type: AppErrorType) -> Self {
        Self {
            message: None,
            cause: Some(err.to_string()),
            error_type,
        }
    }

    pub fn tmpl_err(err: impl ToString) -> Self {
        Self {
            message: Some("渲染模板出错".to_owned()),
            cause: Some(err.to_string()),
            error_type: AppErrorType::Template,
        }
    }
    fn status_code(&self) -> StatusCode {
        match self.error_type {
            AppErrorType::Template => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

impl std::error::Error for AppError {}
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.message)
    }
}
impl From<askama::Error> for AppError {
    fn from(err: Error) -> Self {
        Self::tmpl_err(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = (&self).status_code();
        let msg = match self {
            AppError{
                message: Some(msg), .. // TODO: 这个是什么操作?
            } => msg.clone(),
            AppError {
                error_type: AppErrorType::Template,
                ..
            } => "模板渲染出错".to_string(),
            _ => "发生错误".to_string(),
        };

        let tmpl = ErrTemplate {
            err: msg.to_string(),
        };

        let msg = tmpl.render().unwrap_or(msg.to_string()); // render 在Template里面定义的trait, 所以必须要导入Template
        (status_code, Html(msg)).into_response()
    }
}
