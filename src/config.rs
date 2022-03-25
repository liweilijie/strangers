//! 配置

use serde::Deserialize;

#[derive(Debug, Deserialize, Default, PartialEq)]
pub struct SmsConfig {
    pub check_interval: Option<u64>,
    pub send_sms_toggle: Option<bool>,
    pub expired_days: Option<i64>,
    pub phones: Option<Vec<String>>,
}

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

#[derive(Debug, Deserialize, Clone)]
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
    pub upload_dir: Option<String>,
    pub sms: Option<SmsConfig>,
    pub ssl_enable: bool,
}

impl Config {
    // https://github.com/mehcode/config-rs/blob/master/examples/hierarchical-env/settings.rs
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let cfg = config::Config::builder()
            .add_source(config::Environment::default().try_parsing(true))
            .build()?;

        cfg.try_deserialize()
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::Config;

    #[test]
    fn test_config() {
        dotenv::dotenv().ok();
        let cfg = Config::from_env().unwrap();
        println!("{:#?}", cfg);
        assert_eq!(cfg.web.addr, "0.0.0.0:9528".to_string());
    }

    #[derive(Debug, Default, serde::Deserialize, PartialEq)]
    struct AppConfig {
        list: Vec<String>,
    }

    #[test]
    fn test_list_parse() {
        std::env::set_var("APP_LIST", "Hello World");

        let config = config::Config::builder()
            .add_source(
                config::Environment::with_prefix("APP")
                    .try_parsing(true)
                    .separator("_")
                    .list_separator(" "),
            )
            .build()
            .unwrap();

        let app: AppConfig = config.try_deserialize().unwrap();

        assert_eq!(app.list, vec![String::from("Hello"), String::from("World")]);

        std::env::remove_var("APP_LIST");
    }
}
