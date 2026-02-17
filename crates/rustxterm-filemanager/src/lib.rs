//! # rustxterm-filemanager
//!
//! SFTP and local file browser for RustXterm.
//!
//! This crate provides:
//! - Unified file browsing interface for local and remote filesystems
//! - SFTP-based remote file operations via `rustxterm-ssh`
//! - File upload, download, rename, and delete operations
//! - Directory tree navigation and bookmarks

pub mod browser;
pub mod local;
pub mod remote;
pub mod operations;

mod error;

pub use error::FileManagerError;
