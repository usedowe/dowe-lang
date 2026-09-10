async fn send_web_push(
    config: &WebPushConfig,
    installation: &Installation,
    payload: &NotificationPayload,
) -> Result<(), DispatchError> {
    let subscription: WebPushSubscription =
        serde_json::from_str(&installation.token).map_err(|error| {
            DispatchError::InvalidToken(format!("invalid Web Push subscription: {error}"))
        })?;
    let p256dh = URL_SAFE_NO_PAD
        .decode(subscription.keys.p256dh.as_bytes())
        .map_err(|error| {
            DispatchError::InvalidToken(format!("invalid Web Push p256dh: {error}"))
        })?;
    let auth = URL_SAFE_NO_PAD
        .decode(subscription.keys.auth.as_bytes())
        .map_err(|error| DispatchError::InvalidToken(format!("invalid Web Push auth: {error}")))?;
    if auth.len() != 16 {
        return Err(DispatchError::InvalidToken(
            "invalid Web Push auth length".into(),
        ));
    }
    let public_key = PublicKey::from_sec1_bytes(&p256dh).map_err(|error| {
        DispatchError::InvalidToken(format!("invalid Web Push public key: {error}"))
    })?;
    let auth = Auth::clone_from_slice(&auth);
    let private_key = URL_SAFE_NO_PAD
        .decode(config.vapid_private_key.as_bytes())
        .map_err(|error| DispatchError::Retry(format!("invalid VAPID key: {error}")))?;
    let key_pair = ES256KeyPair::from_bytes(&private_key)
        .map_err(|error| DispatchError::Retry(format!("invalid VAPID key: {error}")))?;
    let body =
        serde_json::to_vec(payload).map_err(|error| DispatchError::Retry(error.to_string()))?;
    let ttl = payload
        .expires_at
        .map(|expires| expires.saturating_sub(now_seconds()).min(u32::MAX as u64) as u32)
        .unwrap_or(86_400);
    let message = WebPushBuilder::new(
        subscription.endpoint.parse().map_err(|error| {
            DispatchError::InvalidToken(format!("invalid Web Push endpoint: {error}"))
        })?,
        public_key,
        auth,
    )
    .with_valid_duration(std::time::Duration::from_secs(u64::from(ttl.max(1))))
    .with_vapid(&key_pair, &config.vapid_subject)
    .build(body)
    .map_err(|error| DispatchError::Retry(error.to_string()))?;
    let mut request = Client::new().post(message.uri().to_string());
    for (name, value) in message.headers() {
        let value = value
            .to_str()
            .map_err(|error| DispatchError::Retry(format!("Web Push header: {error}")))?;
        request = request.header(name.as_str(), value);
    }
    let response = request
        .body(message.into_body())
        .send()
        .await
        .map_err(|error| DispatchError::Retry(format!("Web Push request: {error}")))?;
    classify_http_response(response, "Web Push").await
}

#[derive(Debug, Deserialize)]
struct WebPushSubscription {
    endpoint: String,
    keys: WebPushSubscriptionKeys,
}

#[derive(Debug, Deserialize)]
struct WebPushSubscriptionKeys {
    p256dh: String,
    auth: String,
}

#[derive(Serialize)]
struct ApnsClaims<'a> {
    iss: &'a str,
    iat: u64,
}

async fn send_apns(
    config: &ApnsConfig,
    installation: &Installation,
    payload: &NotificationPayload,
) -> Result<(), DispatchError> {
    let header = Header {
        alg: Algorithm::ES256,
        kid: Some(config.key_id.clone()),
        ..Header::default()
    };
    let token = encode(
        &header,
        &ApnsClaims {
            iss: &config.team_id,
            iat: now_seconds(),
        },
        &EncodingKey::from_ec_pem(config.private_key.as_bytes())
            .map_err(|error| DispatchError::Retry(format!("invalid APNs key: {error}")))?,
    )
    .map_err(|error| DispatchError::Retry(format!("APNs JWT: {error}")))?;
    let body = json!({
        "aps": {
            "alert": {"title": payload.title, "body": payload.body},
            "category": payload.category,
            "thread-id": payload.tag,
            "sound": "default"
        },
        "dowe": payload,
    });
    let endpoint = format!(
        "{}/3/device/{}",
        config.endpoint.trim_end_matches('/'),
        installation.token
    );
    let response = Client::new()
        .post(endpoint)
        .header("authorization", format!("bearer {token}"))
        .header("apns-topic", &config.topic)
        .header("apns-push-type", "alert")
        .header("apns-priority", "10")
        .json(&body)
        .send()
        .await
        .map_err(|error| DispatchError::Retry(format!("APNs request: {error}")))?;
    classify_http_response(response, "APNs").await
}

