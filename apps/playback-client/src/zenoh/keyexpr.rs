pub fn device_command_key(device_id: &str) -> String {
    format!("slideshow/device/{device_id}/command")
}

pub const fn devices_command_key() -> &'static str {
    "slideshow/devices/command"
}

pub fn device_state_key(device_id: &str) -> String {
    format!("slideshow/state/{device_id}")
}

pub fn device_video_state_key(device_id: &str) -> String {
    format!("slideshow/video/state/{device_id}")
}
