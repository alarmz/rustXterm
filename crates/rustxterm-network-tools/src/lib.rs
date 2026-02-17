//! # rustxterm-network-tools
//!
//! Network utilities for RustXterm.
//!
//! This crate provides:
//! - DNS resolution and lookup via `trust-dns-resolver`
//! - Ping and traceroute utilities
//! - Port scanning and connectivity checks
//! - Network interface enumeration

pub mod dns;
pub mod ping;
pub mod scan;

mod error;

pub use error::NetworkToolsError;
