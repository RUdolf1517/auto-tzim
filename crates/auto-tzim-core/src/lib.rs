#[cfg(not(unix))]
compile_error!("auto-tzim currently supports Unix platforms only");

/// Version of the core library.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod config;

pub mod batch;
pub mod discovery;
pub mod hash;
pub mod planner;
pub mod stability;
