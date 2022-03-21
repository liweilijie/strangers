pub mod index;

use axum::routing::get;
use axum::Router;

pub fn routers() -> Router {
    Router::new().route("/", get(index::index))
}
