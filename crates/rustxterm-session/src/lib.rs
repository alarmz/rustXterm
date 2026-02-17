pub mod manager;
pub(crate) mod persistence;

mod error;

pub use error::SessionError;
pub use manager::SessionManager;

// Re-export core session types for convenience.
pub use rustxterm_core::session::{SessionConfig, SessionInfo};
