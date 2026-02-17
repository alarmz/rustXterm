//! # rustxterm-vnc
//!
//! VNC (Virtual Network Computing) protocol handler for RustXterm.
//!
//! This crate provides:
//! - VNC client connection management
//! - RFB (Remote Framebuffer) protocol implementation
//! - Framebuffer rendering and input event forwarding
//! - VNC authentication support

pub mod client;
pub mod framebuffer;
pub mod input;

mod error;

pub use error::VncError;
