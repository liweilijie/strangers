use axum::extract::Extension;
use axum::http::StatusCode;
use axum::routing::{get, get_service};
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use strangers::handler::{auth, backend};
use strangers::model::AppState;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tracing::{debug, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "strangers=debug".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenv::dotenv().ok();
    let cfg = strangers::config::Config::from_env().unwrap();
    let pool = cfg.pg.create_pool(None, tokio_postgres::NoTls).unwrap();
    let rdc = redis::Client::open(&*cfg.redis.dsn).unwrap(); // TODO: 为什么是 &*

    debug!("cfg: {:#?}", cfg);
    info!("web server listening on http://{}", &cfg.web.addr);

    let state = Arc::new(AppState {
        pool,
        rdc,
        sess_cfg: cfg.session,
        hcap_cfg: cfg.hcaptcha,
    });

    let backend_router = backend::routers();
    let static_serve = get_service(ServeDir::new("static")).handle_error(|err| async move {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("载入静态资源出错: {}", err),
        )
    });

    let app = Router::new()
        .nest("/static", static_serve)
        .nest("/admin", backend_router)
        .route("/login", get(auth::admin_login_ui).post(auth::admin_login))
        .layer(ServiceBuilder::new().layer(Extension(state)));

    axum::Server::bind(&cfg.web.addr.parse::<SocketAddr>().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
