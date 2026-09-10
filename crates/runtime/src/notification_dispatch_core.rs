/// Dispatch up to `limit` currently eligible deliveries.
pub async fn dispatch_pending_notifications(
    store: &NotificationStore,
    config: &NotificationDispatcherConfig,
    worker: &str,
    limit: usize,
) -> NotificationDispatchReport {
    let mut report = NotificationDispatchReport::default();
    for _ in 0..limit {
        let claimed = match store.claim(worker, now_seconds(), 60) {
            Ok(Some(delivery)) => delivery,
            Ok(None) => break,
            Err(error) => {
                report.errors.push(error.to_string());
                break;
            }
        };
        let installation = match store.installation(&claimed.installation_id) {
            Ok(value) => value,
            Err(error) => {
                report.errors.push(error.to_string());
                let _ = store.mark_retry(&claimed.id, &error.to_string(), now_seconds());
                continue;
            }
        };
        match dispatch_delivery(config, &installation, &claimed).await {
            Ok(provider_id) => {
                let _ = store.mark_provider_accepted(&claimed.id, &provider_id);
                report.accepted += 1;
            }
            Err(error @ DispatchError::InvalidToken(_)) => {
                let _ = store.revoke_delivery_installation(&claimed.id);
                report.revoked += 1;
                report.errors.push(error.message());
            }
            Err(error @ DispatchError::Retry(_)) => {
                let _ = store.mark_retry(&claimed.id, &error.message(), now_seconds());
                report.retried += 1;
                report.errors.push(error.message());
            }
        }
    }
    report
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NotificationDispatchReport {
    pub accepted: usize,
    pub retried: usize,
    pub revoked: usize,
    pub errors: Vec<String>,
}

async fn dispatch_delivery(
    config: &NotificationDispatcherConfig,
    installation: &Installation,
    delivery: &Delivery,
) -> Result<String, DispatchError> {
    match installation.provider {
        NotificationProvider::WebPush => send_web_push(
            config.web_push.as_ref().ok_or_else(|| {
                DispatchError::Retry("web push credentials are not configured".into())
            })?,
            installation,
            &delivery.payload,
        )
        .await
        .map(|()| "web-push".to_string()),
        NotificationProvider::Apns => send_apns(
            config.apns.as_ref().ok_or_else(|| {
                DispatchError::Retry("APNs credentials are not configured".into())
            })?,
            installation,
            &delivery.payload,
        )
        .await
        .map(|()| "apns".to_string()),
        NotificationProvider::Fcm => send_fcm(
            config
                .fcm
                .as_ref()
                .ok_or_else(|| DispatchError::Retry("FCM credentials are not configured".into()))?,
            installation,
            &delivery.payload,
        )
        .await
        .map(|()| "fcm".to_string()),
        NotificationProvider::Wns => send_wns(
            config
                .wns
                .as_ref()
                .ok_or_else(|| DispatchError::Retry("WNS credentials are not configured".into()))?,
            installation,
            &delivery.payload,
        )
        .await
        .map(|()| "wns".to_string()),
        NotificationProvider::DoweLinux => send_linux_wss(
            config.linux.as_ref().unwrap_or(&LinuxDispatcherConfig {
                connect_timeout: Duration::from_secs(10),
            }),
            installation,
            delivery,
        )
        .await
        .map(|()| "dowe-linux-wss".to_string()),
    }
}


