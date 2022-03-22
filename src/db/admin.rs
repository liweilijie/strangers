use crate::db::select_stmt::SelectStmt;
use crate::model::Admin;
use crate::Result;
use tokio_postgres::types::ToSql;
use tokio_postgres::Client;

pub async fn find_by_condition(
    client: &Client,
    condition: &str,
    args: &[&(dyn ToSql + Sync)],
) -> Result<Admin> {
    let sql = SelectStmt::builder()
        .table("admin")
        .fields("id, username, password, is_sys, is_del")
        .condition(Some(condition))
        .limit(Some(1))
        .build();
    Ok(super::query_one(client, &sql, args, Some("不存在的管理员")).await?)
}
pub async fn find(client: &Client, username: &str) -> Result<Admin> {
    find_by_condition(client, "username=$1 AND is_del=false", &[&username]).await
}
pub async fn find_by_id(client: &Client, id: i32) -> Result<Admin> {
    find_by_condition(client, "id=$1 AND is_del=false", &[&id]).await
}
