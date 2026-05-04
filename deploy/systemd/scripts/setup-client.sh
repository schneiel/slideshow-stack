#!/bin/bash
set -e

INSTALL_DIR="/opt/slideshow-client"
CONFIG_DIR="/etc/slideshow-client"
BIN_DIR="/usr/local/bin"
RUN_AS="${RUN_AS:-pi}"
DEPLOY_DIR=""
ZENOH_ENDPOINTS="udp/zenohd:7447"
SYNC_SERVER_URL="http://localhost:61532"
TARGET_FPS="30"

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "  --install-dir DIR       Installation directory (default: /opt/slideshow-client)"
    echo "  --config-dir DIR       Config directory (default: /etc/slideshow-client)"
    echo "  --bin-dir DIR           Binary directory (default: /usr/local/bin)"
    echo "  --run-as USER           User to run service (default: pi)"
    echo "  --deploy-dir DIR        Deployment directory (required)"
    echo "  --zenoh-endpoint EP     Zenoh endpoint (default: udp/zenohd:7447)"
    echo "  --sync-server-url URL   Sync server URL (default: http://localhost:61532)"
    echo "  --target-fps FPS        Target FPS (default: 30)"
    echo "  --help                  Show this help"
    exit 1
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --install-dir) INSTALL_DIR="$2"; shift 2 ;;
        --config-dir) CONFIG_DIR="$2"; shift 2 ;;
        --bin-dir) BIN_DIR="$2"; shift 2 ;;
        --run-as) RUN_AS="$2"; shift 2 ;;
        --deploy-dir) DEPLOY_DIR="$2"; shift 2 ;;
        --zenoh-endpoint) ZENOH_ENDPOINTS="$2"; shift 2 ;;
        --sync-server-url) SYNC_SERVER_URL="$2"; shift 2 ;;
        --target-fps) TARGET_FPS="$2"; shift 2 ;;
        --help) usage ;;
        *) echo "Unknown option: $1"; usage ;;
    esac
done

if [[ -z "$DEPLOY_DIR" ]]; then
    echo "Error: --deploy-dir is required"
    usage
fi

echo "=== Slideshow Client Installation ==="
echo "  Install dir:       $INSTALL_DIR"
echo "  Config dir:        $CONFIG_DIR"
echo "  Binary dir:        $BIN_DIR"
echo "  Run as:            $RUN_AS"
echo "  Zenoh endpoint:    $ZENOH_ENDPOINTS"
echo "  Sync server URL:   $SYNC_SERVER_URL"
echo "  Target FPS:        $TARGET_FPS"

for binary in playback-client; do
    if [[ ! -f "$DEPLOY_DIR/$binary" ]]; then
        echo "Error: Missing binary $binary in $DEPLOY_DIR"
        exit 1
    fi
done

BUILD_DIR=$(mktemp -d)
trap "rm -rf $BUILD_DIR" EXIT

echo ""
echo "Copying pre-built binaries..."

cp "$DEPLOY_DIR/playback-client" "$BUILD_DIR/slideshow-playback-client"
if [[ -d "$DEPLOY_DIR/lib" ]]; then
    cp -r "$DEPLOY_DIR/lib" "$BUILD_DIR/"
fi

echo ""
echo "Stopping service for update..."
sudo systemctl stop slideshow-playback-client.service 2>/dev/null || true
sleep 1

echo ""
echo "Installing to $INSTALL_DIR..."
sudo mkdir -p "$INSTALL_DIR"
sudo rsync -a "$BUILD_DIR/" "$INSTALL_DIR/"
sudo chmod +x "$INSTALL_DIR/slideshow-playback-client"

echo ""
echo "Creating runtime directories..."
sudo mkdir -p "$INSTALL_DIR/media"
sudo chown -R "$RUN_AS:$RUN_AS" "$INSTALL_DIR"

echo ""
echo "Installing systemd service..."
sudo mkdir -p /etc/systemd/system
sed "s|%RUN_AS%|$RUN_AS|g; s|%INSTALL_DIR%|$INSTALL_DIR|g; s|%CONFIG_DIR%|$CONFIG_DIR|g; s|%ZENOH_ENDPOINTS%|$ZENOH_ENDPOINTS|g; s|%SYNC_SERVER_URL%|$SYNC_SERVER_URL|g; s|%TARGET_FPS%|$TARGET_FPS|g" \
    "$DEPLOY_DIR/services/slideshow-playback-client.service" | sudo tee /etc/systemd/system/slideshow-playback-client.service > /dev/null

echo ""
echo "Reloading systemd and starting service..."
sudo systemctl daemon-reload
sudo systemctl enable slideshow-playback-client.service
sudo systemctl start slideshow-playback-client.service

echo ""
echo "=== Installation Complete ==="
echo ""
echo "View logs:"
echo "  sudo journalctl -u slideshow-playback-client -f"