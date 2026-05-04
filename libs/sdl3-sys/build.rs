use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let is_macos = target_os == "macos";
    let is_linux = target_os == "linux";
    let is_drm = env::var("CARGO_FEATURE_DRM").is_ok();
    let is_desktop = env::var("CARGO_FEATURE_DESKTOP").is_ok();

    println!("cargo:rerun-if-changed={}", manifest_dir.join("build.rs").display());
    println!("cargo:rerun-if-changed={}", manifest_dir.join("sdl3").display());
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_DRM");

    let sdl3_path = manifest_dir.join("sdl3");
    let sdl3_install = out_dir.join("install");
    let sdl3_cmake_config = sdl3_install.join("lib/cmake/SDL3/SDL3Config.cmake");

    if !sdl3_cmake_config.exists() {
        if target == "armv7-unknown-linux-gnueabihf" {
            env::set_var("PKG_CONFIG_PATH", "/usr/lib/arm-linux-gnueabihf/pkgconfig");
        }
        fs::create_dir_all(&sdl3_install).unwrap();

        let mut sdl3_cmake = cmake::Config::new(&sdl3_path);
        sdl3_cmake
            .define("CMAKE_INSTALL_PREFIX", sdl3_install.to_str().unwrap())
            .out_dir(out_dir.join("build"))
            .define("SDL_SHARED", "ON")
            .define("SDL_STATIC", "OFF")
            .define("SDL_STATIC_PIC", "ON")
            .define("SDL_TESTS", "OFF");
        if is_drm {
            sdl3_cmake
                .define("SDL_KMSDRM", "ON")
                .define("SDL_X11", "OFF")
                .define("SDL_WAYLAND", "OFF")
                .define("SDL_UNIX_CONSOLE_BUILD", "ON")
                .define("SDL_OPENGLES", "ON")
                .define("SDL_VIDEO", "ON");
        } else if is_desktop && is_linux {
            sdl3_cmake
                .define("SDL_X11", "ON")
                .define("SDL_WAYLAND", "ON")
                .define("SDL_KMSDRM", "OFF");
        }
        let sdl3_build = sdl3_cmake.build();
        let mut install_cmd = std::process::Command::new("cmake");
        install_cmd.args(["--install", &sdl3_build.to_string_lossy()]);
        install_cmd.output().expect("Failed to install SDL3");
    }

    let include_path = sdl3_install.join("include");
    let bindings_path = out_dir.join("bindings.rs");

    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", include_path.display()))
        .header(format!("{}/SDL3/SDL.h", include_path.display()))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(&bindings_path)
        .expect("Couldn't write bindings");

    println!("cargo:rustc-link-search=native={}", sdl3_install.join("lib").display());
    println!("cargo:rustc-link-lib=dylib=SDL3");
    println!("cargo:include={}", sdl3_install.join("include").display());

    if is_macos {
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=AudioToolbox");
    } else if is_linux && is_drm {
        println!("cargo:rustc-link-lib=drm");
        println!("cargo:rustc-link-lib=gbm");
        println!("cargo:rustc-link-lib=EGL");
        if target == "armv7-unknown-linux-gnueabihf" {
            println!("cargo:rustc-link-search=native=/usr/lib/arm-linux-gnueabihf");
            println!("cargo:rustc-link-search=native=/usr/lib/arm-linux-gnueabihf/dri");
        }
    }
}
