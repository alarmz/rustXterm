//! # rustxterm-serial
//!
//! Serial port handler for RustXterm.
//!
//! This crate provides:
//! - Serial port enumeration and connection management
//! - Baud rate, parity, and flow control configuration
//! - Async read/write over serial connections
//! - DTR/RTS signal control

pub mod config;
pub mod port;

mod error;

pub use error::SerialError;
