use std::sync::Arc;
use tracing::{debug, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use strangers::model::AppState;

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
    let rdc = redis::Client::open(cfg.redis.dsn).unwrap();

    debug!("cfg: {:#?}", cfg);
    info!("web server listening on http://{}", &cfg.web.addr);

    let state = Arc::new(AppState {
        pool,
        rdc,
        sess_cfg: cfg.session,
        hcap_cfg: cfg.hcaptcha,
    });

}