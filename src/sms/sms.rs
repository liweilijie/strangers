use crate::db::medicinal;
use crate::handler::helper::get_client;
use crate::model::AppState;
use crate::Result;
use std::sync::Arc;
use tokio::time;
use tokio::time::{Duration, Instant};
use tracing::warn;

const EXPIRED_DAYS: i64 = 30;
const INTERVAL_SECONDS: u64 = 120;

// 利用tokio后台启动一个定期任务
// 定期扫描medicinal表之中没有被删除,并且已经过期或者即将过期的药品,并且发送短信通知
// 定期扫描,每天发送短信,每天早上9:30做一次
pub async fn do_work(state: Arc<AppState>) -> Result<()> {
    let client = get_client(&state, "定时任务:短信发送模块").await?;
    // 查询已经过期的药品,马上发送短信
    let condition = format!(
        "is_del=false AND validity <= '{}'",
        chrono::Local::now().format("%Y-%m-%d").to_string()
    );
    let expired_medicinal = medicinal::all(&client, &condition, &[]).await?;
    for item in &expired_medicinal {
        warn!("药品已经过期,发送短信通知: {}", item);
    }

    // 查询即将过期的药品
    let condition = format!(
        "is_del=false AND validity BETWEEN '{}' AND '{}'",
        chrono::Local::now().format("%Y-%m-%d").to_string(),
        chrono::Local::now()
            .checked_add_signed(chrono::Duration::days(EXPIRED_DAYS))
            .unwrap()
            .format("%Y-%m-%d")
            .to_string(),
    );
    let expired_medicinal = medicinal::all(&client, &condition, &[]).await?;
    for item in &expired_medicinal {
        warn!("药品即将({}天)过期,发送短信通知: {}", EXPIRED_DAYS, item);
    }
    Ok(())
}

pub async fn sms_schedule(state: Arc<AppState>) {
    // 从现在开始, 每一个小时做一次任务
    // https://rust-book.junmajinlong.com/ch100/03_use_tokio_time.html
    let start = Instant::now();
    let interval = Duration::from_secs(INTERVAL_SECONDS);
    let mut intv = time::interval_at(start, interval);
    loop {
        let _ = do_work(state.clone()).await;
        intv.tick().await;
    }
}
