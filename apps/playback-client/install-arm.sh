#!/bin/bash
set -e

# All apt operations first
sudo apt update
sudo apt install -y build-essential cmake make pkg-config
sudo dpkg --add-architecture armhf
sudo apt-get update
sudo apt-get install -y \
    clang llvm-dev libclang-dev nasm yasm \
    gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf libc6-dev-armhf-cross \
    libssl-dev:armhf zlib1g-dev:armhf \
    libdrm-dev:armhf libgbm-dev:armhf libegl-dev:armhf \
    libx264-dev:armhf libmp3lame-dev:armhf libopus-dev:armhf libvpx-dev:armhf \
    pkg-config-arm-linux-gnueabihf

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"

# Add ARM target
rustup target add armv7-unknown-linux-gnueabihf

# Install Zig
curl -sO https://ziglang.org/download/0.16.0/zig-x86_64-linux-0.16.0.tar.xz
tar -xf zig-x86_64-linux-0.16.0.tar.xz
sudo mv zig-x86_64-linux-0.16.0 /opt/zig
rm zig-x86_64-linux-0.16.0.tar.xz
echo 'export PATH="/opt/zig:$PATH"' >> ~/.bashrc

# Install cargo-zigbuild
cargo install cargo-zigbuild

# Build for ARM
export PATH="/opt/zig:$PATH"
cargo build --release --target armv7-unknown-linux-gnueabihf --features drm
