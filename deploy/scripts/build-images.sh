#!/bin/bash
set -e

REPO_DIR="$(cd "$(dirname "$0")/../.." && pwd)"

if command -v podman &> /dev/null; then
    CONTAINER_CMD="podman"
elif command -v docker &> /dev/null; then
    CONTAINER_CMD="docker"
else
    echo "Error: Neither podman nor docker found"
    exit 1
fi

echo "Building slideshow-stack container images with $CONTAINER_CMD..."

cd "$REPO_DIR"

$CONTAINER_CMD build -t localhost/slideshow-store:latest -f deploy/containers/store/Containerfile apps/store
$CONTAINER_CMD build -t localhost/slideshow-playback-server:latest -f deploy/containers/playback-server/Containerfile apps/playback-server
$CONTAINER_CMD build -t localhost/slideshow-control-panel:latest -f deploy/containers/control-panel/Containerfile apps/control-panel

echo "Done!"