//! # rustxterm-session
//!
//! Session management and persistence for RustXterm.
//!
//! This crate provides:
//! - Session lifecycle management (create, save, restore, delete)
//! - SQLite-backed session persistence
//! - Session grouping and organization
//! - Connection profile storage and retrieval

pub mod manager;
pub mod persistence;
pub mod profile;

mod error;

pub use error::SessionError;
