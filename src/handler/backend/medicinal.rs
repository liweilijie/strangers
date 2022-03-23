use crate::db::medicinal;
use crate::error::{AppError, AppErrorType};
use crate::form::CreateMedicinal;
use crate::handler::helper::{get_client, log_error, render};
use crate::handler::redirect::redirect;
use crate::html::backend::medicinal::{AddTemplate, IndexTemplate, UploadTemplate};
use crate::model::AppState;
use crate::{arg, form, Result};
use axum::extract::{ContentLengthLimit, Extension, Form, Multipart, Query};
use axum::http::StatusCode;
use axum::response::Html;
use calamine::DataType::Empty;
use calamine::{open_workbook, open_workbook_auto, Reader, Xls, Xlsx};
use reqwest::header::HeaderMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::field::debug;
use tracing::{debug, error, warn};

/// 允许上传的文件大小
const MAX_UPLOAD_SIZE: u64 = 1024 * 1024 * 256; // 256 MB

pub async fn index(
    Extension(state): Extension<Arc<AppState>>,
    args: Option<Query<arg::MedicinalBackendQueryArg>>,
) -> Result<Html<String>> {
    let handler_name = "backend_medicinal_index";
    let client = get_client(&state, handler_name).await?;
    let args = args.unwrap().0;
    let q_keyword = format!("%{}%", args.keyword());
    let medicinal_list = medicinal::select(
        &client,
        "is_del=$1 AND name LIKE $2",
        &[&args.is_del(), &q_keyword],
        args.page.unwrap_or(0),
    )
    .await
    .map_err(log_error(handler_name.to_string()))?;
    let tmpl = IndexTemplate {
        arg: args,
        list: medicinal_list,
    };
    render(tmpl, handler_name)
}

pub async fn add() -> Result<Html<String>> {
    let tmpl = AddTemplate {};
    render(tmpl, "backend_medicinal_add")
}

pub async fn add_action(
    Extension(state): Extension<Arc<AppState>>,
    form: Form<form::CreateMedicinal>,
) -> Result<(StatusCode, HeaderMap, ())> {
    let handler_name = "backend_medicinal_add_action";
    let client = get_client(&state, handler_name).await?;
    debug!("add_action and form: {:#?}", form);
    medicinal::create(&client, &form)
        .await
        .map_err(log_error(handler_name.to_string()))?;

    redirect("/admin/medicinal?msg=药品添加成功")
}

pub async fn upload() -> Result<Html<String>> {
    let tmpl = UploadTemplate {};
    render(tmpl, "backend_medicinal_upload")
}

pub async fn upload_action(
    Extension(state): Extension<Arc<AppState>>,
    ContentLengthLimit(mut multipart): ContentLengthLimit<Multipart, { MAX_UPLOAD_SIZE }>,
) -> Result<(StatusCode, HeaderMap, ())> {
    debug!("upload_action and multipart: {:#?}", multipart);
    if let Some(file) = multipart.next_field().await.map_err(|err| {
        error!("upload_action error: {:?}", err);
        AppError::from_str("获取文件失败", AppErrorType::UploadError)
    })? {
        let filename = file.file_name().unwrap().to_string(); // 上传的文件名称
        let sc = PathBuf::from(filename.clone());
        match sc.extension().and_then(|s| s.to_str()) {
            Some("xlsx") | Some("xlsm") | Some("xlsb") | Some("xls") => (),
            _ => {
                return Err(AppError::from_str(
                    "excel文件格式错误",
                    AppErrorType::UploadError,
                ))
            }
        }

        let data = file.bytes().await.unwrap(); // 上传的文件内容

        debug!("upload_action and filename: {:#?}", filename);
        debug!("upload_action and data size: {}", data.len());

        let to_path = format!("{}/{}", &state.upload_dir, filename);

        debug!("upload_action and to_path: {}", to_path);

        // 保存上传的文件
        tokio::fs::write(&to_path, &data)
            .await
            .map_err(|err| AppError::from_err(err, AppErrorType::UploadError))?;

        // 读取 excel内容
        let (result, total_count, success_count) = load_excel_file(&to_path).await?;
        // 将读取到的数据 insert 到数据库中
        let mut insert_count = 0;
        if result.len() > 0 {
            let client = get_client(&state, "backend_medicinal_upload_action").await?;
            for value in &result {
                debug!("upload_action and row: {:#?}", value);
                medicinal::create(&client, value).await.map_or_else(
                    |e| {
                        error!("upload_action error: {:?}", e);
                    },
                    |_| insert_count += 1,
                )
            }
        }

        debug!(
            "upload_action and total_count: {} and insert_count: {}",
            total_count, insert_count
        );

        redirect(&format!(
            "/admin/medicinal?msg=文件:{}上传成功, 总共上传成功条目:{}, 失败条目:{}, 文件大小:{}",
            filename,
            insert_count,
            total_count - insert_count,
            data.len()
        ))
    } else {
        redirect("/admin/medicinal?msg=上传失败")
    }
}

