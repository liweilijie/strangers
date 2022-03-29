use crate::config::SessionConfig;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use tower_cookies::Cookies;
use uuid::Uuid;

pub fn gen_redis_key(cfg: &SessionConfig, id: &str) -> String {
    format!("{}{}", &cfg.prefix, id)
}

pub struct GeneratedKey {
    pub id: String,
    pub cookie_key: String,
    pub redis_key: String,
}

pub fn id() -> String {
    Uuid::new_v4().to_simple().to_string()
}

// cookie_key的生成有一些问题
// 需要考虑到同一个客户端在不同的浏览器中的cookie_key是不一样的
// 所以cookie_key的生成规则是: id_name+timestamp+random
pub fn gen_key(cfg: &SessionConfig) -> GeneratedKey {
    let id = id();
    let cookie_key = cfg.id_name.to_string();
    let redis_key = gen_redis_key(cfg, &id);
    GeneratedKey {
        id,
        cookie_key,
        redis_key,
    }
}

pub fn gen_user_cookie_key(prefix: &str) -> String {
    // 获取毫秒时间戳
    let ts = chrono::Local::now().timestamp_millis();
    // 获取7个随机字符
    let mut rng = thread_rng();
    // String:
    let s: String = (&mut rng)
        .sample_iter(Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    format!("{}{}_{}", prefix, ts, s)
}

// 使用前缀获取cookie的key,value值
pub fn get_cookie_by_pre(ck: &Cookies, prefix: &str) -> Option<CookieKV> {
    let rule = r"^".to_owned() + prefix + r"\d{8,}_\w{7}" + r"$";
    let re = regex::Regex::new(&rule).unwrap();
    ck.list()
        .iter()
        .find(|c| re.is_match(c.name()))
        .map(|c| CookieKV {
            name: c.name().to_string(),
            value: c.value().to_string(),
        })
}

#[derive(Debug, Default)]
pub struct CookieKV {
    pub name: String,
    pub value: String,
}

mod tests {
    #[test]
    fn test_regex_rule() {
        let prefix = "strangers_session";
        let rule = r"^".to_owned() + prefix + r"\d{8,}_\w{7}" + r"$";
        println!("{}", rule);

        let re = regex::Regex::new(&rule).unwrap();
        assert!(re.is_match("strangers_session12345678_2323abc"));
    }
}
