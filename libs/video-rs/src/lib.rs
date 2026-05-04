#![allow(
    clippy::while_float,
    clippy::cast_precision_loss,
)]

pub mod player;
pub mod path_utils;

pub use player::{VideoPlayer, VideoError, Command, Status, VideoStateData, ScalingMode};
pub use path_utils::{PathError, validate_filename, validate_filenames, sanitize_and_join, sanitize_and_join_all, is_video_file};
