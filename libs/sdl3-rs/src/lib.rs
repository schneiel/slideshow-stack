#![allow(
    clippy::must_use_candidate,
    clippy::borrow_as_ptr,
)]

pub use sdl3_sys::*;

use libc::{c_double, c_float, c_int};
pub use sdl3_image_rs::Image;
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};
use thiserror::Error;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vector2 {
    pub x: c_float,
    pub y: c_float,
}

impl Vector2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub x: c_float,
    pub y: c_float,
    pub width: c_float,
    pub height: c_float,
}

impl Rectangle {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Texture2D {
    raw: *mut SDL_Texture,
    width: c_int,
    height: c_int,
    format: c_int,
}

impl Default for Texture2D {
    fn default() -> Self {
        Self {
            raw: ptr::null_mut(),
            width: 0,
            height: 0,
            format: 0,
        }
    }
}

impl Texture2D {
    pub const fn is_valid(&self) -> bool {
        !self.raw.is_null()
    }

    pub const fn width(&self) -> c_int {
        self.width
    }

    pub const fn height(&self) -> c_int {
        self.height
    }
}

pub const FLAG_WINDOW_RESIZABLE: u64 = 0x0000_0020;
pub const FLAG_FULLSCREEN: u64 = 0x0000_0001;
pub const FLAG_VSYNC: u64 = 0x0000_0040;
pub const FLAG_WINDOW_TRANSPARENT: u64 = 0x0004_0000_0000;
pub const FLAG_WINDOW_BORDERLESS: u64 = 0x0000_0010;
pub const FLAG_WINDOW_MINIMIZED: u64 = 0x0000_0004;
pub const FLAG_WINDOW_MAXIMIZED: u64 = 0x0000_0008;
pub const FLAG_WINDOW_HIDDEN: u64 = 0x0000_0002;
pub const FLAG_WINDOW_HIGH_PIXEL_DENSITY: u64 = 0x0000_0200;
pub const FLAG_WINDOW_MOUSE_GRABBED: u64 = 0x0000_0100;
pub const FLAG_WINDOW_ALWAYS_ON_TOP: u64 = 0x0000_8000;
pub const FLAG_WINDOW_NOT_FOCUSABLE: u64 = 0x0002_0000_0000;
pub const FLAG_WINDOW_METAL: u64 = 0x0001_0000_0000;
pub const FLAG_WINDOW_VULKAN: u64 = 0x0040_0000_0000;
pub const FLAG_WINDOW_OPENGL: u64 = 0x0000_0002;
pub const PIXELFORMAT_UNCOMPRESSED_R8G8B8A8: c_int = 0x0157;
pub const WINDOWPOS_CENTERED: c_int = 0x2FFF_0000;

#[derive(Error, Debug)]
pub enum SdlError {
    #[error("SDL initialization failed: {0}")]
    InitFailed(String),
    #[error("Window creation failed: {0}")]
    WindowCreationFailed(String),
    #[error("Renderer creation failed: {0}")]
    RendererCreationFailed(String),
    #[error("Image load failed: {0}")]
    ImageLoadFailed(String),
    #[error("Texture creation failed: {0}")]
    TextureCreationFailed(String),
    #[error("Null pointer error")]
    NullPointer,
}

pub trait Renderer {
    fn set_log_output_function(&mut self, callback: SDL_LogOutputFunction, userdata: *mut libc::c_void);
    fn set_trace_log_level(&self, _log_level: c_int);
    /// # Errors
    /// Returns `SdlError` if SDL initialization or window/renderer creation fails.
    fn init_window(&mut self, width: c_int, height: c_int, title: &str) -> Result<(), SdlError>;
    fn close_window(&mut self);
    fn window_should_close(&self) -> bool;
    fn is_window_ready(&self) -> bool;
    fn set_target_fps(&mut self, fps: c_int);
    fn get_fps(&self) -> c_int;
    fn get_frame_time(&self) -> c_float;
    fn get_time(&self) -> c_double;
    fn set_config_flags(&mut self, flags: u64);
    fn get_current_monitor(&self) -> c_int;
    fn get_monitor_width(&self, _monitor: c_int) -> c_int;
    fn get_monitor_height(&self, _monitor: c_int) -> c_int;
    fn get_screen_width(&self) -> c_int;
    fn get_screen_height(&self) -> c_int;
    fn get_renderer_output_size(&self) -> (c_int, c_int);
    fn get_window_flags(&self) -> u64;
    fn set_window_size(&mut self, width: c_int, height: c_int);
    fn set_window_position(&mut self, x: c_int, y: c_int);
    fn maximize_window(&mut self);
    fn sync_window(&mut self);
    fn show_window(&mut self);
    fn raise_window(&mut self);
    fn minimize_window(&mut self);
    fn restore_window(&mut self);
    fn toggle_fullscreen(&mut self);
    fn hide_cursor(&mut self);
    fn show_cursor(&mut self);
    fn begin_drawing(&self);
    fn end_drawing(&self);
    fn clear_background(&self, color: Color);
    fn draw_text(&self, text: &str, x: c_int, y: c_int, font_size: c_int, color: Color);
    fn measure_text(&self, text: &str, font_size: c_int) -> c_int;
    fn draw_rectangle(&self, x: c_int, y: c_int, width: c_int, height: c_int, color: Color);
    fn draw_rectangle_lines(&self, x: c_int, y: c_int, width: c_int, height: c_int, color: Color);
    fn is_key_pressed(&self, _key: c_int) -> bool;
    fn wait_time(&self, seconds: c_double);
    fn load_texture_from_image(&self, image: Image) -> Texture2D;
    fn unload_texture(&self, texture: Texture2D);
    fn draw_texture_pro(
        &self,
        texture: Texture2D,
        source: Rectangle,
        dest: Rectangle,
        origin: Vector2,
        rotation: c_float,
        tint: Color,
    );
    fn update_texture(&self, texture: &Texture2D, data: &[u8]);
    fn file_exists(&self, path: &str) -> bool;
    fn directory_exists(&self, path: &str) -> bool;
}

pub struct Sdl3Renderer {
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
    renderer_name: Option<String>,
    target_fps: c_int,
    last_frame_time: u64,
    start_time: u64,
    ref_count: std::sync::atomic::AtomicUsize,
    owned: bool,
    window_flags: u64,
    renderer_flags: u32,
}

impl Sdl3Renderer {
    pub const fn new() -> Self {
        Self {
            window: ptr::null_mut(),
            renderer: ptr::null_mut(),
            renderer_name: None,
            target_fps: 60,
            last_frame_time: 0,
            start_time: 0,
            ref_count: std::sync::atomic::AtomicUsize::new(1),
            owned: true,
            window_flags: FLAG_WINDOW_RESIZABLE,
            renderer_flags: 0,
        }
    }

