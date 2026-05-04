#!/bin/bash
set -e

PROJECT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
SSH_HOST="${SSH_HOST:-pi@raspberrypi}"
SSH_PORT="${SSH_PORT:-22}"
RUN_AS="${RUN_AS:-pi}"

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "  -t, --target ARCH   Target architecture (rust target triple)"
    echo "  --ssh-host HOST     SSH host to deploy to (default: pi@raspberrypi)"
    echo "  --ssh-port PORT     SSH port (default: 22)"
    echo "  --run-as USER       User to run services (default: pi)"
    echo "  --help              Show this help"
    exit 1
}

RUST_TARGET=""
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--target) RUST_TARGET="$2"; shift 2 ;;
        --ssh-host) SSH_HOST="$2"; shift 2 ;;
        --ssh-port) SSH_PORT="$2"; shift 2 ;;
        --run-as) RUN_AS="$2"; shift 2 ;;
        --help) usage ;;
        *) echo "Unknown option: $1"; usage ;;
    esac
done

if [[ -z "$RUST_TARGET" ]]; then
    echo "Error: --target is required"
    usage
fi

CACHE_DIR="$PROJECT_DIR/deploy/.cache/downloads/$RUST_TARGET"

SSH_CMD="ssh -o ControlMaster=auto -o ControlPath=/tmp/deploy-ssh-%r@%h -o ControlPersist=10m -p $SSH_PORT"

STORE_BINARY="$PROJECT_DIR/apps/store/target/$RUST_TARGET/release/store"
PLAYBACK_BINARY="$PROJECT_DIR/apps/playback-server/target/$RUST_TARGET/release/playback-server"

if [[ ! -f "$STORE_BINARY" ]]; then
    echo "Error: No store build found at $STORE_BINARY"
    echo "Build with: cargo zigbuild --target $RUST_TARGET --release"
    exit 1
fi

if [[ ! -f "$PLAYBACK_BINARY" ]]; then
    echo "Error: No playback-server build found at $PLAYBACK_BINARY"
    echo "Build with: cargo zigbuild --target $RUST_TARGET --release"
    exit 1
fi

if [[ ! -d "$PROJECT_DIR/apps/control-panel/build" ]]; then
    echo "Error: No control panel build found at apps/control-panel/build"
    echo "Build with: cd apps/control-panel && bun install && bun run build"
    exit 1
fi

if [[ ! -d "$CACHE_DIR" ]]; then
    echo "Error: No cache directory found at $CACHE_DIR"
    echo "Run the download script first to fetch Caddy and Zenoh binaries"
    exit 1
fi

if [[ ! -f "$CACHE_DIR/caddy" ]]; then
    echo "Error: No caddy binary found at $CACHE_DIR/caddy"
    exit 1
fi

if [[ ! -f "$CACHE_DIR/zenohd" ]]; then
    echo "Error: No zenohd binary found at $CACHE_DIR/zenohd"
    exit 1
fi

echo "=== Slideshow Stack Deployment ==="
echo "  Target:     $RUST_TARGET"
echo "  SSH host:   $SSH_HOST"
echo "  SSH port:   $SSH_PORT"
echo "  Run as:     $RUN_AS"
echo ""

echo "Step 1: Creating staging directory on remote server..."
$SSH_CMD "$SSH_HOST" "rm -rf /tmp/slideshow-stack-deploy && mkdir -p /tmp/slideshow-stack-deploy"

echo ""
echo "Step 2: Syncing Rust binaries..."
$SSH_CMD "$SSH_HOST" "mkdir -p /tmp/slideshow-stack-deploy"
rsync -avz --progress -e "$SSH_CMD" \
    "$STORE_BINARY" \
    "$PLAYBACK_BINARY" \
    "$SSH_HOST:/tmp/slideshow-stack-deploy/"

echo ""
echo "Step 3: Syncing control panel..."
rsync -avz --progress -e "$SSH_CMD" \
    "$PROJECT_DIR/apps/control-panel/build/" \
    "$SSH_HOST:/tmp/slideshow-stack-deploy/control-panel/"

echo ""
echo "Step 4: Syncing Caddy and Zenoh..."
rsync -avz --progress -e "$SSH_CMD" \
    "$CACHE_DIR/" \
    "$SSH_HOST:/tmp/slideshow-stack-deploy/"

echo ""
echo "Step 5: Syncing deployment files..."
rsync -avz --progress -e "$SSH_CMD" \
    "$PROJECT_DIR/deploy/systemd/" \
    "$SSH_HOST:/tmp/slideshow-stack-deploy/"

echo ""
echo "Step 6: Running setup on remote server..."
$SSH_CMD "$SSH_HOST" \
    "sudo /tmp/slideshow-stack-deploy/scripts/setup-server.sh --deploy-dir /tmp/slideshow-stack-deploy -t $RUST_TARGET --run-as $RUN_AS"

echo ""
echo "Step 7: Cleaning up SSH ControlMaster..."
$SSH_CMD -O exit "$SSH_HOST" 2>/dev/null || true

echo ""
echo "=== Deployment Complete ==="
