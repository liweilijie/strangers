use crate::db::pagination::Pagination;
use crate::db::select_stmt::SelectStmt;
use crate::db::PAGE_SIZE;
use crate::model::MedicinalList;
use crate::Result;
use tokio_postgres::types::ToSql;
use tokio_postgres::Client;
use tracing::debug;

/// 表名
const TABLE_NAME: &str = "medicinal";

/// 获取药品列表,返回满足条件的药品列表及分页信息或者包含AppError的错误信息
///
/// # 参数
///
/// * `client` - 数据库连接对象
/// * `condition` - 条件
/// * `args` - 条件对应的参数
/// * `page` - 当前分页的页码
pub async fn select(
    client: &Client,
    condition: &str,
    args: &[&(dyn ToSql + Sync)],
    page: u32,
) -> Result<Pagination<Vec<MedicinalList>>> {
    let sql = SelectStmt::builder()
        .table(TABLE_NAME)
        .fields("id, category, name, batch_number, count, validity")
        .condition(Some(condition))
        .order(Some("id DESC"))
        .limit(Some(PAGE_SIZE))
        .offset(Some(page * PAGE_SIZE as u32))
        .build();
    let count_sql = SelectStmt::builder()
        .table(TABLE_NAME)
        .fields("COUNT(*)")
        .condition(Some(condition))
        .build();
    debug!("medicinal select sql: {}", sql);
    debug!("medicinal select count sql: {}", count_sql);
    Ok(super::select(client, &sql, &count_sql, args, page).await?)
}
