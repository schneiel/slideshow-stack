use sdl3_rs::{Color, Rectangle, Renderer, Texture2D, Vector2};
use crate::command::CommandQueue;
use crate::image_loader::{get_current_frame, ImageLoader};
use crate::types::{ScalingMode, SlideshowConfig, SlideshowStateData, Status};
use parking_lot::Mutex;
use rand::Rng;
use rand::thread_rng;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SlideshowError {
    #[error("Queue push failed")]
    QueuePushFailed,
    #[error("Image load failed: {0}")]
    ImageLoadFailed(String),
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

pub struct Slideshow<'a, B: Renderer> {
    backend: &'a B,
    name: String,
    images: Vec<String>,
    current_index: usize,
    is_running: bool,
    is_paused: bool,
    shuffle: bool,
    loop_enabled: bool,
    interval_seconds: f64,
    elapsed_time: f64,
    scaling_mode: ScalingMode,
    #[allow(dead_code)]
    config: SlideshowConfig,
    image_loader: ImageLoader<'a, B>,
    command_queue: CommandQueue,
    state_generation: Mutex<u32>,
    window_width: i32,
    window_height: i32,
}

impl<'a, B: Renderer> Slideshow<'a, B> {
    #[must_use]
    pub fn new(backend: &'a B, config: SlideshowConfig) -> Self {
        Self {
            backend,
            name: String::new(),
            images: Vec::new(),
            current_index: 0,
            is_running: false,
            is_paused: false,
            shuffle: false,
            loop_enabled: true,
            interval_seconds: config.default_interval_seconds,
            elapsed_time: 0.0,
            scaling_mode: ScalingMode::default(),
            config,
            image_loader: ImageLoader::new(backend, config.max_cached_images),
            command_queue: CommandQueue::new(config.queue_capacity),
            state_generation: Mutex::new(0),
            window_width: 0,
            window_height: 0,
        }
    }

    /// # Errors
    /// Returns `SlideshowError::QueuePushFailed` if the command queue is full.
    pub fn push_command(&self, cmd: crate::command::Command) -> Result<(), SlideshowError> {
        self.command_queue.push(cmd).ok_or(SlideshowError::QueuePushFailed)
    }

    /// # Errors
    /// Returns `SlideshowError::QueuePushFailed` if the command queue is full.
    pub fn push_start_command(
        &self,
        name: String,
        interval_seconds: f64,
        shuffle: bool,
        loop_enabled: bool,
        image_paths: Vec<String>,
    ) -> Result<(), SlideshowError> {
        self.command_queue.push(crate::command::Command::Start {
            name,
            interval_seconds,
            shuffle,
            loop_enabled,
            image_paths,
        }).ok_or(SlideshowError::QueuePushFailed)
    }

    /// # Errors
    /// Returns `SlideshowError::QueuePushFailed` if the command queue is full.
    pub fn push_scaling_mode_command(&self, mode: ScalingMode) -> Result<(), SlideshowError> {
        self.command_queue.push(crate::command::Command::SetScalingMode(mode))
            .ok_or(SlideshowError::QueuePushFailed)
    }

    /// # Errors
    /// Returns `SlideshowError::QueuePushFailed` if the command queue is full.
    pub fn push_pause_command(&self, paused: bool) -> Result<(), SlideshowError> {
        let cmd = if paused {
            crate::command::Command::Pause
        } else {
            crate::command::Command::Resume
        };
        self.command_queue.push(cmd).ok_or(SlideshowError::QueuePushFailed)
    }

    pub fn tick(&mut self, delta_time: f64) {
        self.update_window_size();
        self.process_commands();

        if !self.is_running || self.is_paused {
            return;
        }

        self.elapsed_time += delta_time;
        if self.elapsed_time >= self.interval_seconds {
            self.elapsed_time = 0.0;
            self.next_image();
        }
    }

    fn process_commands(&mut self) {
        while let Some(cmd) = self.command_queue.pop() {
            match cmd {
                crate::command::Command::Start {
                    name,
                    interval_seconds,
                    shuffle,
                    loop_enabled,
                    image_paths,
                } => {
                    self.start(name, interval_seconds, shuffle, loop_enabled, image_paths);
                },
                crate::command::Command::Stop => self.stop(),
                crate::command::Command::Pause => {
                    self.is_paused = true;
                    *self.state_generation.lock() += 1;
                },
                crate::command::Command::Resume => {
                    self.is_paused = false;
                    *self.state_generation.lock() += 1;
                },
                crate::command::Command::Next => { let _ = self.next_image(); },
                crate::command::Command::Previous => { let _ = self.previous_image(); },
                crate::command::Command::SetScalingMode(mode) => {
                    self.scaling_mode = mode;
                    *self.state_generation.lock() += 1;
                },
                crate::command::Command::Shutdown => {
                    self.stop();
                },
            }
        }
    }

    fn start(
        &mut self,
        name: String,
        interval_seconds: f64,
        shuffle: bool,
        loop_enabled: bool,
        image_paths: Vec<String>,
    ) {
        self.name = name;
        self.interval_seconds = interval_seconds;
        self.shuffle = shuffle;
        self.loop_enabled = loop_enabled;
        self.images = image_paths;
        self.current_index = 0;
        self.elapsed_time = 0.0;
        self.is_running = true;
        self.is_paused = false;

        if self.shuffle && !self.images.is_empty() {
            let mut rng = thread_rng();
            self.current_index = rng.gen_range(0..self.images.len());
        }

        *self.state_generation.lock() += 1;
    }

