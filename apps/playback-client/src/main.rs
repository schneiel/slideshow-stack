mod autostart;
mod config;
mod slideshow_executor;
mod sync;
mod video_player;
mod zenoh;

use std::sync::Arc;

use std::time::{Duration, Instant};
use anyhow::Context;

use tracing::{debug, info, warn, trace, error};

use autostart::load_autostart_config;
use config::load_config;
use slideshow_executor::{Command, Executor, StartSlideshowData};
use slideshow_rs::{ScalingMode, Slideshow, SlideshowConfig};
use sdl3_rs::{Renderer, Sdl3Renderer};
use sync::SyncService;
use video_player::{VideoPlayer, VideoCommand};
use zenoh::{ZenohManager, SlideshowState, VideoState};

struct MainThreadExecutor<'a, R: Renderer>(slideshow_executor::Executor<'a, R>);

unsafe extern "C" fn sdl3_log_callback(
    _userdata: *mut std::ffi::c_void,
    category: i32,
    priority: sdl3_rs::SDL_LogPriority,
    message: *const std::ffi::c_char,
) {
    let msg = unsafe { std::ffi::CStr::from_ptr(message) }
        .to_string_lossy()
        .into_owned();
    let cat_str = match category {
        0 => "app",
        1 => "error",
        2 => "system",
        3 => "audio",
        4 => "video",
        5 => "render",
        6 => "input",
        _ => "custom",
    };
    match priority {
        sdl3_rs::SDL_LogPriority_SDL_LOG_PRIORITY_TRACE => trace!(target: "sdl3", "[{}] {}", cat_str, msg),
        sdl3_rs::SDL_LogPriority_SDL_LOG_PRIORITY_VERBOSE => trace!(target: "sdl3", "[{}] {}", cat_str, msg),
        sdl3_rs::SDL_LogPriority_SDL_LOG_PRIORITY_DEBUG => debug!(target: "sdl3", "[{}] {}", cat_str, msg),
        sdl3_rs::SDL_LogPriority_SDL_LOG_PRIORITY_INFO => info!(target: "sdl3", "[{}] {}", cat_str, msg),
        sdl3_rs::SDL_LogPriority_SDL_LOG_PRIORITY_WARN => warn!(target: "sdl3", "[{}] {}", cat_str, msg),
        sdl3_rs::SDL_LogPriority_SDL_LOG_PRIORITY_ERROR => error!(target: "sdl3", "[{}] {}", cat_str, msg),
        sdl3_rs::SDL_LogPriority_SDL_LOG_PRIORITY_CRITICAL => error!(target: "sdl3", "[{}] CRITICAL: {}", cat_str, msg),
        _ => debug!(target: "sdl3", "[{}] {}", cat_str, msg),
    }
}

