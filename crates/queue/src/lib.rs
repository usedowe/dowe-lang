mod auth;
mod client;
mod engine;
mod error;
mod model;
mod names;
mod protocol;
mod rabbitmq;
mod remote_subscription;
mod service;
mod storage;
mod topic;
mod transport;
mod vercel;

pub use auth::{CreatedQueueAccount, create_account, verify_account};
pub use client::{QueueClient, QueueSubscription};
pub use engine::{
    DoweQueue, DoweSubscription, init_namespace, list_namespaces, open_namespace, queue_root,
};
pub use error::{QueueError, QueueResult};
pub use model::{
    BindReport, DeclareReport, DirectPublishReport, PublishReport, PurgeReport, QueueConfig,
    QueueDelivery, QueueInspection, QueueInspectionEntry, QueueMessage, QueueProvider,
};
pub use protocol::{QueueRequest, QueueResponse, QueueResponseError};
pub use rabbitmq::{rabbitmq_exchange_name, rabbitmq_queue_name};
pub use service::{
    QueueServerConfig, RunningQueueServer, queue_service_router, queue_service_upgrade,
    serve_queue_server, start_queue_server,
};
pub use topic::{topic_matches, validate_pattern, validate_topic};

#[cfg(test)]
mod auth_lock_tests;
#[cfg(test)]
mod direct_publish_tests;
#[cfg(test)]
mod lock_tests;
#[cfg(test)]
mod retry_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod transport_tests;
