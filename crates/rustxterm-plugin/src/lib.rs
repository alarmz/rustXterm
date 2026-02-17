//! # rustxterm-plugin
//!
//! Plugin system framework for RustXterm.
//!
//! This crate provides:
//! - Dynamic plugin loading via `libloading`
//! - Plugin trait definitions and lifecycle management
//! - Plugin discovery and registration
//! - Inter-plugin communication and event hooks

pub mod api;
pub mod loader;
pub mod registry;

mod error;

pub use error::PluginError;
