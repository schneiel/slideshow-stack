#!/bin/bash
set -e

cd "$(dirname "$0")/../.."

HOST_ARCH="$(rustc -vV | grep '^host:' | cut -d' ' -f2)"
CACHE_DIR="$(pwd)/deploy/.cache/downloads"

RUST_TARGET=""
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--target) RUST_TARGET="$2"; shift 2 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

if [[ -z "$RUST_TARGET" ]]; then
    RUST_TARGET="$HOST_ARCH"
fi

if [[ -z "$RUST_TARGET" ]]; then
    echo "Error: --target is required"
    echo "Usage: $0 -t <rust-target-triple>"
    exit 1
fi

ARCH_CACHE_DIR="$CACHE_DIR/$RUST_TARGET"
DOWNLOAD_DIR="$ARCH_CACHE_DIR"

ZENOH_VERSION="1.9.0"
CADDY_VERSION="2.8.4"

CADDY_ARCH_MAP="
aarch64-unknown-linux-gnu:linux_arm64
armv7-unknown-linux-gnueabihf:linux_armv7
x86_64-unknown-linux-gnu:linux_amd64
"

get_caddy_arch() {
    local target="$1"
    while IFS=: read -r rust_part caddy_part; do
        [[ "$rust_part" == "$target" ]] && echo "$caddy_part" && return
    done <<< "$CADDY_ARCH_MAP"
    echo ""
}

CADDY_ARCH=$(get_caddy_arch "$RUST_TARGET")
if [[ -z "$CADDY_ARCH" ]]; then
    echo "Error: No caddy download URL for target $RUST_TARGET"
    exit 1
fi

mkdir -p "$DOWNLOAD_DIR"

ZENOH_URL="https://github.com/eclipse-zenoh/zenoh/releases/download/${ZENOH_VERSION}/zenoh-${ZENOH_VERSION}-${RUST_TARGET}-standalone.zip"
CADDY_URL="https://github.com/caddyserver/caddy/releases/download/v${CADDY_VERSION}/caddy_${CADDY_VERSION}_${CADDY_ARCH}.tar.gz"

if [[ ! -f "$DOWNLOAD_DIR/zenohd" ]] || [[ ! -f "$DOWNLOAD_DIR/libzenoh_plugin_rest.so" ]]; then
    echo "Downloading zenoh ${ZENOH_VERSION} for ${RUST_TARGET}..."
    curl -sL "$ZENOH_URL" -o /tmp/zenoh.zip
    mkdir -p /tmp/zenoh-extract
    unzip -o /tmp/zenoh.zip -d /tmp/zenoh-extract
    cp /tmp/zenoh-extract/zenohd "$DOWNLOAD_DIR/zenohd"
    chmod +x "$DOWNLOAD_DIR/zenohd"
    for so in /tmp/zenoh-extract/libzenoh_plugin_*.so; do
        if [[ -f "$so" ]]; then
            cp "$so" "$DOWNLOAD_DIR/$(basename $so)"
            chmod +x "$DOWNLOAD_DIR/$(basename $so)"
        fi
    done
    rm -rf /tmp/zenoh.zip /tmp/zenoh-extract
fi

if [[ ! -f "$DOWNLOAD_DIR/caddy" ]]; then
    echo "Downloading caddy ${CADDY_VERSION} for ${RUST_TARGET}..."
    curl -sL "$CADDY_URL" -o /tmp/caddy.tar.gz
    tar -xzf /tmp/caddy.tar.gz -C /tmp/
    cp /tmp/caddy "$DOWNLOAD_DIR/caddy"
    chmod +x "$DOWNLOAD_DIR/caddy"
    rm -rf /tmp/caddy.tar.gz /tmp/caddy
fi

echo ""
echo "Done!"
echo "External binaries at: deploy/.cache/downloads/${RUST_TARGET}/"