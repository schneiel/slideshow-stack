use sdl3_rs::{Renderer, Vector2, Color, Rectangle, Texture2D, Image, PIXELFORMAT_UNCOMPRESSED_R8G8B8A8};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use thiserror::Error;
use ffmpeg_rs::VideoDecoder;
use serde::{Serialize, Deserialize};

#[derive(Error, Debug)]
pub enum VideoError {
    #[error("Video load failed: {0}")]
    LoadFailed(String),
    #[error("Decode error: {0}")]
    DecodeError(String),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Idle = 0,
    Playing = 1,
    Paused = 2,
}

impl Status {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Playing => "playing",
            Self::Paused => "paused",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum ScalingMode {
    None = 0,
    #[default]
    FitToScreen = 1,
    FillToScreen = 2,
    StretchToFit = 3,
}

impl ScalingMode {
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::None => Self::FitToScreen,
            Self::FitToScreen => Self::FillToScreen,
            Self::FillToScreen => Self::StretchToFit,
            Self::StretchToFit => Self::None,
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

#[derive(Debug, Clone)]
pub enum Command {
    Stop,
    Pause,
    Resume,
    SetScalingMode(ScalingMode),
}

#[derive(Debug, Clone)]
pub struct VideoStateData {
    pub timestamp: i64,
    pub status: Status,
    pub filename: String,
    pub width: i32,
    pub height: i32,
    pub current_frame: i32,
    pub total_frames: i32,
    pub scaling_mode: ScalingMode,
    pub reason: String,
}

pub struct VideoPlayer<'a, R: Renderer> {
    renderer: &'a R,
    texture: Texture2D,
    width: i32,
    height: i32,
    status: Status,
    filename: String,
    target_fps: i32,
    paused: bool,
    state_generation: Arc<AtomicU32>,
    decoder: Option<VideoDecoder>,
    current_frame: i32,
    frame_time_accumulator: f64,
    scaling_mode: ScalingMode,
}

impl<'a, R: Renderer> VideoPlayer<'a, R> {
    #[must_use]
    pub fn new(renderer: &'a R) -> Self {
        Self {
            renderer,
            texture: Texture2D::default(),
            width: 0,
            height: 0,
            status: Status::Idle,
            filename: String::new(),
            target_fps: 30,
            paused: false,
            state_generation: Arc::new(AtomicU32::new(0)),
            decoder: None,
            current_frame: 0,
            frame_time_accumulator: 0.0,
            scaling_mode: ScalingMode::default(),
        }
    }