#[derive(Serialize)]
struct ServiceAccountClaims<'a> {
    iss: &'a str,
    scope: &'a str,
    aud: &'a str,
    iat: u64,
    exp: u64,
}

#[derive(Deserialize)]
struct OAuthResponse {
    access_token: String,
}

async fn send_fcm(
    config: &FcmConfig,
    installation: &Installation,
    payload: &NotificationPayload,
) -> Result<(), DispatchError> {
    let now = now_seconds();
    let jwt = encode(
        &Header::new(Algorithm::RS256),
        &ServiceAccountClaims {
            iss: &config.client_email,
            scope: "https://www.googleapis.com/auth/firebase.messaging",
            aud: "https://oauth2.googleapis.com/token",
            iat: now,
            exp: now.saturating_add(3600),
        },
        &EncodingKey::from_rsa_pem(config.private_key.as_bytes())
            .map_err(|error| DispatchError::Retry(format!("invalid FCM key: {error}")))?,
    )
    .map_err(|error| DispatchError::Retry(format!("FCM JWT: {error}")))?;
    let token = Client::new()
        .post("https://oauth2.googleapis.com/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(form_body(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", jwt.as_str()),
        ]))
        .send()
        .await
        .map_err(|error| DispatchError::Retry(format!("FCM OAuth: {error}")))?
        .error_for_status()
        .map_err(|error| DispatchError::Retry(format!("FCM OAuth status: {error}")))?
        .json::<OAuthResponse>()
        .await
        .map_err(|error| DispatchError::Retry(format!("FCM OAuth response: {error}")))?;
    let endpoint = format!(
        "{}/v1/projects/{}/messages:send",
        config.endpoint.trim_end_matches('/'),
        config.project_id
    );
    let body = json!({
        "message": {
            "token": installation.token,
            "notification": {"title": payload.title, "body": payload.body},
            "data": {
                "dowe": serde_json::to_string(payload).map_err(|error| DispatchError::Retry(error.to_string()))?
            }
        }
    });
    let response = Client::new()
        .post(endpoint)
        .bearer_auth(token.access_token)
        .json(&body)
        .send()
        .await
        .map_err(|error| DispatchError::Retry(format!("FCM request: {error}")))?;
    classify_http_response(response, "FCM").await
}

#[derive(Deserialize)]
struct WnsOAuthResponse {
    access_token: String,
}

async fn send_wns(
    config: &WnsConfig,
    installation: &Installation,
    payload: &NotificationPayload,
) -> Result<(), DispatchError> {
    let token_url = format!(
        "{}/{}/oauth2/v2.0/token",
        config.endpoint.trim_end_matches('/'),
        config.tenant_id
    );
    let access = Client::new()
        .post(token_url)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(form_body(&[
            ("client_id", config.client_id.as_str()),
            ("client_secret", config.client_secret.as_str()),
            ("grant_type", "client_credentials"),
            ("scope", "https://wns.windows.com/.default"),
        ]))
        .send()
        .await
        .map_err(|error| DispatchError::Retry(format!("WNS OAuth: {error}")))?
        .error_for_status()
        .map_err(|error| DispatchError::Retry(format!("WNS OAuth status: {error}")))?
        .json::<WnsOAuthResponse>()
        .await
        .map_err(|error| DispatchError::Retry(format!("WNS OAuth response: {error}")))?;
    let escaped_title = xml_escape(&payload.title);
    let escaped_body = xml_escape(&payload.body);
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><toast><visual><binding template=\"ToastGeneric\"><text>{escaped_title}</text><text>{escaped_body}</text></binding></visual></toast>"
    );
    let response = Client::new()
        .post(&installation.token)
        .bearer_auth(access.access_token)
        .header("Content-Type", "text/xml")
        .header("X-WNS-Type", "wns/toast")
        .header("X-WNS-RequestForStatus", "true")
        .body(xml)
        .send()
        .await
        .map_err(|error| DispatchError::Retry(format!("WNS request: {error}")))?;
    classify_http_response(response, "WNS").await
}

async fn classify_http_response(
    response: reqwest::Response,
    provider: &str,
) -> Result<(), DispatchError> {
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }
    let body = response.text().await.unwrap_or_default();
    let error = format!("{provider} returned {status}: {}", truncate(&body, 512));
    if matches!(status, StatusCode::GONE | StatusCode::NOT_FOUND)
        || status == StatusCode::BAD_REQUEST
            && (body.contains("UNREGISTERED")
                || body.contains("BadDeviceToken")
                || body.contains("Unregistered"))
    {
        Err(DispatchError::InvalidToken(error))
    } else {
        Err(DispatchError::Retry(error))
    }
}


