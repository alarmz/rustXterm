//! # rustxterm-tunnel
//!
//! SSH tunnel and port forwarding for RustXterm.
//!
//! This crate provides:
//! - Local port forwarding (SSH -L equivalent)
//! - Remote port forwarding (SSH -R equivalent)
//! - Dynamic SOCKS proxy forwarding (SSH -D equivalent)
//! - Tunnel lifecycle management and monitoring

pub mod dynamic;
pub mod local_forward;
pub mod manager;
pub mod remote_forward;

mod error;

pub use error::TunnelError;
pub use manager::{TunnelInfo, TunnelManager, TunnelType};