    pub const fn with_fps(fps: c_int) -> Self {
        let mut r = Self::new();
        r.target_fps = fps;
        r
    }

    unsafe fn get_last_error() -> String {
        let err = SDL_GetError();
        if err.is_null() {
            String::new()
        } else {
            CStr::from_ptr(err)
                .to_string_lossy()
                .into_owned()
        }
    }

    pub fn get_renderer_name(&self) -> Option<&str> {
        self.renderer_name.as_deref()
    }
}

impl Default for Sdl3Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Sdl3Renderer {
    fn clone(&self) -> Self {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
        Self {
            window: self.window,
            renderer: self.renderer,
            renderer_name: self.renderer_name.clone(),
            target_fps: self.target_fps,
            last_frame_time: self.last_frame_time,
            start_time: self.start_time,
            ref_count: AtomicUsize::new(self.ref_count.load(Ordering::SeqCst)),
            owned: false,
            window_flags: self.window_flags,
            renderer_flags: self.renderer_flags,
        }
    }
}

impl Drop for Sdl3Renderer {
    fn drop(&mut self) {
        let count = self.ref_count.fetch_sub(1, Ordering::SeqCst);
        if count == 1 && self.owned {
            if !self.renderer.is_null() {
                unsafe { SDL_DestroyRenderer(self.renderer) };
            }
            if !self.window.is_null() {
                unsafe { SDL_DestroyWindow(self.window) };
            }
        }
    }
}

impl Renderer for Sdl3Renderer {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn set_log_output_function(&mut self, callback: SDL_LogOutputFunction, userdata: *mut libc::c_void) {
        unsafe {
            SDL_SetLogOutputFunction(callback, userdata);
        }
    }

