use crate::db::pagination::Pagination;
use crate::model::{Category, ExpiredItem};
use crate::{arg, model};
use askama::Template;

#[derive(Template)]
#[template(path = "backend/medicinal/index.html")]
pub struct IndexTemplate {
    pub arg: arg::MedicinalBackendQueryArg,
    pub list: Pagination<Vec<model::MedicinalList>>,
    pub categories: Vec<Category>,       // 分类信息
    pub expired_items: Vec<ExpiredItem>, // 查询过期条件信息
}

#[derive(Template)]
#[template(path = "backend/medicinal/add.html")]
pub struct AddTemplate {}

#[derive(Template)]
#[template(path = "backend/medicinal/upload.html")]
pub struct UploadTemplate {}

#[derive(Template)]
#[template(path = "backend/medicinal/edit.html")]
pub struct EditTemplate {
    pub medicinal: model::MedicinalList,
}
