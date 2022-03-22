use crate::error::AppError;
use crate::Result;
use redis::aio::Connection;
use redis::AsyncCommands;
use redis::Client;

/// 获取连接
async fn get_conn(client: &Client) -> Result<Connection> {
    client.get_async_connection().await.map_err(AppError::from)
}

/// 将数据写入 redis
pub async fn set(client: &Client, key: &str, value: &str, sec: usize) -> Result<()> {
    let mut conn = get_conn(client).await?;
    conn.set_ex(key, value, sec).await.map_err(AppError::from)
}

/// 从 redis 中获取 数据
pub async fn get(client: &Client, key: &str) -> Result<Option<String>> {
    let mut conn = get_conn(client).await?;
    let s: Option<String> = conn.get(key).await.map_err(AppError::from)?;
    Ok(s)
}

/// 判断指定的键是否存在于 redis 中
pub async fn is_exists(client: &Client, key: &str) -> Result<bool> {
    let mut conn = get_conn(client).await?;
    let s: bool = conn.exists(key).await.map_err(AppError::from)?;
    Ok(s)
}

/// 删除指定的键
pub async fn del(client: &Client, key: &str) -> Result<()> {
    let mut conn = get_conn(client).await?;
    conn.del(key).await.map_err(AppError::from)
}
