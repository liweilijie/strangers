use redis::Client;
use crate::config::{HCaptchaConfig, SessionConfig};

pub struct AppState {
    pub pool: deadpool_postgres::Pool,
    pub rdc: Client,
    pub sess_cfg: SessionConfig,
    pub hcap_cfg: HCaptchaConfig,
}