    fn set_trace_log_level(&self, log_level: c_int) {
        let priority = match log_level {
            0 => SDL_LogPriority_SDL_LOG_PRIORITY_TRACE,
            1 => SDL_LogPriority_SDL_LOG_PRIORITY_DEBUG,
            2 => SDL_LogPriority_SDL_LOG_PRIORITY_VERBOSE,
            4 => SDL_LogPriority_SDL_LOG_PRIORITY_WARN,
            5 => SDL_LogPriority_SDL_LOG_PRIORITY_ERROR,
            _ => SDL_LogPriority_SDL_LOG_PRIORITY_INFO,
        };
        unsafe {
            SDL_SetLogPriorities(priority);
        }
    }

    fn init_window(&mut self, width: c_int, height: c_int, title: &str) -> Result<(), SdlError> {
        unsafe {
            if SDL_WasInit(0) == 0 && !SDL_Init(SDL_INIT_VIDEO) {
                return Err(SdlError::InitFailed(Self::get_last_error()));
            }

            let c_title = CString::new(title)
                .map_err(|_| SdlError::InitFailed("NulError".to_string()))?;

            let props = SDL_CreateProperties();

            let prop_title = CString::new("SDL.window.create.title")
                .map_err(|_| SdlError::InitFailed("NulError".to_string()))?;
            SDL_SetStringProperty(props, prop_title.as_ptr(), c_title.as_ptr());

            let prop_width = CString::new("SDL.window.create.width")
                .map_err(|_| SdlError::InitFailed("NulError".to_string()))?;
            SDL_SetNumberProperty(props, prop_width.as_ptr(), i64::from(width));

            let prop_height = CString::new("SDL.window.create.height")
                .map_err(|_| SdlError::InitFailed("NulError".to_string()))?;
            SDL_SetNumberProperty(props, prop_height.as_ptr(), i64::from(height));

            let prop_flags = CString::new("SDL.window.create.flags")
                .map_err(|_| SdlError::InitFailed("NulError".to_string()))?;
            SDL_SetNumberProperty(props, prop_flags.as_ptr(), self.window_flags.cast_signed());

            let prop_transparent = CString::new("SDL.window.create.transparent")
                .map_err(|_| SdlError::InitFailed("NulError".to_string()))?;
            SDL_SetBooleanProperty(props, prop_transparent.as_ptr(), false);

            let window = SDL_CreateWindowWithProperties(props);
            SDL_DestroyProperties(props);

            if window.is_null() {
                return Err(SdlError::WindowCreationFailed(Self::get_last_error()));
            }

            let renderer = SDL_CreateRenderer(window, ptr::null_mut());

            if renderer.is_null() {
                SDL_DestroyWindow(window);
                return Err(SdlError::RendererCreationFailed(Self::get_last_error()));
            }

            SDL_SetWindowOpacity(window, 1.0_f32);

            self.window = window;
            self.renderer = renderer;
            self.renderer_name = None;
            self.start_time = SDL_GetTicks();
        }
        Ok(())
    }

    fn close_window(&mut self) {
        if !self.renderer.is_null() {
            unsafe { SDL_DestroyRenderer(self.renderer) };
            self.renderer = ptr::null_mut();
        }
        if !self.window.is_null() {
            unsafe { SDL_DestroyWindow(self.window) };
            self.window = ptr::null_mut();
        }
    }

    fn window_should_close(&self) -> bool {
        unsafe {
            let mut event: SDL_Event = std::mem::zeroed();
            while SDL_PollEvent(&mut event) {
                if event.type_ == SDL_EventType_SDL_EVENT_QUIT {
                    return true;
                }
            }
        }
        false
    }

    fn is_window_ready(&self) -> bool {
        !self.window.is_null()
    }

    fn set_target_fps(&mut self, fps: c_int) {
        self.target_fps = fps;
    }

    fn get_fps(&self) -> c_int {
        if self.target_fps <= 0 {
            return 0;
        }
        self.target_fps
    }

    fn get_frame_time(&self) -> c_float {
        if self.target_fps == 0 {
            return 0.0;
        }
        #[allow(clippy::cast_possible_truncation)]
        { (1000.0 / f64::from(self.target_fps)) as c_float }
    }

    #[allow(clippy::cast_precision_loss)]
    fn get_time(&self) -> c_double {
        (unsafe { SDL_GetTicks() }) as c_double / 1000.0
    }

    fn set_config_flags(&mut self, flags: u64) {
        self.window_flags = flags;
    }

    fn get_current_monitor(&self) -> c_int {
        0
    }

