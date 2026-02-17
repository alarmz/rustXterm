//! # rustxterm-telnet
//!
//! Telnet protocol handler for RustXterm.
//!
//! This crate provides:
//! - Telnet client connection management
//! - Telnet option negotiation (WILL, WONT, DO, DONT)
//! - Terminal type and window size negotiation
//! - Raw and line-mode input handling

pub mod client;
pub mod codec;
pub mod negotiation;

mod error;

pub use error::TelnetError;
