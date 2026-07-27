mod auth;
mod client;
mod engine;
mod error;
mod names;
mod protocol;
mod service;

pub use auth::{CreatedVectorAccount, create_account, verify_account};
pub use client::{DoweVectorClient, DoweVectorConfig, close_remote_connections};
pub use engine::{
    Embedding, VectorDatabase, VectorInspection, VectorMatch, VectorUpsert, init_database,
    list_databases, open_database, vector_root,
};
pub use error::{VectorError, VectorResult};
pub use protocol::{VectorRequest, VectorResponse};
pub use service::{
    RunningVectorServer, VectorServerConfig, serve_vector_server, start_vector_server,
    vector_service_upgrade,
};

#[cfg(test)]
mod tests;
