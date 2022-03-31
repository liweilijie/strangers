use crate::cache::{Cache, StrValue};
use crate::error::AppError;
use crate::Result;
use redis::aio::Connection;
use redis::AsyncCommands;
use redis::Client;

pub struct RedisDB(Client);

impl RedisDB {
    pub fn new(client: Client) -> RedisDB {
        RedisDB(client)
    }

    pub async fn get_conn(&self) -> Result<Connection> {
        self.0.get_async_connection().await.map_err(AppError::from)
    }
}

impl Cache for RedisDB {
    fn get(&self, key: &str) -> Result<Option<StrValue>> {
        todo!()
    }

    fn set(&self, key: String, value: StrValue) -> Result<Option<StrValue>> {
        todo!()
    }

    fn del(&self, key: &str) -> Result<Option<StrValue>> {
        todo!()
    }

    fn is_exists(&self, key: &str) -> Result<bool> {
        todo!()
    }
}