    fn get_monitor_width(&self, _monitor: c_int) -> c_int {
        unsafe {
            let display_id = SDL_GetPrimaryDisplay();
            let mut rect: SDL_Rect = std::mem::zeroed();
            if SDL_GetDisplayBounds(display_id, &mut rect) && rect.w > 0 {
                rect.w
            } else {
                1280
            }
        }
    }

    fn get_monitor_height(&self, _monitor: c_int) -> c_int {
        unsafe {
            let display_id = SDL_GetPrimaryDisplay();
            let mut rect: SDL_Rect = std::mem::zeroed();
            if SDL_GetDisplayBounds(display_id, &mut rect) && rect.h > 0 {
                rect.h
            } else {
                720
            }
        }
    }

    fn get_screen_width(&self) -> c_int {
        if !self.window.is_null() {
            unsafe {
                let mut w: c_int = 0;
                let mut h: c_int = 0;
                if SDL_GetWindowSize(self.window, &mut w, &mut h) {
                    return w;
                }
            }
        }
        self.get_monitor_width(0)
    }

    fn get_screen_height(&self) -> c_int {
        if !self.window.is_null() {
            unsafe {
                let mut w: c_int = 0;
                let mut h: c_int = 0;
                if SDL_GetWindowSize(self.window, &mut w, &mut h) {
                    return h;
                }
            }
        }
        self.get_monitor_height(0)
    }

    fn get_renderer_output_size(&self) -> (c_int, c_int) {
        if !self.renderer.is_null() {
            unsafe {
                let mut w: c_int = 0;
                let mut h: c_int = 0;
                if SDL_GetCurrentRenderOutputSize(self.renderer, &mut w, &mut h) {
                    return (w, h);
                }
            }
        }
        (0, 0)
    }

    fn get_window_flags(&self) -> u64 {
        if self.window.is_null() {
            0
        } else {
            unsafe { SDL_GetWindowFlags(self.window) as u64 }
        }
    }

    fn set_window_size(&mut self, width: c_int, height: c_int) {
        if !self.window.is_null() {
            unsafe { SDL_SetWindowSize(self.window, width, height) };
        }
    }

    fn set_window_position(&mut self, x: c_int, y: c_int) {
        if !self.window.is_null() {
            unsafe { SDL_SetWindowPosition(self.window, x, y) };
        }
    }

    fn maximize_window(&mut self) {
        if !self.window.is_null() {
            unsafe { SDL_MaximizeWindow(self.window) };
        }
    }

    fn sync_window(&mut self) {
        if !self.window.is_null() {
            unsafe { SDL_SyncWindow(self.window) };
        }
    }

    fn show_window(&mut self) {
        if !self.window.is_null() {
            unsafe { SDL_ShowWindow(self.window) };
        }
    }

    fn raise_window(&mut self) {
        if !self.window.is_null() {
            unsafe { SDL_RaiseWindow(self.window) };
        }
    }

    fn minimize_window(&mut self) {
        if !self.window.is_null() {
            unsafe { SDL_MinimizeWindow(self.window) };
        }
    }

    fn restore_window(&mut self) {
        if !self.window.is_null() {
            unsafe { SDL_RestoreWindow(self.window) };
        }
    }

    fn toggle_fullscreen(&mut self) {
        if !self.window.is_null() {
            unsafe { SDL_SetWindowFullscreen(self.window, true) };
        }
    }

    fn hide_cursor(&mut self) {
        unsafe { SDL_HideCursor() };
    }

    fn show_cursor(&mut self) {
        unsafe { SDL_ShowCursor() };
    }

    fn begin_drawing(&self) {
        if !self.renderer.is_null() {
            unsafe {
                SDL_SetRenderTarget(self.renderer, ptr::null_mut());
            }
        }
    }

    fn end_drawing(&self) {
        if !self.renderer.is_null() {
            unsafe {
                SDL_RenderPresent(self.renderer);
                SDL_PumpEvents();
            }
        }
    }

    fn clear_background(&self, color: Color) {
        if !self.renderer.is_null() {
            unsafe {
                SDL_SetRenderDrawColor(self.renderer, color.r, color.g, color.b, color.a);
                SDL_RenderClear(self.renderer);
            }
        }
    }

    fn draw_text(&self, _text: &str, _x: c_int, _y: c_int, _font_size: c_int, _color: Color) {
    }

