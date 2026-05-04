//! Slideshow Module
//!
//! Encapsulated module for slideshow management.
//! Provides functions for creating, reading, updating, and deleting slideshows.

pub mod service;
pub mod types;

pub use service::{
    create_slideshow, delete_slideshow, get_slideshow, list_slideshows, update_slideshow,
};
