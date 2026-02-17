//! # rustxterm-plugin
//!
//! Plugin system framework for RustXterm.
//!
//! This crate provides:
//! - Dynamic plugin loading via `libloading`
//! - Plugin trait definitions and lifecycle management
//! - Plugin discovery and registration
//! - Inter-plugin communication and event hooks

pub mod loader;
pub mod registry;
pub mod api;

mod error;

pub use error::PluginError;
