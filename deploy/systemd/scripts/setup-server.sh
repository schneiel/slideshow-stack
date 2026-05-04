#!/bin/bash
set -e

PROJECT_DIR=""
INSTALL_DIR="/opt/slideshow"
STORE_DATA_DIR="${STORE_DATA_DIR:-/var/lib/slideshow-store}"
CONTROL_PANEL_DATA_DIR="${CONTROL_PANEL_DATA_DIR:-/var/lib/slideshow-control-panel}"
BIN_DIR="/usr/local/bin"
SYSTEMD_DIR="/etc/systemd/system"
CADDY_CONFIG_DIR="/etc/slideshow"
RUN_AS="${RUN_AS:-pi}"
DEPLOY_DIR=""
RUST_TARGET=""

HOST_ARCH=$(uname -m)

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "  --install-dir DIR    Installation directory (default: /opt/slideshow)"
    echo "  --data-dir DIR       Data directory (default: /var/lib/slideshow)"
    echo "  --bin-dir DIR        Binary directory (default: /usr/local/bin)"
    echo "  --run-as USER        User to run services (default: pi)"
    echo "  -t, --target ARCH   Target architecture (rust target triple)"
    echo "  --deploy-dir DIR     Deployment directory (default: auto-detect)"
    echo "  --help               Show this help"
    exit 1
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --install-dir) INSTALL_DIR="$2"; shift 2 ;;
        --data-dir) STORE_DATA_DIR="$2"; shift 2 ;;
        --bin-dir) BIN_DIR="$2"; shift 2 ;;
        --run-as) RUN_AS="$2"; shift 2 ;;
        --deploy-dir) DEPLOY_DIR="$2"; shift 2 ;;
        -t|--target) RUST_TARGET="$2"; shift 2 ;;
        --help) usage ;;
        *) echo "Unknown option: $1"; usage ;;
    esac
done

if [[ -z "$RUST_TARGET" ]]; then
    RUST_TARGET="$HOST_ARCH"
fi

if [[ -n "$DEPLOY_DIR" ]]; then
    PROJECT_DIR="$DEPLOY_DIR"
else
    PROJECT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
fi

echo "=== Slideshow Stack Installation (Root Level) ==="
echo "  Target:             $RUST_TARGET"
echo "  Install dir:        $INSTALL_DIR"
echo "  Store data dir:     $STORE_DATA_DIR"
echo "  Control panel dir:  $CONTROL_PANEL_DATA_DIR"
echo "  Binary dir:         $BIN_DIR"
echo "  Run as:             $RUN_AS"

for binary in store playback-server; do
    if [[ ! -f "$DEPLOY_DIR/$binary" ]]; then
        echo "Error: Missing binary $binary in $DEPLOY_DIR"
        exit 1
    fi
done

if [[ ! -d "$DEPLOY_DIR/control-panel" ]]; then
    echo "Error: Missing control-panel directory in $DEPLOY_DIR"
    exit 1
fi

BUILD_DIR=$(mktemp -d)
trap "rm -rf $BUILD_DIR" EXIT

echo ""
echo "Copying pre-built binaries..."

cp "$DEPLOY_DIR/store" "$BUILD_DIR/slideshow-store"
cp "$DEPLOY_DIR/playback-server" "$BUILD_DIR/slideshow-playback-server"

for so in "$DEPLOY_DIR"/libzenoh_plugin_*.so; do
    if [[ -f "$so" ]]; then
        cp "$so" "$BUILD_DIR/"
    fi
done

echo ""
echo "Stopping services for update..."
sudo systemctl stop slideshow-zenoh.service 2>/dev/null || true
sudo systemctl stop slideshow-store.service 2>/dev/null || true
sudo systemctl stop slideshow-playback-server.service 2>/dev/null || true
sudo systemctl stop slideshow-control-panel.service 2>/dev/null || true
sleep 1

echo ""
echo "Installing to $INSTALL_DIR..."
sudo mkdir -p "$INSTALL_DIR"
sudo mkdir -p "$STORE_DATA_DIR/media"
sudo mkdir -p "$CONTROL_PANEL_DATA_DIR"
sudo rsync -a "$BUILD_DIR/" "$INSTALL_DIR/"
sudo chmod +x "$INSTALL_DIR/slideshow-store" "$INSTALL_DIR/slideshow-playback-server"

