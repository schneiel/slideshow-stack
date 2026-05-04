//! Store Library
//!
//! Rust implementation of the slideshow store API.
//!
//! # Modules
//!
//! - [`db`] - Database layer with `SQLite`
//! - [`slideshow`] - Slideshow business logic
//! - [`media`] - Media file management
//! - [`api`] - HTTP API layer
//! - [`config`] - Configuration management

pub mod api;
pub mod config;
pub mod db;
pub mod media;
pub mod slideshow;

pub use config::{load_config, Config};