async fn load_excel_file(file: &str) -> Result<(Vec<CreateMedicinal>, u32, u32)> {
    // 判断文件类型是否合法
    let sce = PathBuf::from(file);
    // TODO:
    match sce.extension().and_then(|s| s.to_str()) {
        Some("xlsx") | Some("xlsm") | Some("xlsb") | Some("xls") => (),
        _ => {
            return Err(AppError::from_str(
                "excel文件格式错误",
                AppErrorType::UploadError,
            ))
        }
    }
    // let mut excel: Xlsx<_> = open_workbook(file)?;
    let mut excel = open_workbook_auto(file)?;
    let mut category: Option<String> = None;
    let name_keys = vec!["药品", "名称", "药名", "项目", "型号"];
    let validity_keys = vec!["有效期", "有效期至", "效期", "有效", "日期"];
    let count_keys = vec!["数量", "基数", "数"];
    let batch_number_keys = vec!["批号", "批号号"];

    let mut name_index: Option<usize> = None;
    let mut count_index: Option<usize> = None;
    let mut validity_index: Option<usize> = None;
    let mut batch_number_index: Option<usize> = None;
    let mut index_ok = false;
    let mut result_content: Vec<CreateMedicinal> = vec![];
    let mut success_count: u32 = 0;
    let mut total_count: u32 = 0;

    if let Some(Ok(r)) = excel.worksheet_range("Sheet1") {
        for row in r.rows() {
            total_count += 1;
            // debug!("row: {:#?}", row);
            // 最正常的情况下有四列，第一列是类别，第二列是药品名称，第三列是数量，第四列是有效期 当然顺序还可能不一样，所以要处理顺序
            // 说明有批号
            // 查找 category
            // 至少需要2列才正常
            if row.len() < 2 {
                error!("excel column too small row.len < 2");
                continue;
            }

            if category.is_none() {
                // 判断是否是第一行为类目
                // 所有数据其他都为 Empty, 只有一个值是 String 类型的则为category
                if row[0].is_string() {
                    let mut other_is_empty = true;
                    for (index, val) in row.iter().enumerate() {
                        if index == 0 {
                            continue;
                        } else {
                            if !val.is_empty() {
                                other_is_empty = false;
                            }
                        }
                    }
                    if other_is_empty {
                        category = Some(row[0].to_string().trim().to_string());
                        debug!("category: {:?}", category);
                        total_count -= 1;
                        continue;
                    }
                }
            }

            // 查找标题index
            if !index_ok {
                debug!("row: {:#?}", row);
                name_index = row.iter().position(|x| {
                    // 在 x 之中查找是否包含name_keys所有的关键字其中某一个
                    name_keys
                        .iter()
                        .any(|key| x.is_string() && remove_whitespace(&x.to_string()).contains(key))
                });
                validity_index = row.iter().position(|x| {
                    validity_keys
                        .iter()
                        .any(|key| x.is_string() && remove_whitespace(&x.to_string()).contains(key))
                });
                count_index = row.iter().position(|x| {
                    count_keys
                        .iter()
                        .any(|key| x.is_string() && remove_whitespace(&x.to_string()).contains(key))
                });
                batch_number_index = row.iter().position(|x| {
                    batch_number_keys
                        .iter()
                        .any(|key| x.is_string() && remove_whitespace(&x.to_string()).contains(key))
                });

                debug!(
                    "name_index: {:?}, validity_index: {:?}, count_ndex: {:?}, batch_number_index: {:?}",
                    name_index, validity_index, count_index, batch_number_index
                );

                if name_index.is_some()
                    && validity_index.is_some()
                    && name_index.unwrap() != validity_index.unwrap()
                {
                    index_ok = true;
                    total_count -= 1;
                }
                continue;
            }

            // 获取数据
            if index_ok {
                let name = {
                    if row[name_index.unwrap()].is_string() {
                        row[name_index.unwrap()].to_string().trim().to_string()
                    } else {
                        "".to_string()
                    }
                };
                if name == "" {
                    warn!("skip because of name is empty: {:?}", row);
                    continue;
                }

                let validity = row[validity_index.unwrap()].as_datetime();
                if validity.is_none() {
                    warn!("skip because of validity is empty: {:?}", row);
                    continue;
                }
                let validity = validity.unwrap();

                let count = if count_index.is_some() {
                    row[count_index.unwrap()].to_string().trim().to_string()
                } else {
                    "Empty".to_string()
                };

                let batch_number = if batch_number_index.is_some() {
                    row[batch_number_index.unwrap()]
                        .to_string()
                        .trim()
                        .to_string()
                } else {
                    "Empty".to_string()
                };

                // count 和 batch_number 都可以为空

                debug!(
                    "name: {}, count: {}, validity: {:?}, batch_number: {}",
                    name, count, validity, batch_number
                );

                let medicinal = CreateMedicinal {
                    name,
                    count,
                    validity: validity.format("%Y%m%d").to_string(),
                    batch_number,
                    category: category.clone().unwrap_or("Empty".to_string()),
                };
                debug!("medicinal: {:#?}", medicinal);
                success_count += 1;
                result_content.push(medicinal);
            }
        }
        // 另外还有一种情况是没有批号，那么就没有批号字段，所以这里要单独处理一下
    }

    Ok((result_content, total_count, success_count))
}

fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_excel_file() {
        tracing_subscriber::fmt::init();
        // let file = "./upload/dd.xlsx";
        let file = "./upload/wc.xls";
        load_excel_file(file).await.unwrap();
    }

    #[test]
    fn test_trim_str() {
        let s = "  hello  ";
        assert_eq!(s.trim(), "hello");
        let s = " hello world ";
        assert_eq!(s.trim(), "hello world");

        let s = " hello world ";
        let replace_all = remove_whitespace(s);
        assert_eq!("helloworld", replace_all);

        let s = " 药  品 ";
        let replace_all = remove_whitespace(s);
        assert_eq!("药品", replace_all);
    }
}
