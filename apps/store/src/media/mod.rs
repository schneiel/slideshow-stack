//! Media Module
//!
//! Encapsulated module for media file management.
//! Provides functions for listing, uploading, and deleting media files.
//!
//! # Supported Formats
//!
//! - Images: PNG, JPG, JPEG, GIF
//! - Videos: MP4

pub mod service;
pub mod storage;
pub mod types;

pub use service::{delete_media, get_media_path, list_media, upload_media};
