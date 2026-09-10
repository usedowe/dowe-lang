impl NotificationStore {
    pub fn open(root: impl AsRef<Path>, namespace: &str) -> NotificationResult<Self> {
        validate_text("namespace", namespace, MAX_NAMESPACE_BYTES)?;
        let base = root.as_ref().join(".dowe").join("notifications");
        fs::create_dir_all(&base).map_err(storage_error)?;
        let path = base.join(format!("{namespace}.json"));
        if !path.exists() {
            atomic_write(
                &path,
                br#"{"installations":[],"intents":[],"deliveries":[]}"#,
            )?;
        }
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn register(&self, mut installation: Installation) -> NotificationResult<Installation> {
        installation.validate()?;
        let mut state = self.read()?;
        if let Some(existing) = state
            .installations
            .iter_mut()
            .find(|item| item.id == installation.id)
        {
            if existing.app != installation.app
                || existing.environment != installation.environment
                || existing.tenant != installation.tenant
                || existing.user != installation.user
            {
                return Err(NotificationError::Unauthorized);
            }
            installation.registration_version = existing.registration_version.saturating_add(1);
            *existing = installation.clone();
        } else {
            if installation.registration_version == 0 {
                installation.registration_version = 1;
            }
            state.installations.push(installation.clone());
        }
        self.write(&state)?;
        Ok(installation)
    }

    pub fn revoke(&self, installation_id: &str) -> NotificationResult<()> {
        let mut state = self.read()?;
        let Some(installation) = state
            .installations
            .iter_mut()
            .find(|item| item.id == installation_id)
        else {
            return Err(NotificationError::NotFound);
        };
        installation.active = false;
        installation.registration_version = installation.registration_version.saturating_add(1);
        for delivery in &mut state.deliveries {
            if delivery.installation_id == installation_id
                && matches!(
                    delivery.status,
                    DeliveryStatus::Queued | DeliveryStatus::RetryScheduled
                )
            {
                delivery.status = DeliveryStatus::Revoked;
            }
        }
        self.write(&state)
    }

    /// Return an installation snapshot for a claimed delivery.
    pub fn installation(&self, installation_id: &str) -> NotificationResult<Installation> {
        self.read()?
            .installations
            .into_iter()
            .find(|item| item.id == installation_id)
            .ok_or(NotificationError::NotFound)
    }

    /// Disable an installation after a provider reports that its token is gone.
    pub fn revoke_delivery_installation(&self, delivery_id: &str) -> NotificationResult<()> {
        let mut state = self.read()?;
        let installation_id = state
            .deliveries
            .iter()
            .find(|delivery| delivery.id == delivery_id)
            .map(|delivery| delivery.installation_id.clone())
            .ok_or(NotificationError::NotFound)?;
        if let Some(installation) = state
            .installations
            .iter_mut()
            .find(|installation| installation.id == installation_id)
        {
            installation.active = false;
            installation.registration_version = installation.registration_version.saturating_add(1);
        }
        if let Some(delivery) = state
            .deliveries
            .iter_mut()
            .find(|delivery| delivery.id == delivery_id)
        {
            delivery.status = DeliveryStatus::Revoked;
            delivery.lease_until = None;
            delivery.last_error = Some("provider_token_invalid".to_string());
        }
        self.write(&state)
    }

    pub fn enqueue(
        &self,
        idempotency_key: &str,
        app: &str,
        environment: &str,
        tenant: &str,
        mut payload: NotificationPayload,
        user: &str,
    ) -> NotificationResult<NotificationIntent> {
        payload.validate()?;
        validate_text("idempotency_key", idempotency_key, 256)?;
        let mut state = self.read()?;
        if let Some(existing) = state
            .intents
            .iter()
            .find(|item| item.idempotency_key == idempotency_key)
        {
            let mut comparable = payload.clone();
            if comparable.expires_at.is_none() {
                comparable.expires_at = existing.payload.expires_at;
            }
            if existing.payload != comparable
                || existing.app != app
                || existing.environment != environment
                || existing.tenant != tenant
            {
                return Err(NotificationError::IdempotencyConflict);
            }
            return Ok(existing.clone());
        }
        let now = now_seconds();
        let expires_at = payload.expires_at.unwrap_or(now.saturating_add(86_400));
        payload.expires_at = Some(expires_at);
        let id = stable_id(idempotency_key, &payload)?;
        let mut delivery_ids = Vec::new();
        for installation in state.installations.iter().filter(|item| {
            item.app == app
                && item.environment == environment
                && item.tenant == tenant
                && item.user == user
                && item.accepts(&payload.category)
        }) {
            let delivery_id = format!(
                "{id}:{}:{}",
                installation.id, installation.registration_version
            );
            delivery_ids.push(delivery_id.clone());
            state.deliveries.push(Delivery {
                id: delivery_id,
                notification_id: id.clone(),
                installation_id: installation.id.clone(),
                registration_version: installation.registration_version,
                payload: payload.clone(),
                status: DeliveryStatus::Queued,
                attempts: 0,
                next_attempt_at: now,
                lease_until: None,
                provider_id: None,
                last_error: None,
            });
        }
        let intent = NotificationIntent {
            id,
            idempotency_key: idempotency_key.to_string(),
            app: app.to_string(),
            environment: environment.to_string(),
            tenant: tenant.to_string(),
            payload,
            created_at: now,
            expires_at,
            deliveries: delivery_ids,
        };
        state.intents.push(intent.clone());
        self.write(&state)?;
        Ok(intent)
    }

    pub fn claim(
        &self,
        worker: &str,
        now: u64,
        lease_seconds: u64,
    ) -> NotificationResult<Option<Delivery>> {
        validate_text("worker", worker, 128)?;
        let mut state = self.read()?;
        let mut expired_any = false;
        for delivery in &mut state.deliveries {
            if matches!(
                delivery.status,
                DeliveryStatus::Queued | DeliveryStatus::RetryScheduled
            ) && delivery
                .payload
                .expires_at
                .is_some_and(|until| until <= now)
            {
                delivery.status = DeliveryStatus::Expired;
                delivery.lease_until = None;
                expired_any = true;
            }
        }
        let Some(index) = state.deliveries.iter().position(|delivery| {
            matches!(
                delivery.status,
                DeliveryStatus::Queued | DeliveryStatus::RetryScheduled
            ) && delivery.next_attempt_at <= now
                && delivery.lease_until.is_none_or(|until| until <= now)
                && delivery.payload.expires_at.is_none_or(|until| until > now)
        }) else {
            if expired_any {
                self.write(&state)?;
            }
            return Ok(None);
        };
        let delivery = &mut state.deliveries[index];
        delivery.status = DeliveryStatus::Attempting;
        delivery.attempts = delivery.attempts.saturating_add(1);
        delivery.lease_until = Some(now.saturating_add(lease_seconds));
        delivery.last_error = Some(format!("lease:{worker}"));
        let claimed = delivery.clone();
        self.write(&state)?;
        Ok(Some(claimed))
    }

    pub fn mark_provider_accepted(
        &self,
        delivery_id: &str,
        provider_id: &str,
    ) -> NotificationResult<()> {
        let mut state = self.read()?;
        let delivery = state
            .deliveries
            .iter_mut()
            .find(|item| item.id == delivery_id)
            .ok_or(NotificationError::NotFound)?;
        delivery.status = DeliveryStatus::ProviderAccepted;
        delivery.provider_id = Some(provider_id.to_string());
        delivery.lease_until = None;
        delivery.last_error = None;
        self.write(&state)
    }

    pub fn mark_retry(&self, delivery_id: &str, error: &str, now: u64) -> NotificationResult<()> {
        let mut state = self.read()?;
        let delivery = state
            .deliveries
            .iter_mut()
            .find(|item| item.id == delivery_id)
            .ok_or(NotificationError::NotFound)?;
        delivery.status = if delivery.attempts >= 10 {
            DeliveryStatus::Failed
        } else {
            DeliveryStatus::RetryScheduled
        };
        delivery.next_attempt_at = now.saturating_add(retry_delay(delivery.attempts));
        delivery.lease_until = None;
        delivery.last_error = Some(error.to_string());
        self.write(&state)
    }

    pub fn deliveries(&self) -> NotificationResult<Vec<Delivery>> {
        Ok(self.read()?.deliveries)
    }

    fn read(&self) -> NotificationResult<PersistedState> {
        let bytes = fs::read(&self.path).map_err(storage_error)?;
        serde_json::from_slice(&bytes).map_err(storage_error)
    }

    fn write(&self, state: &PersistedState) -> NotificationResult<()> {
        let bytes = serde_json::to_vec_pretty(state).map_err(storage_error)?;
        atomic_write(&self.path, &bytes)
    }
}