    fn measure_text(&self, text: &str, _font_size: c_int) -> c_int {
        i32::try_from(text.len()).unwrap_or(0) * 8
    }

    #[allow(clippy::cast_precision_loss)]
    fn draw_rectangle(&self, x: c_int, y: c_int, width: c_int, height: c_int, color: Color) {
        if !self.renderer.is_null() {
            unsafe {
                SDL_SetRenderDrawColor(self.renderer, color.r, color.g, color.b, color.a);
                let rect = SDL_FRect {
                    x: x as f32,
                    y: y as f32,
                    w: width as f32,
                    h: height as f32,
                };
                SDL_RenderFillRect(self.renderer, &rect);
            }
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn draw_rectangle_lines(&self, x: c_int, y: c_int, width: c_int, height: c_int, color: Color) {
        if !self.renderer.is_null() {
            unsafe {
                SDL_SetRenderDrawColor(self.renderer, color.r, color.g, color.b, color.a);
                let rect = SDL_FRect {
                    x: x as f32,
                    y: y as f32,
                    w: width as f32,
                    h: height as f32,
                };
                SDL_RenderRect(self.renderer, &rect);
            }
        }
    }

    fn is_key_pressed(&self, _key: c_int) -> bool {
        false
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn wait_time(&self, seconds: c_double) {
        unsafe { SDL_Delay((seconds * 1000.0) as u32) };
    }

    fn load_texture_from_image(&self, image: Image) -> Texture2D {
        if image.is_null() || self.renderer.is_null() {
            return Texture2D::default();
        }

        unsafe {
            let pitch = image.width * 4;
            // Use SDL_CreateTexture with STREAMING access and an explicit format so that
            // subsequent SDL_UpdateTexture calls in frame animation use the same format.
            // SDL_CreateTextureFromSurface may pick a different internal format, which
            // would cause SDL_UpdateTexture to misinterpret the channel order.
            let texture_ptr = SDL_CreateTexture(
                self.renderer,
                SDL_PixelFormat_SDL_PIXELFORMAT_ABGR8888,
                SDL_TextureAccess_SDL_TEXTUREACCESS_STREAMING,
                image.width,
                image.height,
            );

            if texture_ptr.is_null() {
                return Texture2D::default();
            }

            let rect = SDL_Rect {
                x: 0,
                y: 0,
                w: image.width,
                h: image.height,
            };
            SDL_UpdateTexture(
                texture_ptr,
                &rect,
                image.data,
                pitch,
            );

            Texture2D {
                raw: texture_ptr,
                width: image.width,
                height: image.height,
                format: SDL_PixelFormat_SDL_PIXELFORMAT_ABGR8888.cast_signed(),
            }
        }
    }

    fn unload_texture(&self, texture: Texture2D) {
        if !texture.raw.is_null() {
            unsafe { SDL_DestroyTexture(texture.raw) };
        }
    }

    fn draw_texture_pro(
        &self,
        texture: Texture2D,
        source: Rectangle,
        dest: Rectangle,
        _origin: Vector2,
        _rotation: c_float,
        tint: Color,
    ) {
        if !self.renderer.is_null() && !texture.raw.is_null() {
            unsafe {
                let src = SDL_FRect {
                    x: source.x,
                    y: source.y,
                    w: source.width,
                    h: source.height,
                };
                let dst_rect = SDL_FRect {
                    x: dest.x,
                    y: dest.y,
                    w: dest.width,
                    h: dest.height,
                };
                SDL_SetRenderDrawColor(self.renderer, tint.r, tint.g, tint.b, tint.a);
                SDL_RenderTexture(self.renderer, texture.raw, &src, &dst_rect);
            }
        }
    }

    fn update_texture(&self, texture: &Texture2D, data: &[u8]) {
        if texture.raw.is_null() || data.is_empty() || self.renderer.is_null() {
            return;
        }

        unsafe {
            let pitch = texture.width * 4;
            let rect = SDL_Rect {
                x: 0,
                y: 0,
                w: texture.width,
                h: texture.height,
            };
            SDL_UpdateTexture(
                texture.raw,
                &rect,
                data.as_ptr().cast::<libc::c_void>(),
                pitch,
            );
        }
    }

    fn file_exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }

    fn directory_exists(&self, path: &str) -> bool {
        std::path::Path::new(path).is_dir()
    }
}
