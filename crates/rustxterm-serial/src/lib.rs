//! # rustxterm-serial
//!
//! Serial port handler for RustXterm.
//!
//! This crate provides:
//! - Serial port enumeration and connection management
//! - Baud rate, parity, and flow control configuration
//! - Async read/write over serial connections
//! - DTR/RTS signal control

pub mod port;
pub mod config;

mod error;

pub use error::SerialError;
