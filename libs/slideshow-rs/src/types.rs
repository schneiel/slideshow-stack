use sdl3_rs::Texture2D;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ScalingMode {
    #[default]
    FitToScreen = 1,
    None = 0,
    FillToScreen = 2,
    StretchToFit = 3,
}

impl ScalingMode {
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::FitToScreen => Self::None,
            Self::None => Self::FillToScreen,
            Self::FillToScreen => Self::StretchToFit,
            Self::StretchToFit => Self::FitToScreen,
        }
    }

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::FitToScreen => "fit",
            Self::FillToScreen => "fill",
            Self::StretchToFit => "stretch",
        }
    }
}

impl From<i32> for ScalingMode {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::None,
            2 => Self::FillToScreen,
            3 => Self::StretchToFit,
            _ => Self::FitToScreen,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Stopped = 0,
    Playing = 1,
    Paused = 2,
}

impl Status {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Playing => "playing",
            Self::Paused => "paused",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SlideshowConfig {
    pub max_images: usize,
    pub max_cached_images: usize,
    pub queue_capacity: usize,
    pub max_gif_frames: usize,
    pub max_path_length: usize,
    pub max_slideshow_name_length: usize,
    pub default_gif_delay_ms: u32,
    pub default_interval_seconds: f64,
    pub target_fps: i32,
    pub max_gif_atlas_bytes: usize,
    pub max_single_image_bytes: usize,
    pub max_image_dimension: i32,
    pub max_failed_paths: usize,
    pub enable_debug_overlay: bool,
}

impl Default for SlideshowConfig {
    fn default() -> Self {
        Self {
            max_images: 200,
            max_cached_images: 10,
            queue_capacity: 32,
            max_gif_frames: 32,
            max_path_length: 512,
            max_slideshow_name_length: 128,
            target_fps: 30,
            default_gif_delay_ms: 100,
            default_interval_seconds: 5.0,
            max_gif_atlas_bytes: 100_000_000,
            max_single_image_bytes: 100_000_000,
            max_image_dimension: 4096,
            max_failed_paths: 20,
            enable_debug_overlay: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SlideshowStateData {
    pub timestamp: i64,
    pub status: Status,
    pub name: String,
    pub current_image: String,
    pub current_index: usize,
    pub total_images: usize,
    pub interval_seconds: f64,
    pub scaling_mode: ScalingMode,
    pub reason: String,
}

impl Default for SlideshowStateData {
    fn default() -> Self {
        Self {
            timestamp: 0,
            status: Status::Stopped,
            name: String::new(),
            current_image: String::new(),
            current_index: 0,
            total_images: 0,
            interval_seconds: 5.0,
            scaling_mode: ScalingMode::FitToScreen,
            reason: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub path: String,
    pub texture: Texture2D,
    pub width: i32,
    pub height: i32,
    pub is_gif: bool,
    pub current_frame: usize,
    pub frame_count: usize,
    pub frame_delays_ms: Vec<u32>,
    pub last_frame_time: f64,
    pub frame_data: Option<Vec<u8>>,
}