use crate::db::medicinal;
use crate::error::{AppError, AppErrorType};
use crate::form::CreateMedicinal;
use crate::handler::helper::{get_client, log_error, render};
use crate::handler::redirect::redirect;
use crate::html::backend::medicinal::{AddTemplate, EditTemplate, IndexTemplate, UploadTemplate};
use crate::model::{get_expired_str, AppState, MedicinalList};
use crate::{arg, form, Result};
use axum::body::StreamBody;
use axum::extract::{ContentLengthLimit, Extension, Form, Multipart, Path, Query};
use axum::http::{header, StatusCode};
use axum::response::{Headers, Html};
use calamine::DataType::Empty;
use calamine::{open_workbook, open_workbook_auto, Reader, Xls, Xlsx};
use chrono::{Datelike, Local};
use csv::StringRecord;
use dateparser::DateTimeUtc;
use encoding_rs::{Encoder, Encoding, GBK, UTF_8};
use reqwest::header::HeaderMap;
use std::fmt::format;
use std::path::PathBuf;
use std::sync::Arc;
use tokio_util::io::ReaderStream;

use tracing::field::debug;
use tracing::{debug, error, info, warn};

/// 允许上传的文件大小
const MAX_UPLOAD_SIZE: u64 = 1024 * 1024 * 256; // 256 MB
const DEFAULT_VALIDITY_DATE: &str = "2099-12-31";

pub async fn download(
    Extension(state): Extension<Arc<AppState>>,
    args: Option<Query<arg::MedicinalBackendQueryArg>>,
) -> Result<(StatusCode, HeaderMap, ())> {
    let handler_name = "download";
    let client = get_client(&state, handler_name).await?;

    let result = medicinal::all(&client, &format!("is_del=false"), &[])
        .await
        .map_err(log_error(handler_name.to_string()))?;
    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(&result);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = StreamBody::new(stream);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "text/toml; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=medicinal.toml".parse().unwrap(),
    );
    headers.insert(
        header::LOCATION,
        "/admin/medicinal?msg=导出成功!".parse().unwrap(),
    );

    // let headers = Headers([
    //     (header::CONTENT_TYPE, "text/toml; charset=utf-8"),
    //     (
    //         header::CONTENT_DISPOSITION,
    //         "attachment; filename=\"Cargo.toml\"",
    //     ),
    //     (header::LOCATION, "/admin/medicinal?msg=导出成功!"),
    // ]);

    Ok((StatusCode::FOUND, headers, ()))
}

pub async fn index(
    Extension(state): Extension<Arc<AppState>>,
    args: Option<Query<arg::MedicinalBackendQueryArg>>,
) -> Result<Html<String>> {
    debug!("args: {:#?}", args);
    let handler_name = "backend_medicinal_index";
    let client = get_client(&state, handler_name).await?;
    let args = args.unwrap().0;
    let q_keyword = format!("%{}%", args.keyword());

    // 处理category 查询
    let q_category = format!("%{}%", args.category());

    let q_expired = args.expired();
    // 处理查询日期(月份)
    let q_expired_result = if q_expired > 0 {
        if q_expired == 1 {
            // 表示已经过期的数据
            format!(
                " AND validity <= '{}'",
                chrono::Local::now().format("%Y-%m-%d").to_string(),
            )
        } else {
            // 表示between 当前时间到 (q_expired-1)*30 天之间的数据
            format!(
                " AND validity BETWEEN '{}' AND '{}'",
                chrono::Local::now().format("%Y-%m-%d").to_string(),
                chrono::Local::now()
                    .checked_add_signed(chrono::Duration::days((q_expired - 1) as i64 * 30))
                    .unwrap()
                    .format("%Y-%m-%d")
                    .to_string(),
            )
        }
    } else {
        "".to_string()
    };

    // let q_order = args.order();
    // let q_order_result = if !&q_order.is_empty() {
    //     format!(" ORDER BY {}", q_order)
    // } else {
    //     "".to_string()
    // };

    debug!("q_expired_result: {}", q_expired_result);
    // debug!("q_order_result: {}", q_order_result);
    let condition = format!(
        "is_del=$1 AND name LIKE $2 AND category LIKE $3 {}",
        q_expired_result
    );

    let medicinal_list = medicinal::select(
        &client,
        // "is_del=$1 AND name LIKE $2",
        &condition,
        &[&args.is_del(), &q_keyword, &q_category],
        args.page.unwrap_or(0),
    )
    .await
    .map_err(log_error(handler_name.to_string()))?;

    let categories = medicinal::categories(&client)
        .await
        .map_err(log_error(handler_name.to_string()))?;

    let tmpl = IndexTemplate {
        arg: args,
        list: medicinal_list,
        categories,
        expired_items: get_expired_str(),
    };
    render(tmpl, handler_name)
}

