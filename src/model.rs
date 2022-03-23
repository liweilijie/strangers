use crate::config::{HCaptchaConfig, SessionConfig};
use redis::Client;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

pub struct AppState {
    pub pool: deadpool_postgres::Pool,
    pub rdc: Client,
    pub sess_cfg: SessionConfig,
    pub hcap_cfg: HCaptchaConfig,
    pub upload_dir: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AdminSession {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub is_sys: bool,
    pub dateline: i32,
}

#[derive(PostgresMapper)]
#[pg_mapper(table = "admin")]
pub struct Admin {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub is_sys: bool,
    pub is_del: bool,
}

#[derive(PostgresMapper)]
#[pg_mapper(table = "admin")]
pub struct AdminID {
    pub id: i32,
}

#[derive(PostgresMapper)]
#[pg_mapper(table = "medicinal")]
pub struct MedicinalList {
    pub id: i32,
    pub category: String,
    pub name: String,
    pub batch_number: String,
    pub count: String,
    pub validity: chrono::NaiveDate,
    pub is_del: bool,
}

#[derive(PostgresMapper)]
#[pg_mapper(table = "medicinal")]
pub struct MedicinalID {
    pub id: i32,
}