#[allow(clippy::too_many_lines)]
fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    let cfg = load_config()?;
    cfg.print();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&cfg.log_level)),
        )
        .init();

    info!("Starting playback client, log level: {}", cfg.log_level);

    let sdl3_log_level: i32 = match cfg.log_level.to_lowercase().as_str() {
        "trace" | "debug" => 0,
        "error" | "warn" => 5,
        _ => 4,
    };

    let mut renderer = Sdl3Renderer::new();

    renderer.set_log_output_function(Some(sdl3_log_callback), std::ptr::null_mut());
    renderer.set_trace_log_level(sdl3_log_level);

    let display_width = cfg.display_width.unwrap_or(1280);
    let display_height = cfg.display_height.unwrap_or(720);

    renderer.set_config_flags(0);
    if let Err(e) = renderer.init_window(display_width, display_height, "Slideshow") {
        tracing::error!("Failed to initialize window: {}", e);
        anyhow::bail!("Window initialization failed: {e}");
    }
    tracing::info!("Using SDL renderer: {:?}", renderer.get_renderer_name());
    renderer.show_window();
    renderer.toggle_fullscreen();
    renderer.hide_cursor();

    tracing::info!("Window shown, size: {}x{}", display_width, display_height);

    renderer.begin_drawing();
    renderer.clear_background(sdl3_rs::Color { r: 0, g: 0, b: 0, a: 255 });
    renderer.end_drawing();

    let config = SlideshowConfig {
        target_fps: cfg.target_fps,
        ..Default::default()
    };
    let renderer_for_slideshow = renderer.clone();
    let slideshow = Slideshow::new(&renderer_for_slideshow, config);

    let mut last_slideshow_generation: u32 = 0;
    let mut last_video_generation: u32 = 0;

    let (slideshow_command_tx, slideshow_command_rx) = std::sync::mpsc::sync_channel::<Command>(100);
    let (video_command_tx, video_command_rx) = std::sync::mpsc::sync_channel::<VideoCommand>(100);
    let (shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
    let shutdown_tx_for_shutdown = shutdown_tx.clone();

    let renderer_for_video = renderer.clone();
    let mut video_player = VideoPlayer::new(&renderer_for_video, cfg.media_directory.clone());

    let mut executor = MainThreadExecutor(Executor::new(slideshow, cfg.media_directory.clone()));

    let sync_service = Arc::new(
        SyncService::new(&cfg.sync_server_url, &cfg.media_directory, 300)
            .context("failed to initialize sync service")?,
    );

    let slideshow_state = Arc::new(parking_lot::Mutex::new(SlideshowState::default()));
    let video_state = Arc::new(parking_lot::Mutex::new(VideoState::default()));
    let slideshow_state_for_zenoh = slideshow_state.clone();
    let video_state_for_zenoh = video_state.clone();

    let cfg_for_zenoh = cfg.clone();

    let zenoh_tick_tx = Arc::new(std::sync::Mutex::new(None::<tokio::sync::mpsc::Sender<()>>));

    let zenoh_tick_tx_clone = zenoh_tick_tx.clone();
    let zenoh_manager: std::thread::JoinHandle<anyhow::Result<()>> = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let (manager, tick_tx) = ZenohManager::new(
                cfg_for_zenoh,
                slideshow_command_tx,
                video_command_tx,
                sync_service,
                slideshow_state_for_zenoh,
                video_state_for_zenoh,
            ).await.context("failed to initialize zenoh manager")?;
            if let Ok(mut tx) = zenoh_tick_tx_clone.lock() {
                *tx = Some(tick_tx);
            }
            manager.run(shutdown_rx).await
        })
    });

    let mut autostart_cmd = handle_autostart(&cfg);

    tracing::info!("Starting main loop");

    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        info!("Shutdown signal received (Ctrl+C)");
        r.store(false, std::sync::atomic::Ordering::Relaxed);
        if let Err(e) = shutdown_tx.try_send(()) {
            warn!("Failed to send shutdown signal to subscription loop: {}", e);
        }
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_secs(3));
            info!("Force exiting after shutdown timeout");
            std::process::exit(0);
        });
    })?;

    #[allow(clippy::cast_sign_loss)]
    let frame_duration = Duration::from_millis(1000_u64 / cfg.target_fps as u64);
    let mut frame_start = Instant::now();

    while running.load(std::sync::atomic::Ordering::Relaxed) {
        let now = Instant::now();
        let delta_time = frame_start.elapsed().as_secs_f64();
        frame_start = now;

        if let Some(cmd) = autostart_cmd.take() {
            info!("Processing autostart command");
            if let Err(e) = executor.0.execute_command(&cmd) {
                tracing::error!("Failed to execute autostart: {}", e);
            } else if let Err(e) = executor.0.tick_slideshow(Some(delta_time)) {
                tracing::error!("Autostart tick error: {}", e);
            }
        }

        process_slideshow_commands(&mut executor, &slideshow_command_rx);
        process_video_commands(&mut video_player, &video_command_rx);

        if let Err(e) = executor.0.tick_slideshow(Some(delta_time)) {
            tracing::error!("Slideshow tick error: {}", e);
        }

        if let Err(e) = video_player.tick(delta_time) {
            tracing::error!("Video tick error: {}", e);
        }

        let current_slideshow_rev = executor.0.get_state_revision();
        if current_slideshow_rev != last_slideshow_generation {
            last_slideshow_generation = current_slideshow_rev;
            if let Some(state) = executor.0.poll_state() {
                tracing::info!("Slideshow state: status={}, image={:?}", state.status.as_str(), state.current_image);
                let mut zstate = slideshow_state.lock();
                zstate.status = state.status.as_str().to_string();
                if state.status.as_str() == "stopped" {
                    zstate.name = None;
                    zstate.image = None;
                    zstate.interval = None;
                    zstate.scaling_mode = None;
                    zstate.total_images = None;
                    zstate.current_index = None;
                } else {
                    zstate.name = Some(state.name.clone());
                    zstate.image = Some(state.current_image.clone());
                    #[allow(clippy::cast_possible_truncation)]
                    { zstate.interval = Some(state.interval_seconds.round() as i64); }
                    zstate.scaling_mode = Some(state.scaling_mode.as_str().to_string());
                    zstate.total_images = Some(i64::try_from(state.total_images).unwrap_or(i64::MAX));
                    zstate.current_index = Some(i64::try_from(state.current_index).unwrap_or(i64::MAX));
                }
            }
            if let Ok(tx_guard) = zenoh_tick_tx.lock()
                && let Some(ref tx) = *tx_guard {
                    let _ = tx.try_send(());
                }
        }

        renderer.begin_drawing();
        renderer.clear_background(sdl3_rs::Color { r: 0, g: 0, b: 0, a: 255 });
        tracing::debug!(target: "render", "Frame rendered, video_active={}", video_player.is_active());
        let screen_w = renderer.get_screen_width();
        let screen_h = renderer.get_screen_height();
        tracing::trace!(target: "render", "Screen size: {}x{}", screen_w, screen_h);
        if video_player.is_active() {
            tracing::trace!(target: "render", "Rendering VIDEO");
            if let Err(e) = video_player.render() {
                tracing::error!("Video render error: {}", e);
            }
        } else {
            tracing::trace!(target: "render", "Rendering SLIDESHOW");
            if let Err(e) = executor.0.render_slideshow() {
                tracing::error!("Slideshow render error: {}", e);
            }
        }
        renderer.end_drawing();

        let current_video_rev = video_player.get_state_revision();
        if current_video_rev != last_video_generation {
            last_video_generation = current_video_rev;
            if let Some(state) = video_player.poll_state() {
                tracing::trace!("Video state changed: timestamp={}, status={}", state.timestamp, state.status.as_str());
                let mut zstate = video_state.lock();
                zstate.status = state.status.as_str().to_string();

                if state.filename.is_empty() {
                    zstate.filename = None;
                } else {
                    zstate.filename = Some(state.filename.clone());
                }
                zstate.scaling_mode = Some(state.scaling_mode.as_str().to_string());
            }
            if let Ok(tx_guard) = zenoh_tick_tx.lock()
                && let Some(ref tx) = *tx_guard {
                    let _ = tx.try_send(());
                }
        } else if !video_player.is_active() {
            let mut zstate = video_state.lock();
            if zstate.status != "idle" {
                zstate.status = "idle".to_string();
                zstate.filename = None;
                zstate.scaling_mode = None;
            }
        }

        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::yield_now();
        }
    }

    info!("Shutting down...");
    if let Err(e) = shutdown_tx_for_shutdown.try_send(()) {
        warn!("Failed to send shutdown signal: {}", e);
    }
    if let Err(e) = zenoh_manager.join() {
        tracing::error!("Zenoh manager thread panicked: {:?}", e);
    }
    executor.0.shutdown()?;
    video_player.shutdown();
    info!("Client stopped");
    Ok(())
}

