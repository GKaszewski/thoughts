pub mod adapter;
pub mod config;

pub use adapter::ObjectStorageAdapter;
pub use config::{build_store, StorageConfig};
