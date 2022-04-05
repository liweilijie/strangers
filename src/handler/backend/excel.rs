use crate::error::{AppError, AppErrorType};
use crate::model::MedicinalList;
use crate::Result;
use chrono::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::fs::remove_file;
use tracing::{debug, error};
use uuid::Uuid;
use xlsxwriter::{DateTime as XLSDateTime, Format, Workbook, Worksheet};

const FONT_SIZE: f64 = 12.0;

pub fn create_xlsx_for_medicinal(values: &Vec<MedicinalList>) -> Result<String> {
    let uuid = Uuid::new_v4().to_string();
    // now()_uuid.xlsx
    let file_name = format!(
        "{}_{}.xlsx",
        chrono::Local::now().format("%Y-%m-%d-%H-%M-%S"),
        uuid
    );
    debug!("file_name: {}", file_name);
    let workbook = Workbook::new(&file_name);

    let mut sheet = workbook.add_worksheet(None).map_err(|e| {
        error!("add_worksheet failed: {:?}", e);
        AppError::from_err(e, AppErrorType::ExcelError)
    })?;

    let mut width_map: HashMap<u16, usize> = HashMap::new();
    create_headers(&mut sheet, &mut width_map);

    let fmt = workbook
        .add_format()
        .set_text_wrap()
        .set_font_size(FONT_SIZE);

    let date_fmt = workbook
        .add_format()
        .set_num_format("yyyy-mm-dd")
        .set_font_size(FONT_SIZE);

    for (i, v) in values.iter().enumerate() {
        add_row(i as u32, &v, &mut sheet, &date_fmt, &mut width_map);
    }

    width_map.iter().for_each(|(k, v)| {
        let _ = sheet.set_column(*k as u16, *k as u16, *v as f64 * 1.2, Some(&fmt));
    });

    workbook.close().expect("workbook can be closed.");

    // let result = fs::read(&file_name).map_err(|e| {
    //     error!("read failed: {:?}", e);
    //     AppError::from_err(e, AppErrorType::ExcelError)
    // })?;
    // remove_file(&file_name).expect("can remove file");
    Ok(file_name)
}

fn add_row(
    row: u32,
    med: &MedicinalList,
    sheet: &mut Worksheet,
    date_fmt: &Format,
    width_map: &mut HashMap<u16, usize>,
) {
    add_string_column(row, 0, &med.category, sheet, width_map);
    add_string_column(row, 1, &med.name, sheet, width_map);
    add_string_column(row, 2, &med.batch_number, sheet, width_map);
    add_string_column(row, 3, &med.spec, sheet, width_map);
    add_string_column(row, 4, &med.count, sheet, width_map);
    // NaiveDate to DateTime<Local>

    let dt: DateTime<Local> = Local
        .ymd(
            med.validity.year(),
            med.validity.month(),
            med.validity.day(),
        )
        .and_hms(0, 0, 0);
    add_date_column(row, 5, &dt, sheet, width_map, date_fmt);
}

fn add_string_column(
    row: u32,
    column: u16,
    data: &str,
    sheet: &mut Worksheet,
    mut width_map: &mut HashMap<u16, usize>,
) {
    let _ = sheet.write_string(row + 1, column, data, None);
    set_new_max_with(column, data.len(), &mut width_map);
}

fn add_date_column(
    row: u32,
    column: u16,
    date: &DateTime<Local>,
    sheet: &mut Worksheet,
    mut width_map: &mut HashMap<u16, usize>,
    date_fmt: &Format,
) {
    let d = XLSDateTime::new(
        date.year() as i16,
        date.month() as i8,
        date.day() as i8,
        date.hour() as i8,
        date.minute() as i8,
        date.second() as f64,
    );
    let _ = sheet.write_datetime(row + 1, column, &d, Some(date_fmt));
    set_new_max_with(column, 11, &mut width_map);
}

pub fn create_headers(sheet: &mut Worksheet, mut width_map: &mut HashMap<u16, usize>) {
    let _ = sheet.write_string(0, 0, "药箱", None);
    let _ = sheet.write_string(0, 1, "药品名称", None);
    let _ = sheet.write_string(0, 2, "批号", None);
    let _ = sheet.write_string(0, 3, "规格", None);
    let _ = sheet.write_string(0, 4, "数量", None);
    let _ = sheet.write_string(0, 5, "有效期", None);

    set_new_max_with(0, "药箱".len(), &mut width_map);
    set_new_max_with(1, "药品名称".len(), &mut width_map);
    set_new_max_with(2, "批号".len(), &mut width_map);
    set_new_max_with(3, "规格".len(), &mut width_map);
    set_new_max_with(4, "数量".len(), &mut width_map);
    set_new_max_with(5, "有效期".len(), &mut width_map);
}

fn set_new_max_with(col: u16, new: usize, width_map: &mut HashMap<u16, usize>) {
    match width_map.get(&col) {
        Some(max) => {
            if new > *max {
                width_map.insert(col, new);
            }
        }
        None => {
            width_map.insert(col, new);
        }
    }
}