fn handle_autostart(cfg: &config::Config) -> Option<Command> {
    let autostart_cfg = match load_autostart_config(&cfg.autostart_file) {
        Ok(cfg) => cfg,
        Err(e) => {
            debug!("Failed to load autostart config: {}", e);
            return None;
        }
    };

    if !autostart_cfg.enabled {
        debug!("Autostart disabled, waiting for commands");
        return None;
    }

    debug!("Autostart enabled for slideshow '{}'", autostart_cfg.name);

    let scaling_mode = match autostart_cfg.scaling_mode.to_lowercase().as_str() {
        "none" => ScalingMode::None,
        "fit" => ScalingMode::FitToScreen,
        "fill" => ScalingMode::FillToScreen,
        "stretch" => ScalingMode::StretchToFit,
        _ => {
            debug!(
                "Unknown scaling mode '{}', using 'fit'",
                autostart_cfg.scaling_mode
            );
            ScalingMode::FitToScreen
        }
    };

    if autostart_cfg.media_files.is_empty() {
        info!("No media files specified in autostart config");
        return None;
    }

    let start_data = StartSlideshowData {
        name: autostart_cfg.name.clone(),
        interval_seconds: autostart_cfg.interval_seconds,
        shuffle_enabled: autostart_cfg.options.shuffle_enabled,
        loop_enabled: autostart_cfg.options.loop_enabled,
        scaling_mode,
        media_files: autostart_cfg.media_files,
        media_directory: cfg.media_directory.clone(),
    };

    Some(Command::StartSlideshow { data: start_data })
}

fn process_slideshow_commands<R: Renderer>(
    executor: &mut MainThreadExecutor<'_, R>,
    command_rx: &std::sync::mpsc::Receiver<Command>,
) {
    while let Ok(command) = command_rx.try_recv() {
        match executor.0.execute_command(&command) {
            Ok(()) => {}
            Err(e) => {
                tracing::error!("Slideshow command error: {}", e);
            }
        }
    }
}

fn process_video_commands<R: Renderer>(
    video_player: &mut VideoPlayer<'_, R>,
    command_rx: &std::sync::mpsc::Receiver<VideoCommand>,
) {
    while let Ok(command) = command_rx.try_recv() {
        match video_player.execute_command(&command) {
            Ok(()) => {}
            Err(e) => {
                tracing::error!("Video command error: {}", e);
            }
        }
    }
}
