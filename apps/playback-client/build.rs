use std::path::{Path, PathBuf};
use std::fs;

fn install_dir(include_var: &str) -> PathBuf {
    PathBuf::from(std::env::var(include_var)
        .unwrap_or_else(|_| panic!("{include_var} not set")))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let target_bin_dir = Path::new(&out_dir)
        .ancestors()
        .nth(3)
        .unwrap()
        .to_path_buf();

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let is_macos = target_os == "macos";

    let sdl3_install = install_dir("DEP_SDL3_INCLUDE");
    let sdl3_image_install = install_dir("DEP_SDL3_IMAGE_INCLUDE");
    let ffmpeg_install = install_dir("DEP_FFMPEG_INCLUDE");

    let target_lib_dir = target_bin_dir.join("lib");
    fs::create_dir_all(&target_lib_dir).ok();

    let sdl3_lib_names: &[&str] = if is_macos {
        &["libSDL3.0.dylib", "libSDL3.dylib"]
    } else {
        &["libSDL3.so", "libSDL3.so.0"]
    };
    for lib in sdl3_lib_names {
        let src = sdl3_install.join("lib").join(lib);
        if src.exists() {
            symlink::symlink_obj_or_copy(&src, &target_lib_dir.join(lib));
        }
    }

    if let Ok(entries) = fs::read_dir(sdl3_image_install.join("lib")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name()
                && let Some(name_str) = name.to_str()
                && name_str.starts_with("libSDL3_image") && (Path::new(name_str).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("so")) || Path::new(name_str).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("dylib")) || name_str.contains(".so.")) {
                symlink::symlink_obj_or_copy(&path, &target_lib_dir.join(name));
            }
        }
    }

    if let Ok(entries) = fs::read_dir(ffmpeg_install.join("lib")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name()
                && let Some(name_str) = name.to_str()
                && name_str.starts_with("lib") && (Path::new(name_str).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("so")) || Path::new(name_str).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("dylib")) || name_str.contains(".so.")) {
                symlink::symlink_obj_or_copy(&path, &target_lib_dir.join(name));
            }
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
}

#[cfg(unix)]
mod symlink {
    use std::fs;
    use std::path::Path;
    use std::os::unix::fs::symlink;

    pub fn symlink_obj_or_copy(src: &Path, dst: &Path) {
        if dst.exists() || dst.is_symlink() {
            fs::remove_file(dst).ok();
        }
        if symlink(src, dst).is_err() {
            fs::copy(src, dst).ok();
        }
    }
}

#[cfg(windows)]
mod symlink {
    use std::fs;
    use std::path::Path;

    pub fn symlink_obj_or_copy(src: &Path, dst: &Path) {
        fs::copy(src, dst).ok();
    }
}
