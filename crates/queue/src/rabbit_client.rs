use crate::error::{QueueError, QueueResult};
use crate::model::{
    BindReport, DeclareReport, DeliveryReceipt, DirectPublishReport, PublishReport, PurgeReport,
    QueueConfig, QueueDelivery, QueueInspection, QueueMessage, delivery,
};
use crate::names::{validate_consumer_name, validate_queue_name};
use crate::storage::timestamp;
use crate::topic::{validate_pattern, validate_topic};
use dowe_id::generate_ulid;
use futures_util::StreamExt;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicPublishOptions, BasicQosOptions,
    ConfirmSelectOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
    QueuePurgeOptions,
};
use lapin::types::FieldTable;
use lapin::{
    Acker, BasicProperties, Confirmation, Connection, ConnectionProperties, Consumer, ExchangeKind,
};
use serde_json::Value;
use std::net::IpAddr;

pub(crate) const RABBIT_PREFETCH: u16 = 1;

#[derive(Clone)]
pub struct RabbitClient {
    config: QueueConfig,
}

pub struct RabbitSubscription {
    _connection: Connection,
    consumer: Consumer,
    closed: bool,
}

struct RabbitReceipt {
    acker: Acker,
    resolved: bool,
}

