pub mod arg;
pub mod config;
pub mod db;
pub mod error;
pub mod form;
pub mod handler;
pub mod hcaptcha;
pub mod html;
pub mod middleware;
pub mod model;
pub mod password;
pub mod rdb;
pub mod session;
pub mod sms;
pub mod time;

/// 结果
type Result<T> = std::result::Result<T, self::error::AppError>;