// 添加药品页
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

// 编辑药品
pub async fn edit(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Html<String>> {
    debug!("edit id: {}", id);
    let handler_name = "backend_medicinal_edit";
    let client = get_client(&state, handler_name).await?;
    debug!("start to find id.");
    let med = medicinal::find(&client, Some("id=$1"), &[&id])
        .await
        .map_err(log_error(handler_name.to_string()))?;
    let tmpl = EditTemplate { medicinal: med };
    render(tmpl, handler_name)
}

// 编辑药品操作
pub async fn edit_action(
    Extension(state): Extension<Arc<AppState>>,
    form: Form<form::UpdateMedicinal>,
) -> Result<(StatusCode, HeaderMap, ())> {
    let handler_name = "backend_medicinal_edit_action";
    let client = get_client(&state, handler_name).await?;
    debug!("edit_action and form: {:#?}", form);
    medicinal::update(&client, &form)
        .await
        .map_err(log_error(handler_name.to_string()))?;

    redirect("/admin/medicinal?msg=药品编辑成功")
}

// 删除药品
pub async fn del(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<(StatusCode, HeaderMap, ())> {
    let handler_name = "backend_medicinal_del";
    let client = get_client(&state, handler_name).await?;
    medicinal::delete(&client, id)
        .await
        .map_err(log_error(handler_name.to_string()))?;
    redirect("/admin/medicinal?msg=药品删除成功")
}

// 恢复某个药品
pub async fn recover(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<(StatusCode, HeaderMap, ())> {
    let handler_name = "backend_medicinal_recover";
    let client = get_client(&state, handler_name).await?;
    medicinal::recover(&client, id)
        .await
        .map_err(log_error(handler_name.to_string()))?;
    redirect("/admin/medicinal?msg=药品恢复成功")
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
            Some("csv") => (),
            // Some("csv") | Some("xlsx") | Some("xlsm") | Some("xlsb") | Some("xls") => (),
            _ => {
                return Err(AppError::from_str(
                    "上传文件格式错误(建议将文件另存为csv,上传csv格式)",
                    AppErrorType::UploadError,
                ))
            }
        }

        let data = file.bytes().await.unwrap(); // 上传的文件内容

        debug!("upload_action and filename: {:#?}", filename);
        debug!("upload_action and data size: {}", data.len());

        // 文件创建以时间为前缀
        let to_path = format!(
            "{}/{}-{}",
            &state.upload_dir,
            format!(
                "upload_{}",
                chrono::Local::now().format("%Y-%m-%d_%H:%M:%S")
            ),
            filename
        );

        debug!("upload_action and to_path: {}", to_path);

        // 保存上传的文件
        tokio::fs::write(&to_path, &data)
            .await
            .map_err(|err| AppError::from_err(err, AppErrorType::UploadError))?;

        let (result, total_count, _success_count) = match sc.extension().and_then(|s| s.to_str()) {
            // 如果是csv文件,则解析csv文件
            Some("csv") => load_csv_file(&to_path).await.map_err(|err| {
                error!("{} 文件内容读取失败, error: {:?}", &to_path, err);
                AppError::from_err(err, AppErrorType::UploadError)
            })?,
            // 如果是excel文件,则读取 excel内容
            // _ => load_excel_file(&to_path).await.map_err(|err| {
            //     error!("{} 文件内容读取失败, error: {:?}", &to_path, err);
            //     AppError::from_err(err, AppErrorType::UploadError)
            // })?,
            _ => {
                return Err(AppError::from_str(
                    "上传文件格式错误(建议将文件另存为csv,上传csv格式)",
                    AppErrorType::UploadError,
                ))
            }
        };

        debug!("读取文件条数: {}", result.len());

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

fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

async fn load_csv_file(file: &str) -> Result<(Vec<CreateMedicinal>, u32, u32)> {
    info!("load csv file: {}", file);
    let encoding_from = Encoding::for_label("gbk".as_bytes()).unwrap_or(GBK);
    let encoding_to = Encoding::for_label("utf8".as_bytes()).unwrap_or(UTF_8);
    convert_encoding(file, encoding_from, encoding_to);
    info!("convert_encoding done");
    // 读取规则
    // 第一行是类目
    // 后面找到index 才能正常读取.
    // let mut rdr = csv::Reader::from_path(file).unwrap();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b',')
        .from_path(file)?;

    let mut category: Option<String> = None;
    let name_keys = vec!["药品", "名称", "药名", "项目", "型号"];
    let validity_keys = vec!["有效期", "有效期至", "效期", "有效"];
    let count_keys = vec!["数量", "基数", "数"];
    let batch_number_keys = vec!["批号", "生产日期", "生产"];
    let spec_keys = vec!["规格"];
    let skip_keys = vec!["签名"]; // 如果有内容包含签名的，则不进行处理与判断

    let mut name_index: Option<usize> = None;
    let mut count_index: Option<usize> = None;
    let mut validity_index: Option<usize> = None;
    let mut batch_number_index: Option<usize> = None;
    let mut spec_index: Option<usize> = None;
    let mut index_ok = false;
    let mut result_content: Vec<CreateMedicinal> = vec![];
    let mut success_count: u32 = 0;
    let mut total_count: u32 = 0;

    for row in rdr.records() {
        debug!("row: {:#?}", row);
        // 最正常的情况下有四列，第一列是药品名称， 第二列是批号，第三列是数量，第四列是有效期 当然顺序还可能不一样，所以要处理顺序
        // 说明有批号
        // 查找 category
        // 至少需要2列才正常, 一列是名字，一列是有效期
        if row.is_err() {
            warn!("row is err: {:?}", row);
            continue;
        }

        let row: StringRecord = row.unwrap();
        if row.len() < 2 {
            error!("excel column too small row.len < 2");
            continue;
        }

        // 所有行的内容都为空则跳过
        // all() 接受一个返回 true 或 false 的闭包。它将这个闭包应用于迭代器的每个元素，如果它们都返回 true，那么 all() 也返回。 如果它们中的任何一个返回 false，则返回 false。
        // all() 短路; 换句话说，它一旦找到 false 就会停止处理，因为无论发生什么，结果也将是 false。
        // 空的迭代器将返回 true。
        if row
            .iter()
            .all(|x| x.is_empty() || remove_whitespace(&x).is_empty())
        {
            continue;
        }

        // any() 短路; 换句话说，它一旦找到 true 就会停止处理，因为无论发生什么，结果也将是 true。
        // 如果任意一行有如下内容的则返回不计算
        // 所有字段都是空白的，则直接跳过
        if row.iter().any(|x| {
            skip_keys
                .iter()
                .any(|key| !x.is_empty() && remove_whitespace(&x.to_string()).contains(key))
        }) {
            warn!("找到过滤的行，所以此行不进行处理直接跳过。{:#?}", row);
            continue;
        }

        total_count += 1;

        if category.is_none() {
            // 判断是否是第一行为类目
            // 所有数据其他都为 Empty, 只有一个值是 String 类型的则为category
            if !row[0].is_empty() {
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
                    total_count -= 1; // 不算在总数里面
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
                    .any(|key| !x.is_empty() && remove_whitespace(&x.to_string()).contains(key))
            });
            validity_index = row.iter().position(|x| {
                validity_keys
                    .iter()
                    .any(|key| !x.is_empty() && remove_whitespace(&x.to_string()).contains(key))
            });
            count_index = row.iter().position(|x| {
                count_keys
                    .iter()
                    .any(|key| !x.is_empty() && remove_whitespace(&x.to_string()).contains(key))
            });
            batch_number_index = row.iter().position(|x| {
                batch_number_keys
                    .iter()
                    .any(|key| !x.is_empty() && remove_whitespace(&x.to_string()).contains(key))
            });

            spec_index = row.iter().position(|x| {
                spec_keys
                    .iter()
                    .any(|key| !x.is_empty() && remove_whitespace(&x.to_string()).contains(key))
            });

            debug!(
                    "name_index: {:?}, validity_index: {:?}, count_ndex: {:?}, batch_number_index: {:?}, spec_index:{:?}",
                    name_index, validity_index, count_index, batch_number_index, spec_index,
                );

            if name_index.is_some()
                && validity_index.is_some()
                && name_index.unwrap() != validity_index.unwrap()
            {
                index_ok = true;
                total_count -= 1; // 不算在总数里面
            }
            continue;
        }

        // 获取数据
        if index_ok {
            let name = {
                if !row[name_index.unwrap()].is_empty() {
                    row[name_index.unwrap()].to_string().trim().to_string()
                } else {
                    "".to_string()
                }
            };
            if name == "" {
                warn!("skip because of name is empty: {:?}", row);
                continue;
            }

            let mut validity = remove_whitespace(&row[validity_index.unwrap()])
                .parse::<DateTimeUtc>()
                .ok();

            if validity.is_none() {
                if remove_whitespace(&row[validity_index.unwrap()]) == "无".to_string() {
                    // 结定一个默认的日期
                    validity = Some(DEFAULT_VALIDITY_DATE.parse::<DateTimeUtc>().unwrap());
                    warn!("匹配到 无，所以无效期默认为 {}", DEFAULT_VALIDITY_DATE);
                } else {
                    warn!("skip because of validity is empty: {:?}", row);
                    continue;
                }
            }

            let validity = validity.unwrap().0; // 只需要日期不需要时间
            let validity = validity.with_timezone(&Local);
            debug!("local validity: {:?}", validity);

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

            let spec = if spec_index.is_some() {
                row[spec_index.unwrap()].to_string().trim().to_string()
            } else {
                "Empty".to_string()
            };

            // count 和 batch_number 都可以为空

            debug!(
                "name: {}, count: {}, validity: {:?}, batch_number: {}, spec: {}",
                name, count, validity, batch_number, spec
            );

            let batch_number = {
                if batch_number == "" {
                    "Empty".to_string()
                } else {
                    batch_number
                }
            };

            let count = {
                if count == "" {
                    "Empty".to_string()
                } else {
                    count
                }
            };

            let medicinal = CreateMedicinal {
                name,
                count,
                validity: chrono::NaiveDate::from_ymd(
                    validity.year(),
                    validity.month(),
                    validity.day(),
                ),
                batch_number,
                spec,
                category: category.clone().unwrap_or("Empty".to_string()),
            };
            debug!("medicinal: {:#?}", medicinal);
            success_count += 1;
            result_content.push(medicinal);
        }
    }

    debug!(
        "total_count: {}, success_count: {}",
        total_count, success_count
    );
    Ok((result_content, total_count, success_count))
}

fn convert_encoding(file: &str, encoding_from: &'static Encoding, encoding_to: &'static Encoding) {
    debug!("convert_encoding file: {}", file);
    match std::fs::read(file) {
        Ok(bytes) => {
            let (string, encoding, has_malformed) = encoding_from.decode(&bytes);
            if encoding != encoding_from {
                println!("^^^^Detected encoding is {}", encoding.name());
            }
            if has_malformed {
                println!("^^^^There are malformed characters");
            } else {
                let (bytes, encoding, has_unmappable) = encoding_to.encode(&string);
                if encoding != encoding_to {
                    println!("^^^^Saved encoding is {}", encoding.name());
                }
                if has_unmappable {
                    println!("^^^^There are unmappable characters");
                }
                std::fs::write(file, bytes).unwrap_or_else(|err| {
                    println!("^^^^write error: {}", err);
                });
            }
        }
        Err(err) => {
            println!("^^^^read error: {}", err);
        }
    }
}

mod tests {
    use super::*;
    use csv::ByteRecord;
    use tracing::info;

    #[tokio::test]
    async fn test_load_excel_file() {
        tracing_subscriber::fmt::init();
        // let file = "./upload/dd.xlsx";
        let file = "./upload/up.xls";
        // load_excel_file(file).await.unwrap();
    }

    #[tokio::test]
    async fn test_load_csv_file() {
        tracing_subscriber::fmt::init();
        let file = "./upload/ts2.csv";
        load_csv_file(file).await.unwrap();
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

    #[test]
    fn test_get_date() {
        let s = "2022-03-24";
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert_eq!(date, s);

        let s = "2022-04-23";
        // 增加一个月时间
        let date = chrono::Local::now()
            .checked_add_signed(chrono::Duration::days(30))
            .unwrap()
            .format("%Y-%m-%d")
            .to_string();
        println!("{}", date);
        assert_eq!(date, s);
    }

    #[test]
    fn test_get_date_by_csv() {
        tracing_subscriber::fmt::init();
        let s = "2024年8月";
        info!("{}", s);
        let parsed = s.parse::<dateparser::DateTimeUtc>().unwrap().0;
        info!("{:#?}", parsed);
        let parsed = parsed.with_timezone(&Local);
        // let parsed = dateparser::parse_with_timezone(s, &Local).unwrap();
        // let parsed = chrono::NaiveDate::parse_from_str(s, "%Y年%m月").unwrap();
        info!("{:#?}", parsed);

        let s = "2023-04";
        info!("{}", s);
        let parsed = s.parse::<dateparser::DateTimeUtc>().unwrap().0;
        info!("{:#?}", parsed);
        let parsed = parsed.with_timezone(&Local);
        // let parsed = dateparser::parse_with_timezone(s, &Local).unwrap();
        // let parsed = chrono::NaiveDate::parse_from_str(s, "%Y年%m月").unwrap();
        info!("{:#?}", parsed);

        let parsed = DEFAULT_VALIDITY_DATE
            .parse::<dateparser::DateTimeUtc>()
            .unwrap()
            .0;
        info!("default validity date: {:#?}", parsed);
    }

    #[test]
    fn test_remove_whitespace() {
        // StringRecord(["外科手套", "2副", "      2023-04", ""])

        tracing_subscriber::fmt::init();
        let byte_record = ByteRecord::from(vec!["外科手套", "2副", "      2023-04", ""]);
        let str_record = StringRecord::from_byte_record(byte_record).unwrap();
        str_record.iter().for_each(|s| {
            info!("[{}]", remove_whitespace(s));
        });
    }
}

// async fn load_excel_file(file: &str) -> Result<(Vec<CreateMedicinal>, u32, u32)> {
//     // 判断文件类型是否合法
//     let sce = PathBuf::from(file);
//     // TODO:
//     match sce.extension().and_then(|s| s.to_str()) {
//         Some("xlsx") | Some("xlsm") | Some("xlsb") | Some("xls") => (),
//         _ => {
//             return Err(AppError::from_str(
//                 "excel文件格式错误",
//                 AppErrorType::UploadError,
//             ))
//         }
//     }
//     // let mut excel: Xlsx<_> = open_workbook(file)?;
//     let mut excel = open_workbook_auto(file)?;
//     let mut category: Option<String> = None;
//     let name_keys = vec!["药品", "名称", "药名", "项目", "型号"];
//     let validity_keys = vec!["有效期", "有效期至", "效期", "有效"];
//     let count_keys = vec!["数量", "基数", "数"];
//     let batch_number_keys = vec!["批号", "生产日期", "生产"];
//
//     let mut name_index: Option<usize> = None;
//     let mut count_index: Option<usize> = None;
//     let mut validity_index: Option<usize> = None;
//     let mut batch_number_index: Option<usize> = None;
//     let mut index_ok = false;
//     let mut result_content: Vec<CreateMedicinal> = vec![];
//     let mut success_count: u32 = 0;
//     let mut total_count: u32 = 0;
//
//     if let Some(Ok(r)) = excel.worksheet_range("Sheet1") {
//         for row in r.rows() {
//             total_count += 1;
//             // debug!("row: {:#?}", row);
//             // 最正常的情况下有四列，第一列是药品名称， 第二列是批号，第三列是数量，第四列是有效期 当然顺序还可能不一样，所以要处理顺序
//             // 说明有批号
//             // 查找 category
//             // 至少需要2列才正常, 一列是名字，一列是有效期
//             if row.len() < 2 {
//                 error!("excel column too small row.len < 2");
//                 continue;
//             }
//
//             if category.is_none() {
//                 // 判断是否是第一行为类目
//                 // 所有数据其他都为 Empty, 只有一个值是 String 类型的则为category
//                 if row[0].is_string() {
//                     let mut other_is_empty = true;
//                     for (index, val) in row.iter().enumerate() {
//                         if index == 0 {
//                             continue;
//                         } else {
//                             if !val.is_empty() {
//                                 other_is_empty = false;
//                             }
//                         }
//                     }
//                     if other_is_empty {
//                         category = Some(row[0].to_string().trim().to_string());
//                         debug!("category: {:?}", category);
//                         total_count -= 1;
//                         continue;
//                     }
//                 }
//             }
//
//             // 查找标题index
//             if !index_ok {
//                 debug!("row: {:#?}", row);
//                 name_index = row.iter().position(|x| {
//                     // 在 x 之中查找是否包含name_keys所有的关键字其中某一个
//                     name_keys
//                         .iter()
//                         .any(|key| x.is_string() && remove_whitespace(&x.to_string()).contains(key))
//                 });
//                 validity_index = row.iter().position(|x| {
//                     validity_keys
//                         .iter()
//                         .any(|key| x.is_string() && remove_whitespace(&x.to_string()).contains(key))
//                 });
//                 count_index = row.iter().position(|x| {
//                     count_keys
//                         .iter()
//                         .any(|key| x.is_string() && remove_whitespace(&x.to_string()).contains(key))
//                 });
//                 batch_number_index = row.iter().position(|x| {
//                     batch_number_keys
//                         .iter()
//                         .any(|key| x.is_string() && remove_whitespace(&x.to_string()).contains(key))
//                 });
//
//                 debug!(
//                     "name_index: {:?}, validity_index: {:?}, count_ndex: {:?}, batch_number_index: {:?}",
//                     name_index, validity_index, count_index, batch_number_index
//                 );
//
//                 if name_index.is_some()
//                     && validity_index.is_some()
//                     && name_index.unwrap() != validity_index.unwrap()
//                 {
//                     index_ok = true;
//                     total_count -= 1;
//                 }
//                 continue;
//             }
//
//             // 获取数据
//             if index_ok {
//                 let name = {
//                     if row[name_index.unwrap()].is_string() {
//                         row[name_index.unwrap()].to_string().trim().to_string()
//                     } else {
//                         "".to_string()
//                     }
//                 };
//                 if name == "" {
//                     warn!("skip because of name is empty: {:?}", row);
//                     continue;
//                 }
//
//                 let validity = row[validity_index.unwrap()].as_datetime();
//                 if validity.is_none() {
//                     warn!("skip because of validity is empty: {:?}", row);
//                     continue;
//                 }
//                 let validity = validity.unwrap().date(); // 只需要日期不需要时间
//
//                 let count = if count_index.is_some() {
//                     row[count_index.unwrap()].to_string().trim().to_string()
//                 } else {
//                     "Empty".to_string()
//                 };
//
//                 let batch_number = if batch_number_index.is_some() {
//                     row[batch_number_index.unwrap()]
//                         .to_string()
//                         .trim()
//                         .to_string()
//                 } else {
//                     "Empty".to_string()
//                 };
//
//                 // count 和 batch_number 都可以为空
//
//                 debug!(
//                     "name: {}, count: {}, validity: {:?}, batch_number: {}",
//                     name, count, validity, batch_number
//                 );
//
//                 let batch_number = {
//                     if batch_number == "" {
//                         "Empty".to_string()
//                     } else {
//                         batch_number
//                     }
//                 };
//
//                 let count = {
//                     if count == "" {
//                         "Empty".to_string()
//                     } else {
//                         count
//                     }
//                 };
//
//                 let medicinal = CreateMedicinal {
//                     name,
//                     count,
//                     validity,
//                     batch_number,
//                     category: category.clone().unwrap_or("Empty".to_string()),
//                 };
//                 debug!("medicinal: {:#?}", medicinal);
//                 success_count += 1;
//                 result_content.push(medicinal);
//             }
//         }
//         // 另外还有一种情况是没有批号，那么就没有批号字段，所以这里要单独处理一下
//     }
//
//     Ok((result_content, total_count, success_count))
// }
