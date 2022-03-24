use crate::arg;
use crate::db::pagination::Pagination;
use crate::model::Admin;
use askama::Template;

#[derive(Template)]
#[template(path = "backend/admin/index.html")]
pub struct IndexTemplate {
    pub list: Pagination<Vec<Admin>>,
    pub arg: arg::BackendQueryArg,
}

#[derive(Template)]
#[template(path = "backend/admin/add.html")]
pub struct AddTemplate {}
#[derive(Template)]
#[template(path = "backend/admin/edit.html")]
pub struct EditTemplate {
    pub admin: Admin,
}
