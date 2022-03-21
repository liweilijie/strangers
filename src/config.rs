//! 配置

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WebConfig {
    /// web服务监听地址
    pub addr: String,
    /// 安全key
    pub secret_key: String,
}

#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    pub dsn: String,
}

#[derive(Debug, Deserialize)]
pub struct SessionConfig {
    pub prefix: String,
    pub id_name: String,
    pub expired: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HCaptchaConfig {
    pub site_key: String,
    pub secret_key: String,
}

/// 配置
#[derive(Debug, Deserialize)]
pub struct Config {
    /// web配置
    pub web: WebConfig,
    /// Postgres配置
    pub pg: deadpool_postgres::Config,
    pub redis: RedisConfig,
    pub session: SessionConfig,
    pub hcaptcha: HCaptchaConfig,
}

impl Config {
    // https://github.com/mehcode/config-rs/blob/master/examples/hierarchical-env/settings.rs
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut cfg = config::Config::builder()
            .add_source(config::Environment::default())
            .build()?;

        cfg.try_deserialize()
    }
}

mod tests {
    use super::Config;

    #[test]
    fn test_config() {
        dotenv::dotenv().ok();
        let cfg = Config::from_env().unwrap();
        // println!("{:#?}", cfg);
        assert_eq!(cfg.web.addr, "127.0.0.1:9527".to_string());
    }
}
