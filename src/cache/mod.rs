pub mod redisdb;
pub mod sleddb;

use crate::error::AppError;
use serde::{Deserialize, Serialize};

use crate::Result;

/// 对缓存的抽象,我们不关心数据存在哪儿,但需要定义外界如何在存储打交道
pub trait Cache: Send + Sync + 'static {
    fn get(&self, key: &str) -> Result<Option<StrValue>>;
    // 设置一个key的value,返回旧的value
    fn set(&self, key: String, value: StrValue) -> Result<Option<StrValue>>;
    // 删除key值,返回旧的value
    fn del(&self, key: &str) -> Result<Option<StrValue>>;

    // 是否存在
    fn is_exists(&self, key: &str) -> Result<bool>;
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StrValue {
    value: String,
    expired: Option<i64>, // 时间戳
}

impl StrValue {
    pub fn set_expired(&mut self, expired: i64) {
        self.expired = Some(expired);
    }
}

impl From<&str> for StrValue {
    fn from(value: &str) -> Self {
        StrValue {
            value: value.to_string(),
            expired: None,
        }
    }
}

impl From<String> for StrValue {
    fn from(value: String) -> Self {
        StrValue {
            value,
            expired: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::sleddb::SledDb;

    #[test]
    fn sleddb_basic_interface_should_work() {
        let cache = SledDb::new("./test_sleddb");
        test_basic_interface(cache);
    }

    fn test_basic_interface(cache: impl Cache) {
        // 第一次set值
        let v1 = cache.set("k1".into(), "v1".into());
        assert!(v1.unwrap().is_none());

        // 第二次set值,会返回旧的值
        let v2 = cache.set("k1".into(), "v2".into());
        assert_eq!(v2.unwrap(), Some("v1".into()));

        // get 存在的key会得到最新的值
        let v = cache.get("k1");
        assert_eq!(v.unwrap(), Some("v2".into()));

        // get不存在的key会返回None
        let v = cache.get("k2");
        assert_eq!(v.unwrap(), None);

        // is_exists 存在的key会返回true, 否则返回false
        assert!(cache.is_exists("k1").unwrap());
        assert!(!cache.is_exists("k2").unwrap());

        // del存在的key会返回旧的值
        let v = cache.del("k1");
        assert_eq!(v.unwrap(), Some("v2".into()));

        // del不存在的key会返回None
        assert_eq!(cache.del("k2").unwrap(), None);
    }
}
