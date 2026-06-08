mod auth;
mod codec;
mod engine;
mod error;
mod names;
mod remote;

pub use auth::{CreatedKvUser, create_user, verify_user};
pub use engine::{
    KvDatabase, KvInspection, KvSetReport, clear_memory, init_database, kv_root, list_databases,
    open_database,
};
pub use error::{KvError, KvResult};
pub use remote::{
    KvServerConfig, RemoteKvClient, RemoteKvConfig, RemoteKvRequest, RunningKvServer,
    serve_kv_server, start_kv_server,
};

#[cfg(test)]
mod tests;
