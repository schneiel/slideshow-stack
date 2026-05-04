use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target = env::var("TARGET").unwrap();
    let host_arch = std::env::consts::ARCH;
    let target_arch = if target.contains("armv7") || target.contains("aarch64") || target.contains("arm64") {
        if target.contains("armv7") { "arm".to_string() } else { "aarch64".to_string() }
    } else {
        env::var("CARGO_CFG_TARGET_ARCH").unwrap()
    };
    let is_cross = target_arch != host_arch;

    println!("cargo:rerun-if-changed={}", manifest_dir.join("build.rs").display());
    println!("cargo:rerun-if-changed={}", manifest_dir.join("ffmpeg").display());

    let ffmpeg_path = manifest_dir.join("ffmpeg");

    if !ffmpeg_path.exists() {
        println!("cargo:warning=FFmpeg submodule not found. Run: git submodule update --init");
        return;
    }

    let ffmpeg_install = out_dir.join("install");
    let ffmpeg_lib = if target_os == "macos" {
        ffmpeg_install.join("lib").join("libavutil.dylib")
    } else {
        ffmpeg_install.join("lib").join("libavutil.so")
    };

    if !ffmpeg_lib.exists() {
        let ffmpeg_build = out_dir.join("build");
        fs::create_dir_all(&ffmpeg_build).unwrap();
        fs::create_dir_all(&ffmpeg_install).unwrap();

        let config_h = ffmpeg_build.join("config.h");

        if !config_h.exists() {
            let mut cmd = std::process::Command::new(&ffmpeg_path.join("configure"));
            cmd.current_dir(&ffmpeg_build)
                .arg(format!("--prefix={}", ffmpeg_install.display()))
                .arg("--disable-doc")
                .arg("--disable-programs")
                .arg("--disable-static")
                .arg("--enable-shared")
                .arg("--enable-network")
                .arg("--enable-rpath")
                .arg("--disable-everything")
                .arg("--enable-protocol=file")
                .arg("--enable-demuxer=mov,mp4,m4v,avi,mkv")
                .arg("--enable-decoder=h264,mpeg4,mpegvideo");

            let mut make_env: HashMap<String, String> = HashMap::new();

            if is_cross && target_arch == "arm" {
                cmd.env("CC", "arm-linux-gnueabihf-gcc")
                    .env("CXX", "arm-linux-gnueabihf-g++")
                    .env("LD", "arm-linux-gnueabihf-ld")
                    .env("CFLAGS", "-fPIC -O2 -march=armv7-a -mfpu=vfpv3-d16 -mfloat-abi=hard")
                    .env("CXXFLAGS", "-fPIC -O2 -march=armv7-a -mfpu=vfpv3-d16 -mfloat-abi=hard")
                    .env("LDFLAGS", "-fPIC -march=armv7-a -mfpu=vfpv3-d16 -mfloat-abi=hard")
                    .arg("--cross-prefix=arm-linux-gnueabihf-")
                    .arg("--target-os=linux")
                    .arg("--arch=arm");

                make_env.insert("CC".to_string(), "arm-linux-gnueabihf-gcc".to_string());
                make_env.insert("CXX".to_string(), "arm-linux-gnueabihf-g++".to_string());
                make_env.insert("LD".to_string(), "arm-linux-gnueabihf-ld".to_string());
                make_env.insert("CFLAGS".to_string(), "-fPIC -O2 -march=armv7-a -mfpu=vfpv3-d16 -mfloat-abi=hard".to_string());
                make_env.insert("CXXFLAGS".to_string(), "-fPIC -O2 -march=armv7-a -mfpu=vfpv3-d16 -mfloat-abi=hard".to_string());
                make_env.insert("LDFLAGS".to_string(), "-fPIC -march=armv7-a -mfpu=vfpv3-d16 -mfloat-abi=hard".to_string());
            }

            let output = cmd.output().expect("FFmpeg configure failed");
            if !output.status.success() {
                eprintln!("cargo:warning=FFmpeg configure failed:");
                eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                return;
            }

            let mut make_cmd = std::process::Command::new("make");
            make_cmd.current_dir(&ffmpeg_build)
                .arg("-j")
                .arg(num_cpus::get().to_string())
                .arg("V=1");
            for (key, value) in &make_env {
                make_cmd.env(key, value);
            }
            let output = make_cmd.output().expect("FFmpeg make failed");
            if !output.status.success() {
                eprintln!("FFmpeg make failed:");
                eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                panic!("FFmpeg build failed - cannot continue");
            }

            let mut install_cmd = std::process::Command::new("make");
            install_cmd.current_dir(&ffmpeg_build)
                .arg("install");
            for (key, value) in &make_env {
                install_cmd.env(key, value);
            }
            install_cmd.output().expect("FFmpeg make install failed");

            fs::copy(ffmpeg_build.join("config.h"), config_h).ok();
        }
    }

    let bindings_path = out_dir.join("bindings.rs");

    let ffmpeg_include = if ffmpeg_lib.exists() {
        ffmpeg_install.join("include")
    } else {
        ffmpeg_path.join("include")
    };

    let mut bindings_builder = bindgen::Builder::default()
        .clang_arg(format!("-I{}", ffmpeg_include.display()))
        .header(format!("{}/libavutil/avutil.h", ffmpeg_include.display()))
        .header(format!("{}/libavcodec/avcodec.h", ffmpeg_include.display()))
        .header(format!("{}/libavformat/avformat.h", ffmpeg_include.display()))
        .header(format!("{}/libswscale/swscale.h", ffmpeg_include.display()))
        .header(format!("{}/libavfilter/avfilter.h", ffmpeg_include.display()))
        .header(format!("{}/libavdevice/avdevice.h", ffmpeg_include.display()))
        .blocklist_item("FP_NAN")
        .blocklist_item("FP_INFINITE")
        .blocklist_item("FP_ZERO")
        .blocklist_item("FP_SUBNORMAL")
        .blocklist_item("FP_NORMAL")
        .blocklist_function("_ZGV.*");

    if is_cross && target_arch == "arm" {
        bindings_builder = bindings_builder
            .clang_arg("--target=arm-linux-gnueabihf")
            .clang_arg("-march=armv7-a")
            .clang_arg("-mfpu=vfpv3-d16")
            .clang_arg("-mfloat-abi=hard");
    }

    let bindings = bindings_builder
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(&bindings_path)
        .expect("Couldn't write bindings");

    println!("cargo:rustc-link-search=native={}", ffmpeg_install.join("lib").display());
    println!("cargo:rustc-link-lib=dylib=avutil");
    println!("cargo:rustc-link-lib=dylib=avformat");
    println!("cargo:rustc-link-lib=dylib=avcodec");
    println!("cargo:rustc-link-lib=dylib=swscale");
    println!("cargo:rustc-link-lib=dylib=avfilter");
    println!("cargo:rustc-link-lib=dylib=avdevice");
    println!("cargo:rustc-link-lib=dylib=z");
    println!("cargo:include={}", ffmpeg_install.join("include").display());
}
