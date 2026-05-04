# Deploying Quadlets on Linux

## Prerequisites

- Podman installed
- systemd user session active
- SSH access to target server

## Deployment Steps

### 1. Copy Quadlets to Server

```bash
scp -r quadlets/ user@server:/home/user/.config/containers/systemd/
```

### 2. Reload systemd

```bash
systemctl --user daemon-reload
```

### 3. Start Services

```bash
systemctl --user start slideshow-store.service
systemctl --user start slideshow-zenoh.service
systemctl --user start slideshow-playback-server.service
systemctl --user start slideshow-control-panel.service
```

### 4. Enable and Start All at Once

```bash
systemctl --user enable --now slideshow-store.service slideshow-zenoh.service slideshow-playback-server.service slideshow-control-panel.service
```

### 5. Check Status

```bash
systemctl --user status slideshow-store
journalctl --user -u slideshow-store -f
```

## Troubleshooting

- Ensure user has an active session: `loginctl`
- Check Podman is in systemd PATH
- Verify network and volumes were created: `podman network ls`, `podman volume ls`
