use crate::db::pagination::Pagination;
use crate::db::select_stmt::SelectStmt;
use crate::db::PAGE_SIZE;
use crate::error::{AppError, AppErrorType};
use crate::form::CreateMedicinal;
use crate::model::{MedicinalID, MedicinalList};
use crate::Result;
use std::str::FromStr;
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
        .fields("id, category, name, batch_number, count, validity, is_del")
        .condition(Some(condition))
        .order(Some("validity ASC"))
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

/// 创建药品，返回新创建的药品的 ID 或者包含AppError的错误信息
///
/// # 参数
///
/// * `client` - 数据库连接对象
/// * `medicinal` - 药品信息
pub async fn create(client: &Client, medicinal: &CreateMedicinal) -> Result<MedicinalID> {
    debug!("medicinal create: {:?}", medicinal);
    if is_exists_name_category_batch_number(
        client,
        &medicinal.name,
        &medicinal.category,
        &medicinal.batch_number,
    )
    .await?
    {
        return Err(crate::error::AppError::is_exists(&format!(
            "药品名称:'{}' 或类目:'{}' 或者批号: '{}' 已存在",
            medicinal.name, medicinal.category, medicinal.batch_number
        )));
    }

    let validity: chrono::NaiveDate =
        chrono::NaiveDate::parse_from_str(&medicinal.validity, "%Y%m%d").map_err(|err| {
            AppError::from_str(
                &format!("日期格式错误,错误信息:'{}'", err.to_string()),
                AppErrorType::DbError,
            )
        })?;
    debug!("medicinal create: {:?}", validity);

    let sql = "INSERT INTO medicinal (category, name, batch_number, count, validity) VALUES ($1, $2, $3, $4, $5) RETURNING id";
    Ok(super::query_one(
        client,
        sql,
        &[
            &medicinal.category,
            &medicinal.name,
            &medicinal.batch_number,
            &medicinal.count,
            &validity,
        ],
        Some("插入药品失败"),
    )
    .await?)
}

/// 根据条件判断药品是否存在，返回药品是否存在，或者包含AppError的错误信息
///
/// # 参数
///
/// * `client` - 数据库连接对象
/// * `condition` - 条件
/// * `args` - 条件对应的参数
async fn is_exists(
    client: &Client,
    condition: Option<&str>,
    args: &[&(dyn ToSql + Sync)],
) -> Result<bool> {
    let c = count(client, condition, args).await?;
    Ok(c > 0)
}

/// 根据条件统计，返回符合条件的记录数，或者包含AppError的错误信息
///
/// # 参数
///
/// * `client` - 数据库连接对象
/// * `condition` - 条件
/// * `args` - 条件对应的参数
async fn count(
    client: &Client,
    condition: Option<&str>,
    args: &[&(dyn ToSql + Sync)],
) -> Result<i64> {
    let sql = SelectStmt::builder()
        .table(TABLE_NAME)
        .fields("COUNT(*)")
        .condition(condition)
        .build();
    debug!("medicinal count sql: {}", sql);
    super::count(client, &sql, args).await
}

/// 判断药品名称和批号是否存在，或者包含AppError的错误信息
///
/// # 参数
///
/// * `client` - 数据库连接对象
/// * `name` - 药品名称
/// * `batch_number` - 批号
pub async fn is_exists_name_batch_number(
    client: &Client,
    name: &str,
    batch_number: &str,
) -> Result<bool> {
    let condition = Some("name = $1 AND batch_number = $2");
    // let args = &[name, batch_number];
    is_exists(client, condition, &[&name, &batch_number]).await
}

/// 判断药品名称和批号是否存在，或者包含AppError的错误信息
///
/// # 参数
///
/// * `client` - 数据库连接对象
/// * `name` - 药品名称
/// * `category` - 类目
/// * `batch_number` - 批号
pub async fn is_exists_name_category_batch_number(
    client: &Client,
    name: &str,
    category: &str,
    batch_number: &str,
) -> Result<bool> {
    let condition = Some("name = $1 AND category = $2 AND batch_number = $3");
    is_exists(client, condition, &[&name, &category, &batch_number]).await
}
