use crate::config::{HCaptchaConfig, SessionConfig, SmsConfig};
use crate::sms::sms::EXPIRED_DAYS;
use chrono::Datelike;
use redis::Client;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tokio_pg_mapper_derive::PostgresMapper;

pub struct AppState {
    pub pool: deadpool_postgres::Pool,
    pub rdc: Client,
    pub sess_cfg: SessionConfig,
    pub hcap_cfg: HCaptchaConfig,
    pub upload_dir: String,
    pub sms_cfg: Option<SmsConfig>,
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
    pub spec: String,
    pub count: String,
    pub validity: chrono::NaiveDate,
    pub is_del: bool,
}

impl MedicinalList {
    pub fn is_expired(&self) -> bool {
        let now = chrono::Local::now();
        let validity = chrono::NaiveDate::from_ymd(now.year(), now.month(), now.day());

        self.validity <= validity
    }

    pub fn is_expired_as1(&self) -> bool {
        let now = chrono::Local::now();
        let validity = chrono::NaiveDate::from_ymd(now.year(), now.month(), now.day());

        self.validity <= validity + chrono::Duration::days(EXPIRED_DAYS)
    }

    pub fn is_expired_as3(&self) -> bool {
        let now = chrono::Local::now();
        let validity = chrono::NaiveDate::from_ymd(now.year(), now.month(), now.day());

        self.validity <= validity + chrono::Duration::days(91)
    }

    pub fn is_expired_as6(&self) -> bool {
        let now = chrono::Local::now();
        let validity = chrono::NaiveDate::from_ymd(now.year(), now.month(), now.day());

        self.validity <= validity + chrono::Duration::days(180)
    }
}

pub struct ExpiredItem {
    pub name: String,
    pub id: u8,
}

pub fn get_expired_str() -> Vec<ExpiredItem> {
    vec![
        ExpiredItem {
            name: "所有数据".to_string(),
            id: 0,
        },
        ExpiredItem {
            name: "已经过期".to_string(),
            id: 1,
        },
        ExpiredItem {
            name: "一个月".to_string(),
            id: 2,
        },
        ExpiredItem {
            name: "三个月".to_string(),
            id: 4,
        },
        ExpiredItem {
            name: "六个月".to_string(),
            id: 7,
        },
        ExpiredItem {
            name: "九个月".to_string(),
            id: 10,
        },
    ]
}

impl Display for MedicinalList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}-{}-{}-{}-{}-{}",
            self.name, self.batch_number, self.spec, self.category, self.count, self.validity
        )
    }
}

#[derive(PostgresMapper)]
#[pg_mapper(table = "medicinal")]
pub struct MedicinalID {
    pub id: i32,
}

#[derive(PostgresMapper)]
#[pg_mapper(table = "medicinal")]
pub struct Category {
    pub category: String,
}
