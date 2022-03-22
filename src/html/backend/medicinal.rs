use crate::db::pagination::Pagination;
use crate::{arg, model};
use askama::Template;

#[derive(Template)]
#[template(path = "backend/medicinal/index.html")]
pub struct IndexTemplate {
    pub arg: arg::MedicinalBackendQueryArg,
    pub list: Pagination<Vec<model::MedicinalList>>,
}

#[derive(Template)]
#[template(path = "backend/medicinal/add.html")]
pub struct AddTemplate {}
