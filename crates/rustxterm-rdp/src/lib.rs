//! # rustxterm-rdp
//!
//! RDP (Remote Desktop Protocol) handler for RustXterm.
//!
//! This crate provides:
//! - RDP client connection management
//! - Display and input channel handling
//! - Clipboard and drive redirection support
//!
//! **Note:** IronRDP integration planned for full RDP protocol support.

pub mod client;
pub mod display;

mod error;

pub use error::RdpError;
