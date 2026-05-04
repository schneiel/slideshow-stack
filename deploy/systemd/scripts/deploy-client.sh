#!/bin/bash
set -e

PROJECT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
SSH_HOST="${SSH_HOST:-pi@clientpi}"
SSH_PORT="${SSH_PORT:-22}"
RUN_AS="${RUN_AS:-pi}"
ZENOH_ENDPOINT="${ZENOH_ENDPOINT:-udp/zenohd:7447}"
SYNC_SERVER_URL="${SYNC_SERVER_URL:-http://localhost:61532}"
TARGET_FPS="${TARGET_FPS:-30}"

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "  -t, --target ARCH       Target architecture (rust target triple)"
    echo "  --ssh-host HOST         SSH host to deploy to (default: pi@clientpi)"
    echo "  --ssh-port PORT         SSH port (default: 22)"
    echo "  --run-as USER           User to run services (default: pi)"
    echo "  --zenoh-endpoint EP     Zenoh endpoint (default: udp/zenohd:7447)"
    echo "  --sync-server-url URL   Sync server URL (default: http://localhost:61532)"
    echo "  --target-fps FPS        Target FPS (default: 30)"
    echo "  --help                  Show this help"
    exit 1
}

RUST_TARGET=""
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--target) RUST_TARGET="$2"; shift 2 ;;
        --ssh-host) SSH_HOST="$2"; shift 2 ;;
        --ssh-port) SSH_PORT="$2"; shift 2 ;;
        --run-as) RUN_AS="$2"; shift 2 ;;
        --zenoh-endpoint) ZENOH_ENDPOINT="$2"; shift 2 ;;
        --sync-server-url) SYNC_SERVER_URL="$2"; shift 2 ;;
        --target-fps) TARGET_FPS="$2"; shift 2 ;;
        --help) usage ;;
        *) echo "Unknown option: $1"; usage ;;
    esac
done

if [[ -z "$RUST_TARGET" ]]; then
    echo "Error: --target is required"
    usage
fi

SSH_CMD="ssh -o ControlMaster=auto -o ControlPath=/tmp/deploy-ssh-%r@%h -o ControlPersist=10m -p $SSH_PORT"

PLAYBACK_CLIENT_BINARY="$PROJECT_DIR/apps/playback-client/target/$RUST_TARGET/release/playback-client"
PLAYBACK_CLIENT_LIB="$PROJECT_DIR/apps/playback-client/target/$RUST_TARGET/release/lib"

if [[ ! -f "$PLAYBACK_CLIENT_BINARY" ]]; then
    echo "Error: No playback-client build found at $PLAYBACK_CLIENT_BINARY"
    echo "Build with: cargo zigbuild --target $RUST_TARGET --release --features drm"
    exit 1
fi

echo "=== Slideshow Client Deployment ==="
echo "  Target:          $RUST_TARGET"
echo "  SSH host:        $SSH_HOST"
echo "  SSH port:        $SSH_PORT"
echo "  Run as:          $RUN_AS"
echo "  Zenoh endpoint:  $ZENOH_ENDPOINT"
echo "  Sync server:     $SYNC_SERVER_URL"
echo "  Target FPS:      $TARGET_FPS"
echo ""

echo "Step 1: Creating staging directory on remote server..."
$SSH_CMD "$SSH_HOST" "rm -rf /tmp/slideshow-client-deploy && mkdir -p /tmp/slideshow-client-deploy"

echo ""
echo "Step 2: Syncing playback-client binary and libraries..."
rsync -avz --progress -e "$SSH_CMD" \
    "$PLAYBACK_CLIENT_BINARY" \
    "$PLAYBACK_CLIENT_LIB" \
    "$SSH_HOST:/tmp/slideshow-client-deploy/"

echo ""
echo "Step 3: Syncing deployment files..."
rsync -avz --progress -e "$SSH_CMD" \
    "$PROJECT_DIR/deploy/systemd/" \
    "$SSH_HOST:/tmp/slideshow-client-deploy/"

echo ""
echo "Step 4: Running setup on remote server..."
$SSH_CMD "$SSH_HOST" \
    "sudo /tmp/slideshow-client-deploy/scripts/setup-client.sh \
        --deploy-dir /tmp/slideshow-client-deploy \
        --run-as $RUN_AS \
        --zenoh-endpoint '$ZENOH_ENDPOINT' \
        --sync-server-url '$SYNC_SERVER_URL' \
        --target-fps $TARGET_FPS"

echo ""
echo "Step 5: Cleaning up SSH ControlMaster..."
$SSH_CMD -O exit "$SSH_HOST" 2>/dev/null || true

echo ""
echo "=== Deployment Complete ==="