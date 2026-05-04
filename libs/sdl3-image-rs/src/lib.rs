pub use sdl3_image_sys::{IMG_Load, IMG_LoadAnimation, IMG_FreeAnimation, IMG_LoadTexture};
pub use sdl3_image_sys::{SDL_Surface as ImgSurface, SDL_PixelFormat, SDL_PixelFormat_SDL_PIXELFORMAT_ABGR8888, SDL_GetError, SDL_DestroySurface, SDL_ConvertSurface};

use libc::{c_int, c_void};
use std::ffi::{CStr, CString};
use std::ptr;
use thiserror::Error;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Image {
    pub data: *mut c_void,
    pub width: c_int,
    pub height: c_int,
    pub mipmaps: c_int,
    pub format: c_int,
}

impl Default for Image {
    fn default() -> Self {
        Self {
            data: ptr::null_mut(),
            width: 0,
            height: 0,
            mipmaps: 0,
            format: 0,
        }
    }
}

impl Image {
    #[must_use]
    pub const fn is_null(&self) -> bool {
        self.data.is_null()
    }
}

#[derive(Error, Debug)]
pub enum ImageError {
    #[error("Failed to load image: {0}")]
    LoadFailed(String),
    #[error("Failed to load animation: {0}")]
    AnimationLoadFailed(String),
    #[error("Failed to load texture: {0}")]
    TextureLoadFailed(String),
    #[error("Null pointer error")]
    NullPointer,
    #[error("Invalid path")]
    InvalidPath,
}

pub struct ImageLoader;

impl ImageLoader {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// # Errors
    /// Returns `ImageError` if the image cannot be loaded or converted.
    pub fn load(&self, path: &str) -> Result<Image, ImageError> {
        let c_path = CString::new(path).map_err(|_| ImageError::InvalidPath)?;

        unsafe {
            let surface = IMG_Load(c_path.as_ptr());
            if surface.is_null() {
                let err = SDL_GetError();
                let err_msg = if err.is_null() {
                    String::from("Unknown error")
                } else {
                    CStr::from_ptr(err)
                        .to_string_lossy()
                        .into_owned()
                };
                return Err(ImageError::LoadFailed(err_msg));
            }

            let converted = SDL_ConvertSurface(
                surface,
                SDL_PixelFormat_SDL_PIXELFORMAT_ABGR8888,
            );
            SDL_DestroySurface(surface);

            if converted.is_null() {
                return Err(ImageError::LoadFailed("Surface conversion failed".to_string()));
            }

            let width = (*converted).w;
            let height = (*converted).h;
            let pitch = (*converted).pitch;
            let data_size = usize::try_from(height).unwrap_or(0) * usize::try_from(pitch).unwrap_or(0);

            let mut data_copy = vec![0u8; data_size];
            std::ptr::copy_nonoverlapping(
                (*converted).pixels.cast::<u8>(),
                data_copy.as_mut_ptr(),
                data_size,
            );
            SDL_DestroySurface(converted);

            let data_ptr = data_copy.as_mut_ptr().cast::<c_void>();
            std::mem::forget(data_copy);

            Ok(Image {
                data: data_ptr,
                width,
                height,
                mipmaps: 1,
                format: 0,
            })
        }
    }

    /// # Errors
    /// Returns `ImageError` if the animation cannot be loaded or converted.
    pub fn load_animation(&self, path: &str) -> Result<(Image, c_int, Vec<u32>), ImageError> {
        let c_path = CString::new(path).map_err(|_| ImageError::InvalidPath)?;

        unsafe {
            let anim = IMG_LoadAnimation(c_path.as_ptr());
            if anim.is_null() {
                let err = SDL_GetError();
                let err_msg = if err.is_null() {
                    String::from("Unknown error")
                } else {
                    CStr::from_ptr(err)
                        .to_string_lossy()
                        .into_owned()
                };
                return Err(ImageError::AnimationLoadFailed(err_msg));
            }

            let frame_count = (*anim).count;
            let width = (*anim).w;
            let height = (*anim).h;

            if frame_count <= 0 {
                IMG_FreeAnimation(anim);
                return Err(ImageError::AnimationLoadFailed("No frames".to_string()));
            }

            let frame_count_usize = usize::try_from(frame_count).unwrap_or(0);
            let width_usize = usize::try_from(width).unwrap_or(0);
            let height_usize = usize::try_from(height).unwrap_or(0);

            let mut frame_delays = Vec::with_capacity(frame_count_usize);
            for i in 0..frame_count_usize {
                let delay = *(*anim).delays.add(i);
                frame_delays.push(delay.cast_unsigned());
            }

            let rgba_frame_size = width_usize * height_usize * 4;
            let total_size = rgba_frame_size * frame_count_usize;
            let mut all_frames = Vec::with_capacity(total_size);

            for i in 0..frame_count_usize {
                let frame = *(*anim).frames.add(i);
                if frame.is_null() {
                    IMG_FreeAnimation(anim);
                    return Err(ImageError::AnimationLoadFailed("Null frame".to_string()));
                }
                let converted = SDL_ConvertSurface(
                    frame,
                    SDL_PixelFormat_SDL_PIXELFORMAT_ABGR8888,
                );
                if converted.is_null() {
                    IMG_FreeAnimation(anim);
                    return Err(ImageError::AnimationLoadFailed("Conversion failed".to_string()));
                }
                let pitch = usize::try_from((*converted).pitch).unwrap_or(0);
                let row_bytes = width_usize * 4;
                let pixels = (*converted).pixels.cast::<u8>();
                for row in 0..height_usize {
                    all_frames.extend_from_slice(
                        std::slice::from_raw_parts(pixels.add(row * pitch), row_bytes)
                    );
                }
                SDL_DestroySurface(converted);
            }

            IMG_FreeAnimation(anim);

            let data_ptr = all_frames.as_mut_ptr().cast::<c_void>();
            std::mem::forget(all_frames);

            let image = Image {
                data: data_ptr,
                width,
                height,
                mipmaps: 1,
                format: 0,
            };

            Ok((image, frame_count, frame_delays))
        }
    }
}

impl Default for ImageLoader {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for ImageLoader {}
unsafe impl Sync for ImageLoader {}