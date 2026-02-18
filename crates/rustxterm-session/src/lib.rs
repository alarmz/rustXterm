pub mod manager;
pub mod persistence;

mod error;

pub use error::SessionError;
pub use manager::SessionManager;
pub use persistence::TunnelConfig;

// Re-export core session types for convenience.
pub use rustxterm_core::session::{SessionConfig, SessionInfo};
