use crate::db::medicinal;
use crate::handler::helper::get_client;
use crate::model::{AppState, MedicinalList};
use crate::sms::send::NotifySms;
use crate::Result;
use std::sync::Arc;
use tokio::time;
use tokio::time::{Duration, Instant};
use tokio_postgres::Client;
use tracing::{debug, error, warn};

const EXPIRED_DAYS: i64 = 30;
const INTERVAL_SECONDS: u64 = 1200;
const INTERVAL_SCHEDULE_SECONDS: u64 = 120;

// 利用tokio后台启动一个定期任务
// 定期扫描medicinal表之中没有被删除,并且已经过期或者即将过期的药品,并且发送短信通知
// 定期扫描,每天发送短信,每天早上9:30做一次
pub async fn do_work(state: Arc<AppState>, notify_state: Arc<NotifySms>) -> Result<()> {
    let client = get_client(&state, "定时任务:短信发送模块").await?;
    // 查询已经过期的药品,马上发送短信
    let condition = format!(
        "is_del=false AND validity <= '{}' AND notify_at <= '{}'",
        chrono::Local::now().format("%Y-%m-%d").to_string(),
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    );
    debug!("condition: {}", condition);
    let expired_medicinal = medicinal::all(&client, &condition, &[]).await?;
    let mut ids = Vec::new();
    for item in &expired_medicinal {
        warn!("药品已经过期,发送短信通知: {}", item);
        send(item, notify_state.clone(), "已经过期药品").await;
        ids.push(item.id.to_string());
    }

    if ids.len() > 0 {
        // 更新notify_at时间
        match update(&client, ids).await {
            Ok(_) => {}
            Err(e) => {
                error!("更新notify_at时间失败: {}", e);
            }
        }
    }

    // 查询即将过期的药品
    let expired_days = **&state
        .sms_cfg
        .as_ref()
        .unwrap()
        .expired_days
        .as_ref()
        .unwrap_or(&EXPIRED_DAYS);
    let condition = format!(
        "is_del=false AND notify_at <= '{}' AND validity BETWEEN '{}' AND '{}'",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        chrono::Local::now().format("%Y-%m-%d").to_string(),
        chrono::Local::now()
            .checked_add_signed(chrono::Duration::days(expired_days))
            .unwrap()
            .format("%Y-%m-%d")
            .to_string(),
    );
    let expired_medicinal = medicinal::all(&client, &condition, &[]).await?;
    let mut ids = Vec::new();
    for item in &expired_medicinal {
        warn!("药品即将({}天)过期,发送短信通知: {}", expired_days, item);
        send(
            item,
            notify_state.clone(),
            &format!("即将过期({}天)药品", expired_days),
        )
        .await;
        ids.push(item.id.to_string());
    }

    if ids.len() > 0 {
        // 更新notify_at时间
        match update(&client, ids).await {
            Ok(_) => {}
            Err(e) => {
                error!("更新notify_at时间失败: {}", e);
            }
        }
    }

    Ok(())
}

async fn update(client: &Client, ids: Vec<String>) -> Result<bool> {
    // 更新通知时间
    // 24小时后的时间
    let after24hours = chrono::Local::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .unwrap_or(chrono::Local::now());
    // 打印时区
    let condition = format!("id in ({})", ids.join(","));
    let update_data = format!("notify_at='{}'", after24hours.to_rfc3339());
    debug!("condition: {}, update_data: {}", condition, update_data);
    medicinal::update_notify_at(&client, &condition, &update_data).await
}

pub async fn sms_schedule(state: Arc<AppState>, notify_state: Arc<NotifySms>) {
    // 从现在开始, 每一个小时做一次任务
    // https://rust-book.junmajinlong.com/ch100/03_use_tokio_time.html
    if *&state.sms_cfg.as_ref().is_none()
        || !*&state
            .sms_cfg
            .as_ref()
            .unwrap()
            .send_sms_toggle
            .unwrap_or(false)
    {
        warn!("短信配置未配置,不启动定时任务");
        return;
    }
    let start = Instant::now();
    let interval = Duration::from_secs(
        *&state
            .sms_cfg
            .as_ref()
            .unwrap()
            .check_interval
            .unwrap_or(INTERVAL_SECONDS),
    );
    debug!("sms_schedule interval: {}s", &interval.as_secs());
    // let mut intv = time::interval_at(start, interval);
    let mut intv = time::interval_at(start, Duration::from_secs(INTERVAL_SCHEDULE_SECONDS));
    loop {
        let _ = do_work(state.clone(), notify_state.clone()).await;
        intv.tick().await;
    }
}

async fn send(item: &MedicinalList, notify_state: Arc<NotifySms>, msg: &str) {
    // 构建发送短信的内容
    let p = aliyun_sdk::SmsParam {
        name: format!(
            "{}:{}-{}-{}",
            msg, &item.name, item.batch_number, item.category
        ),
        code: format!("有效期:{}", item.validity.format("%Y-%m-%d")),
    };

    let sms = aliyun_sdk::SmsRequest {
        phones: notify_state.phones.clone(),
        sign_name: notify_state.sign_name.clone(),
        template_code: notify_state.template_code.clone(),
        out_id: Some("123".to_string()),
        param: p,
    };

    notify_state
        .tx
        .clone()
        .send(sms)
        .map_err(|e| {
            error!("发送短信失败: {}", e);
        })
        .ok();
}

mod tests {

    #[test]
    fn test_time_format_with_zone() {
        let now = chrono::Local::now();
        let now_str = now.to_rfc3339(); // 2022-03-29T14:30:37.386513327+08:00
        println!("now_str: {}", now_str);
    }
}
