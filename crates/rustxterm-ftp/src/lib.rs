//! # rustxterm-ftp
//!
//! FTP and FTPS protocol handler for RustXterm.
//!
//! This crate provides:
//! - FTP client connection management via `suppaftp`
//! - FTPS (FTP over TLS) support
//! - File upload, download, and directory listing
//! - Transfer progress tracking and resumption

pub mod client;
pub mod transfer;

mod error;

pub use error::FtpError;
