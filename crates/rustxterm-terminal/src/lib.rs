//! # rustxterm-terminal
//!
//! Terminal emulation and PTY (pseudo-terminal) handling for RustXterm.
//!
//! This crate provides:
//! - PTY allocation and management via `portable-pty`
//! - Terminal state parsing and emulation via `vte`
//! - Input/output stream handling for terminal sessions
//! - Terminal resize and signal forwarding

pub mod emulator;
pub mod pty;

mod error;

pub use error::TerminalError;