echo ""
echo "Installing control panel files..."
sudo rsync -a "$DEPLOY_DIR/control-panel/" "$CONTROL_PANEL_DATA_DIR/"

echo ""
echo "Installing binaries..."
sudo mkdir -p "$BIN_DIR"
sudo install -m 755 "$INSTALL_DIR/slideshow-store" "$BIN_DIR/slideshow-store"
sudo install -m 755 "$INSTALL_DIR/slideshow-playback-server" "$BIN_DIR/slideshow-playback-server"
sudo install -m 755 "$INSTALL_DIR/caddy" "$BIN_DIR/caddy"

echo ""
echo "Installing zenoh..."
sudo install -m 755 "$INSTALL_DIR/zenohd" "$BIN_DIR/zenohd"
sudo mkdir -p "$BIN_DIR/../lib"
sudo install -m 755 "$INSTALL_DIR/libzenoh_plugin_rest.so" "$BIN_DIR/../lib/" 2>/dev/null || true
sudo install -m 755 "$INSTALL_DIR/libzenoh_plugin_storage_manager.so" "$BIN_DIR/../lib/" 2>/dev/null || true

echo ""
echo "Installing systemd services..."
sudo mkdir -p "$SYSTEMD_DIR"
for svc in slideshow-zenoh slideshow-playback-server slideshow-store; do
    sed "s|%RUN_AS%|$RUN_AS|g; s|%BIN_DIR%|$BIN_DIR|g; s|%STORE_DATA_DIR%|$STORE_DATA_DIR|g" \
        "$DEPLOY_DIR/services/$svc.service" | sudo tee "$SYSTEMD_DIR/$svc.service" > /dev/null
done
sed "s|%RUN_AS%|$RUN_AS|g; s|%BIN_DIR%|$BIN_DIR|g; s|%CONTROL_PANEL_DATA_DIR%|$CONTROL_PANEL_DATA_DIR|g; s|%CADDY_CONFIG_DIR%|$CADDY_CONFIG_DIR|g" \
    "$DEPLOY_DIR/services/slideshow-control-panel.service" | sudo tee "$SYSTEMD_DIR/slideshow-control-panel.service" > /dev/null
sed "s|%RUN_AS%|$RUN_AS|g; s|%BIN_DIR%|$BIN_DIR|g" \
    "$DEPLOY_DIR/services/slideshow-network.service" | sudo tee "$SYSTEMD_DIR/slideshow-network.service" > /dev/null

echo "Installing Caddy config..."
sudo mkdir -p "$CADDY_CONFIG_DIR"
sudo cp "$DEPLOY_DIR/Caddyfile" "$CADDY_CONFIG_DIR/Caddyfile"

echo "Setting permissions..."
sudo chown -R root:root "$INSTALL_DIR"
sudo chown -R "$RUN_AS:$RUN_AS" "$STORE_DATA_DIR"
sudo chown -R "$RUN_AS:$RUN_AS" "$CONTROL_PANEL_DATA_DIR"

echo ""
echo "Reloading systemd and starting services..."
sudo systemctl daemon-reload
sudo systemctl enable slideshow-network.service
sudo systemctl enable slideshow-zenoh.service
sudo systemctl enable slideshow-store.service
sudo systemctl enable slideshow-playback-server.service
sudo systemctl enable slideshow-control-panel.service
sudo systemctl start slideshow-network.service
sudo systemctl start slideshow-zenoh.service
sudo systemctl start slideshow-store.service
sudo systemctl start slideshow-playback-server.service
sudo systemctl start slideshow-control-panel.service

echo ""
echo "=== Installation Complete ==="
echo ""
echo "View logs:"
echo "  sudo journalctl -u slideshow-network -f"
echo "  sudo journalctl -u slideshow-zenoh -f"
echo "  sudo journalctl -u slideshow-store -f"
echo "  sudo journalctl -u slideshow-playback-server -f"
echo "  sudo journalctl -u slideshow-control-panel -f"
