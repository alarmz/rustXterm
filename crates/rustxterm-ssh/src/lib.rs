//! # rustxterm-ssh
//!
//! SSH and SFTP protocol handler for RustXterm.
//!
//! This crate provides:
//! - SSH client connection management via `russh`
//! - Key-based and password authentication
//! - SFTP file transfer operations via `russh-sftp`
//! - SSH channel multiplexing and session handling

pub mod auth;
pub mod channel;
pub mod client;
pub mod sftp;

mod error;

pub use error::SshError;