    fn stop(&mut self) {
        self.is_running = false;
        self.is_paused = false;
        self.images.clear();
        self.current_index = 0;
        self.elapsed_time = 0.0;
        self.image_loader.clear_cache();
        *self.state_generation.lock() += 1;
    }

    fn next_image(&mut self) -> Option<()> {
        if self.images.is_empty() {
            return None;
        }
        self.current_index = if self.loop_enabled {
            (self.current_index + 1) % self.images.len()
        } else {
            self.current_index.saturating_add(1).min(self.images.len() - 1)
        };
        *self.state_generation.lock() += 1;
        Some(())
    }

    fn previous_image(&mut self) -> Option<()> {
        if self.images.is_empty() {
            return None;
        }
        self.current_index = if self.loop_enabled {
            if self.current_index == 0 {
                self.images.len() - 1
            } else {
                self.current_index - 1
            }
        } else {
            self.current_index.saturating_sub(1)
        };
        *self.state_generation.lock() += 1;
        Some(())
    }

    fn update_window_size(&mut self) {
        self.window_width = self.backend.get_screen_width();
        self.window_height = self.backend.get_screen_height();
    }

    pub fn render(&mut self) {
        if !self.is_running || self.images.is_empty() {
            return;
        }

        let current_time = self.backend.get_time();
        let path = &self.images[self.current_index];

        let texture_opt;
        let img_dim_opt;
        {
            if let Some(entry) = self.image_loader.load(path) {
                texture_opt = Some(get_current_frame(self.backend, entry, current_time));
                img_dim_opt = Some((entry.width, entry.height));
            } else {
                texture_opt = None;
                img_dim_opt = None;
            }
        }
        if let (Some(texture), Some((width, height))) = (texture_opt, img_dim_opt) {
            self.render_texture(texture, width, height);
        }
    }

    fn render_texture(&self, texture: Texture2D, img_width: i32, img_height: i32) {
        let screen_width = self.window_width as f32;
        let screen_height = self.window_height as f32;
        let video_width = img_width as f32;
        let video_height = img_height as f32;

        let scale_x = screen_width / video_width;
        let scale_y = screen_height / video_height;

        let (source, dest) = match self.scaling_mode {
            ScalingMode::None => {
                let source = Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: video_width,
                    height: video_height,
                };
                let dest = Rectangle {
                    x: (screen_width - video_width) / 2.0,
                    y: (screen_height - video_height) / 2.0,
                    width: video_width,
                    height: video_height,
                };
                (source, dest)
            },
            ScalingMode::FitToScreen => {
                let scale = scale_x.min(scale_y);
                let dest_width = video_width * scale;
                let dest_height = video_height * scale;
                let source = Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: video_width,
                    height: video_height,
                };
                let dest = Rectangle {
                    x: (screen_width - dest_width) / 2.0,
                    y: (screen_height - dest_height) / 2.0,
                    width: dest_width,
                    height: dest_height,
                };
                (source, dest)
            },
            ScalingMode::FillToScreen => {
                let scale = scale_x.max(scale_y);
                let source = if scale_x >= scale_y {
                    let source_width = screen_width / scale;
                    Rectangle { x: (video_width - source_width) / 2.0, y: 0.0, width: source_width, height: video_height }
                } else {
                    let source_height = screen_height / scale;
                    Rectangle { x: 0.0, y: (video_height - source_height) / 2.0, width: video_width, height: source_height }
                };
                let dest = Rectangle { x: 0.0, y: 0.0, width: screen_width, height: screen_height };
                (source, dest)
            },
            ScalingMode::StretchToFit => {
                let source = Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: video_width,
                    height: video_height,
                };
                let dest = Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: screen_width,
                    height: screen_height,
                };
                (source, dest)
            },
        };

        self.backend.draw_texture_pro(
            texture,
            source,
            dest,
            Vector2::ZERO,
            0.0,
            Color::WHITE,
        );
    }

    #[must_use]
    pub const fn get_window_size(&self) -> (i32, i32) {
        (self.window_width, self.window_height)
    }

    #[must_use]
    pub const fn is_running(&self) -> bool {
        self.is_running
    }

    #[must_use]
    pub const fn is_paused(&self) -> bool {
        self.is_paused
    }

    #[must_use]
    pub const fn get_current_index(&self) -> usize {
        self.current_index
    }

    #[must_use]
    pub const fn get_total_images(&self) -> usize {
        self.images.len()
    }

    #[must_use]
    pub const fn get_scaling_mode(&self) -> ScalingMode {
        self.scaling_mode
    }

    #[must_use]
    pub fn get_state_generation(&self) -> u32 {
        *self.state_generation.lock()
    }

    #[must_use]
    pub fn get_state(&self) -> SlideshowStateData {
        SlideshowStateData {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or_else(|_| 0, |d| i64::try_from(d.as_millis()).unwrap_or(0)),
            status: if self.is_paused {
                Status::Paused
            } else if self.is_running {
                Status::Playing
            } else {
                Status::Stopped
            },
            name: self.name.clone(),
            current_image: if self.is_running && !self.images.is_empty() {
                self.images[self.current_index].clone()
            } else {
                String::new()
            },
            current_index: self.current_index,
            total_images: self.images.len(),
            interval_seconds: self.interval_seconds,
            scaling_mode: self.scaling_mode,
            reason: String::new(),
        }
    }
}
