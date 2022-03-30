use crate::cache::{Cache, StrValue};
use crate::error::AppError;
use crate::error::AppErrorType::SledError;
use crate::Result;
use sled::Db;
use std::path::Path;

#[derive(Debug)]
pub struct SledDb(Db);

impl SledDb {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self(sled::open(path).unwrap())
    }
}

/// 把Option<Result<T, E>> flip 成 Result<Option<T>, E>
/// 从这个函数里,你可以看到函数式编程的优雅
fn flip<T>(x: Option<Result<T>>) -> Result<Option<T>> {
    x.map_or(Ok(None), |x| x.map(Some))
}

impl Cache for SledDb {
    fn get(&self, key: &str) -> Result<Option<StrValue>> {
        let result: Option<Result<StrValue>> = self.0.get(key.as_bytes())?.map(|v| {
            serde_json::from_slice(v.as_ref()).map_err(|e| AppError::from_err(e, SledError))
        });
        flip(result)
    }

    fn set(&self, key: String, value: StrValue) -> Result<Option<StrValue>> {
        // 需要将StrValue序列化为json字符串之后再做存储
        let data = serde_json::to_vec(&value)?;
        let result: Option<Result<StrValue>> = self.0.insert(key, data)?.map(|v| {
            serde_json::from_slice(v.as_ref()).map_err(|e| AppError::from_err(e, SledError))
        });
        flip(result)
    }

    fn del(&self, key: &str) -> Result<Option<StrValue>> {
        let result: Option<Result<StrValue>> = self.0.remove(key.as_bytes())?.map(|v| {
            serde_json::from_slice(v.as_ref()).map_err(|e| AppError::from_err(e, SledError))
        });
        flip(result)
    }

    fn is_exists(&self, key: &str) -> Result<bool> {
        Ok(self.0.contains_key(key.as_bytes())?)
    }
}
