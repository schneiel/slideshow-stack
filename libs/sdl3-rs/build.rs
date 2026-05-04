use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let is_linux = target_os == "linux";
    let is_drm = env::var("CARGO_FEATURE_DRM").is_ok();

    if is_linux && is_drm {
        println!("cargo:rustc-link-lib=drm");
        println!("cargo:rustc-link-lib=gbm");
        println!("cargo:rustc-link-lib=EGL");
    }

    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_DRM");
}
