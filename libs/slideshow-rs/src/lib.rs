#![allow(
    clippy::struct_excessive_bools,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::zero_sized_map_values,
    clippy::significant_drop_tightening,
    clippy::significant_drop_in_scrutinee,
    clippy::map_unwrap_or,
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    clippy::elidable_lifetime_names,
)]

pub mod types;
pub mod slideshow;
pub mod image_loader;
pub mod command;

pub use types::*;
pub use slideshow::{Slideshow, SlideshowError};
pub use command::{Command, CommandQueue};
