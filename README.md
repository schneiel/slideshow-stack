# Slideshow Stack

[![License: CC BY-NC 4.0](https://img.shields.io/badge/License-CC%20BY--NC%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by-nc/4.0/)

Distributed digital signage solution for managing and displaying multimedia content across multiple display clients.


## Apps

### Control Panel (`apps/control-panel`)

SvelteKit Web UI for managing media, slideshows, and controlling playback clients.

### Playback Server (`apps/playback-server`)

Rust server that routes commands from the control panel to playback clients via Zenoh.

### Playback Client (`apps/playback-client`)

Rust client that renders slideshows and videos on displays using SDL3.

Supports two rendering modes:
- `desktop` (default) - SDL3 for standard displays
- `drm` - DRM for embedded/headless displays

### Store (`apps/store`)

Rust server for media file storage and slideshow metadata.

## Development

### Prerequisites

- Rust 1.93+
- Bun

## Project Structure

```
slideshow-stack/
├── apps/
│   ├── control-panel/     # SvelteKit web UI
│   ├── playback-client/   # Rust SDL3 client
│   ├── playback-server/   # Rust command router
│   └── store/             # Rust REST API
├── libs/                  # Shared Rust libraries
├── deploy/                # Deployment configs
```
