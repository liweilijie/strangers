use axum::extract::{extractor_middleware, Extension};
use axum::http::StatusCode;
use axum::routing::{get, get_service};
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use std::sync::Arc;
use strangers::handler::{auth, backend, home};
use strangers::middleware::admin_auth::Auth;
use strangers::model::AppState;
use strangers::sms::send::NotifySms;
use strangers::sms::sms;
use tower::ServiceBuilder;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use tower_http::services::ServeDir;
use tracing::{debug, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    // dotenv 放前面，因为配置文件里面有可能配置 `RUST_LOG` 信息，但是有可能会受全局变量配置的影响, whatever.
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "strangers=debug".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cfg = strangers::config::Config::from_env().unwrap();
    let pool = cfg.pg.create_pool(None, tokio_postgres::NoTls).unwrap();
    let rdc = redis::Client::open(&*cfg.redis.dsn).unwrap(); // TODO: 为什么是 &*

    debug!("cfg: {:#?}", cfg);

    let state = Arc::new(AppState {
        pool,
        rdc,
        sess_cfg: cfg.session,
        hcap_cfg: cfg.hcaptcha,
        sms_cfg: cfg.sms,
        upload_dir: cfg.upload_dir.unwrap_or("upload".to_string()),
    });

    // 初始化短信发送任务
    let notify_state = Arc::new(NotifySms::new());

    // 启动短信告警后台服务
    tokio::spawn(sms::sms_schedule(state.clone(), notify_state.clone()));

    let backend_router = backend::routers().layer(extractor_middleware::<Auth>());
    let static_serve = get_service(ServeDir::new("static")).handle_error(|err| async move {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("载入静态资源出错: {}", err),
        )
    });

    // 注册所有路由
    let app = Router::new()
        .nest("/static", static_serve)
        .nest("/admin", backend_router)
        .route("/login", get(auth::admin_login_ui).post(auth::admin_login))
        .route("/logout", get(auth::admin_logout))
        .route("/", get(home::admin_index))
        .layer(ServiceBuilder::new().layer(Extension(state)))
        .layer(CookieManagerLayer::new());

    // 启动服务
    if !&cfg.ssl_enable {
        info!("web server listening on http://{}", &cfg.web.addr);
        axum::Server::bind(&cfg.web.addr.parse::<SocketAddr>().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    } else {
        // 启动 https 服务
        info!("web server listening on https://{}", &cfg.web.addr);
        let ssl_cfg = RustlsConfig::from_pem_file("certs/emacsvi.com.cer", "certs/emacsvi.com.key")
            .await
            .unwrap();
        axum_server::bind_rustls(cfg.web.addr.parse::<SocketAddr>().unwrap(), ssl_cfg)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}