    /// # Errors
    /// Returns `VideoError::LoadFailed` if the file cannot be loaded.
    pub fn load(&mut self, filename: &str) -> Result<(), VideoError> {
        self.cleanup();
        self.filename = filename.to_string();

        if !Path::new(filename).exists() {
            return Err(VideoError::LoadFailed(format!("File not found: {filename}")));
        }

        let decoder = VideoDecoder::new(filename)
            .map_err(VideoError::LoadFailed)?;

        self.width = decoder.width();
        self.height = decoder.height();
        self.decoder = Some(decoder);
        self.status = Status::Paused;
        self.paused = true;
        self.current_frame = 0;
        self.frame_time_accumulator = 0.0;
        self.state_generation.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// # Errors
    /// Returns `VideoError::DecodeError` if frame decoding fails.
    pub fn tick(&mut self, delta_time: f64) -> Result<(), VideoError> {
        if self.status == Status::Idle || self.paused {
            return Ok(());
        }

        let Some(decoder) = &mut self.decoder else {
            return Ok(());
        };

        let frame_duration = 1.0 / decoder.frame_rate();
        self.frame_time_accumulator += delta_time;

        while self.frame_time_accumulator >= frame_duration {
            self.frame_time_accumulator -= frame_duration;

            match decoder.decode_frame() {
                Ok(true) => {
                    self.current_frame += 1;
                    decoder.set_current_frame(self.current_frame);

                    #[allow(clippy::as_ptr_cast_mut)]
                    let image = Image {
                        data: decoder.rgba_data().as_ptr() as *mut std::ffi::c_void,
                        width: self.width,
                        height: self.height,
                        mipmaps: 1,
                        format: PIXELFORMAT_UNCOMPRESSED_R8G8B8A8,
                    };

                    if self.texture.is_valid() {
                        self.renderer.update_texture(&self.texture, decoder.rgba_data());
                    } else {
                        self.texture = self.renderer.load_texture_from_image(image);
                    }
                    return Ok(());
                }
                Ok(false) => {
                    self.current_frame = 0;
                    decoder.seek(0).map_err(VideoError::DecodeError)?;
                }
                Err(e) => return Err(VideoError::DecodeError(e)),
            }
        }

        Ok(())
    }

    pub const fn set_target_fps(&mut self, fps: i32) {
        self.target_fps = fps;
    }

    #[must_use]
    pub const fn get_target_fps(&self) -> i32 {
        self.target_fps
    }

    #[must_use]
    pub const fn get_width(&self) -> i32 {
        self.width
    }

    #[must_use]
    pub const fn get_height(&self) -> i32 {
        self.height
    }

    #[must_use]
    pub const fn get_texture(&self) -> &Texture2D {
        &self.texture
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn push_command(&mut self, cmd: Command) {
        match cmd {
            Command::Stop => {
                self.status = Status::Idle;
                self.cleanup();
            },
            Command::Pause => {
                self.paused = true;
                self.status = Status::Paused;
            },
            Command::Resume => {
                self.paused = false;
                self.status = Status::Playing;
            },
            Command::SetScalingMode(mode) => {
                self.scaling_mode = mode;
            },
        }
        self.state_generation.fetch_add(1, Ordering::Relaxed);
    }

    #[must_use]
    pub const fn has_player(&self) -> bool {
        self.decoder.is_some()
    }

    #[must_use]
    pub const fn renderer(&self) -> &'a R {
        self.renderer
    }

    pub fn render(&self) {
        if !self.texture.is_valid() {
            return;
        }

        let screen_width = self.renderer.get_screen_width() as f32;
        let screen_height = self.renderer.get_screen_height() as f32;

        let (source, dest) = self.calculate_source_and_dest(screen_width, screen_height);

        self.renderer.draw_texture_pro(
            self.texture,
            source,
            dest,
            Vector2::ZERO,
            0.0,
            Color::WHITE,
        );
    }

    fn calculate_source_and_dest(&self, screen_width: f32, screen_height: f32) -> (Rectangle, Rectangle) {
        let video_width = self.width as f32;
        let video_height = self.height as f32;

        let scale_x = screen_width / video_width;
        let scale_y = screen_height / video_height;

        let dest_x;
        let dest_y;
        let dest_width;
        let dest_height;
        let source_x;
        let source_y;
        let source_width;
        let source_height;

        match self.scaling_mode {
            ScalingMode::None => {
                dest_width = video_width;
                dest_height = video_height;
                dest_x = (screen_width - dest_width) / 2.0;
                dest_y = (screen_height - dest_height) / 2.0;
                source_x = 0.0;
                source_y = 0.0;
                source_width = video_width;
                source_height = video_height;
            },
            ScalingMode::FitToScreen => {
                let scale = scale_x.min(scale_y);
                dest_width = video_width * scale;
                dest_height = video_height * scale;
                dest_x = (screen_width - dest_width) / 2.0;
                dest_y = (screen_height - dest_height) / 2.0;
                source_x = 0.0;
                source_y = 0.0;
                source_width = video_width;
                source_height = video_height;
            },
            ScalingMode::FillToScreen => {
                let scale = scale_x.max(scale_y);
                dest_width = screen_width;
                dest_height = screen_height;
                dest_x = 0.0;
                dest_y = 0.0;
                if scale_x >= scale_y {
                    source_width = screen_width / scale;
                    source_height = video_height;
                    source_x = (video_width - source_width) / 2.0;
                    source_y = 0.0;
                } else {
                    source_width = video_width;
                    source_height = screen_height / scale;
                    source_x = 0.0;
                    source_y = (video_height - source_height) / 2.0;
                }
            },
            ScalingMode::StretchToFit => {
                dest_width = screen_width;
                dest_height = screen_height;
                dest_x = 0.0;
                dest_y = 0.0;
                source_x = 0.0;
                source_y = 0.0;
                source_width = video_width;
                source_height = video_height;
            },
        }

        let source = Rectangle {
            x: source_x,
            y: source_y,
            width: source_width,
            height: source_height,
        };

        let dest = Rectangle {
            x: dest_x,
            y: dest_y,
            width: dest_width,
            height: dest_height,
        };

        (source, dest)
    }

    #[must_use]
    pub fn get_state_generation(&self) -> u32 {
        self.state_generation.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn get_state(&self) -> VideoStateData {
        VideoStateData {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or_else(|_| 0, |d| i64::try_from(d.as_millis()).unwrap_or(0)),
            status: self.status,
            filename: self.filename.clone(),
            width: self.width,
            height: self.height,
            current_frame: self.current_frame,
            total_frames: 0,
            scaling_mode: self.scaling_mode,
            reason: String::new(),
        }
    }

    fn cleanup(&mut self) {
        if self.texture.is_valid() {
            self.renderer.unload_texture(self.texture);
            self.texture = Texture2D::default();
        }
        self.width = 0;
        self.height = 0;
        self.filename.clear();
        self.decoder = None;
        self.current_frame = 0;
        self.frame_time_accumulator = 0.0;
        self.state_generation.fetch_add(1, Ordering::Relaxed);
    }
}