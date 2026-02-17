//! # rustxterm-credentials
//!
//! Credential and password management for RustXterm.
//!
//! This crate provides:
//! - Encrypted credential storage using AES-GCM
//! - Master password derivation via PBKDF2
//! - OS keyring integration for secure key storage
//! - SQLite-backed credential database
//! - SSH key management and passphrase handling

pub mod store;
pub mod crypto;
pub mod keyring_backend;
pub mod database;

mod error;

pub use error::CredentialError;
