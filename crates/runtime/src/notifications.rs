use dowe_notifications::{
    Installation, NotificationIntent, NotificationPayload, NotificationResult, NotificationStore,
};
use std::path::Path;

/// Open the application's durable notification namespace.
pub fn open_notification_store(
    project_root: impl AsRef<Path>,
    namespace: &str,
) -> NotificationResult<NotificationStore> {
    NotificationStore::open(project_root, namespace)
}

/// Register or rotate one authenticated device installation.
pub fn register_notification_installation(
    store: &NotificationStore,
    subject: &str,
    installation: Installation,
) -> NotificationResult<Installation> {
    register_authenticated_notification_installation(store, subject, installation)
}

/// Register an installation after binding it to the authenticated request subject.
pub fn register_authenticated_notification_installation(
    store: &NotificationStore,
    subject: &str,
    installation: Installation,
) -> NotificationResult<Installation> {
    if subject.is_empty() || installation.user != subject {
        return Err(dowe_notifications::NotificationError::Unauthorized);
    }
    store.register(installation)
}

/// Create a durable notification intent and its per-installation deliveries.
pub fn enqueue_notification(
    store: &NotificationStore,
    idempotency_key: &str,
    app: &str,
    environment: &str,
    tenant: &str,
    user: &str,
    payload: NotificationPayload,
) -> NotificationResult<NotificationIntent> {
    store.enqueue(idempotency_key, app, environment, tenant, payload, user)
}
