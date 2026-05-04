use sdl3_rs::Renderer;
use sdl3_image_rs::ImageLoader as SdlImageLoader;
use crate::types::CacheEntry;
use sdl3_rs::Texture2D;
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct ImageLoader<'a, B: Renderer> {
    backend: &'a B,
    loader: SdlImageLoader,
    cache: HashMap<String, CacheEntry>,
    access_order: Mutex<VecDeque<String>>,
    capacity: usize,
    failed_paths: Mutex<HashSet<String>>,
}

impl<'a, B: Renderer> ImageLoader<'a, B> {
    #[must_use]
    pub fn new(backend: &'a B, capacity: usize) -> Self {
        Self {
            backend,
            loader: SdlImageLoader::new(),
            cache: HashMap::new(),
            access_order: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
            failed_paths: Mutex::new(HashSet::new()),
        }
    }

    pub fn load(&mut self, path: &str) -> Option<&mut CacheEntry> {
        if self.cache.contains_key(path) {
            let mut order = self.access_order.lock();
            order.retain(|p| p != path);
            order.push_back(path.to_string());
            drop(order);
            return self.cache.get_mut(path);
        }

        if self.failed_paths.lock().contains(path) {
            return None;
        }

        let is_gif_file = path.to_lowercase().ends_with(".gif");

        let (texture, width, height, frame_count, frame_delays, frame_data) = if is_gif_file {
            let result = self.loader.load_animation(path);
            let Ok((image, frame_count, frame_delays)) = result else {
                self.failed_paths.lock().insert(path.to_string());
                return None;
            };

            if image.is_null() {
                self.failed_paths.lock().insert(path.to_string());
                return None;
            }

            let width = image.width;
            let height = image.height;
            let is_gif = is_gif_file && frame_count > 1;

            let frame_data = if is_gif {
                let bytes_per_pixel = 4;
                let frame_size = usize::try_from(width * height * bytes_per_pixel).unwrap_or(0);
                let total_frames = usize::try_from(frame_count).unwrap_or(1);
                let total_size = frame_size
                    .checked_mul(total_frames)
                    .unwrap_or(0);
                let data_ptr = image.data;
                let slice = unsafe { std::slice::from_raw_parts(data_ptr as *const u8, total_size) };
                Some(slice.to_vec())
            } else {
                None
            };

            let sdl_image = sdl3_rs::Image {
                data: image.data,
                width: image.width,
                height: image.height,
                mipmaps: image.mipmaps,
                format: image.format,
            };
            let texture = self.backend.load_texture_from_image(sdl_image);

            (texture, width, height, frame_count as usize, frame_delays, frame_data)
        } else {
            let result = self.loader.load(path);
            let Ok(image) = result else {
                self.failed_paths.lock().insert(path.to_string());
                return None;
            };

            if image.is_null() {
                self.failed_paths.lock().insert(path.to_string());
                return None;
            }

            let width = image.width;
            let height = image.height;

            let sdl_image = sdl3_rs::Image {
                data: image.data,
                width: image.width,
                height: image.height,
                mipmaps: image.mipmaps,
                format: image.format,
            };
            let texture = self.backend.load_texture_from_image(sdl_image);

            (texture, width, height, 1usize, Vec::new(), None)
        };

        if self.cache.len() >= self.capacity {
            let lru_path = self.access_order.lock().pop_front();
            if let Some(lru_path) = lru_path
                && let Some(evicted) = self.cache.remove(&lru_path) {
                self.backend.unload_texture(evicted.texture);
            }
        }

        let actual_is_gif = is_gif_file && frame_count > 1;
        let actual_frame_count = if actual_is_gif { frame_count } else { 1 };
        let frame_delays = if actual_is_gif && !frame_delays.is_empty() {
            frame_delays
        } else if actual_is_gif {
            vec![100; actual_frame_count]
        } else {
            Vec::new()
        };
        let entry = CacheEntry {
            path: path.to_string(),
            texture,
            width,
            height,
            is_gif: actual_is_gif,
            current_frame: 0,
            frame_count: actual_frame_count,
            frame_delays_ms: frame_delays,
            last_frame_time: 0.0,
            frame_data,
        };

        self.cache.insert(path.to_string(), entry);
        self.access_order.lock().push_back(path.to_string());
        self.cache.get_mut(path)
    }

    pub fn is_loaded(&self, path: &str) -> bool {
        self.cache.contains_key(path)
    }

    pub fn is_failed(&self, path: &str) -> bool {
        self.failed_paths.lock().contains(path)
    }

    pub fn clear_cache(&mut self) {
        for entry in self.cache.values() {
            self.backend.unload_texture(entry.texture);
        }
        self.cache.clear();
        self.access_order.lock().clear();
    }

    pub fn clear_failed(&mut self) {
        self.failed_paths.lock().clear();
    }

    pub fn unload(&mut self, path: &str) {
        if let Some(entry) = self.cache.remove(path) {
            self.backend.unload_texture(entry.texture);
            self.access_order.lock().retain(|p| p != path);
        }
    }
}

pub fn get_current_frame<B: Renderer>(
    backend: &B,
    entry: &mut CacheEntry,
    current_time: f64,
) -> Texture2D {
    if !entry.is_gif || entry.frame_count <= 1 {
        return entry.texture;
    }

    let elapsed = current_time - entry.last_frame_time;
    let delay_sec = f64::from(entry.frame_delays_ms[entry.current_frame]) / 1000.0;

    let prev_frame = entry.current_frame;

    if elapsed >= delay_sec {
        entry.current_frame = (entry.current_frame + 1) % entry.frame_count;
        entry.last_frame_time = current_time;
    }

    if entry.current_frame != prev_frame
        && let Some(ref frame_data) = entry.frame_data
    {
        let bytes_per_pixel = 4;
        let frame_size = usize::try_from(entry.width * entry.height * bytes_per_pixel).unwrap_or(0);
        let offset = frame_size * entry.current_frame;
        if offset + frame_size <= frame_data.len() {
            backend.update_texture(&entry.texture, &frame_data[offset..offset + frame_size]);
        }
    }

    entry.texture
}

impl<B: Renderer> Drop for ImageLoader<'_, B> {
    fn drop(&mut self) {
        for entry in self.cache.values() {
            self.backend.unload_texture(entry.texture);
        }
    }
}
