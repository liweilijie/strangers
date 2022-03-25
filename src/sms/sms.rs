use crate::db::medicinal;
use crate::handler::helper::get_client;
use crate::model::{AppState, MedicinalList};
use crate::sms::send::NotifySms;
use crate::Result;
use std::sync::Arc;
use tokio::time;
use tokio::time::{Duration, Instant};
use tracing::{debug, error, warn};

const EXPIRED_DAYS: i64 = 30;
const INTERVAL_SECONDS: u64 = 120;

// 利用tokio后台启动一个定期任务
// 定期扫描medicinal表之中没有被删除,并且已经过期或者即将过期的药品,并且发送短信通知
// 定期扫描,每天发送短信,每天早上9:30做一次
pub async fn do_work(state: Arc<AppState>, notify_state: Arc<NotifySms>) -> Result<()> {
    let client = get_client(&state, "定时任务:短信发送模块").await?;
    // 查询已经过期的药品,马上发送短信
    let condition = format!(
        "is_del=false AND validity <= '{}'",
        chrono::Local::now().format("%Y-%m-%d").to_string()
    );
    let expired_medicinal = medicinal::all(&client, &condition, &[]).await?;
    for item in &expired_medicinal {
        warn!("药品已经过期,发送短信通知: {}", item);
        send(item, notify_state.clone(), "已经过期药品").await;
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
        "is_del=false AND validity BETWEEN '{}' AND '{}'",
        chrono::Local::now().format("%Y-%m-%d").to_string(),
        chrono::Local::now()
            .checked_add_signed(chrono::Duration::days(expired_days))
            .unwrap()
            .format("%Y-%m-%d")
            .to_string(),
    );
    let expired_medicinal = medicinal::all(&client, &condition, &[]).await?;
    for item in &expired_medicinal {
        warn!("药品即将({}天)过期,发送短信通知: {}", expired_days, item);
        send(
            item,
            notify_state.clone(),
            &format!("即将过期({}天)药品", expired_days),
        )
        .await;
    }
    Ok(())
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
    let mut intv = time::interval_at(start, interval);
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
