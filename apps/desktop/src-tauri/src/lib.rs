mod runtime;

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use keytone_core::{Effects, KeyCode, KeyEvent, KeyState};
use runtime::{AppRuntime, AppSnapshot};
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
fn get_state(runtime: State<'_, AppRuntime>) -> AppSnapshot {
    runtime.snapshot()
}

#[tauri::command]
fn set_engine(runtime: State<'_, AppRuntime>, enabled: bool) -> Result<AppSnapshot, String> {
    runtime
        .set_engine(enabled)
        .map_err(|error| error.to_string())?;
    Ok(runtime.snapshot())
}

#[tauri::command]
fn update_effects(runtime: State<'_, AppRuntime>, effects: Effects) -> Result<AppSnapshot, String> {
    runtime
        .update_effects(effects)
        .map_err(|error| error.to_string())?;
    Ok(runtime.snapshot())
}

#[tauri::command]
fn update_preferences(
    runtime: State<'_, AppRuntime>,
    release_sounds: bool,
    play_repeats: bool,
    launch_at_startup: bool,
    keep_running_on_close: bool,
) -> Result<AppSnapshot, String> {
    runtime
        .update_preferences(
            release_sounds,
            play_repeats,
            launch_at_startup,
            keep_running_on_close,
        )
        .map_err(|error| error.to_string())?;
    Ok(runtime.snapshot())
}

#[tauri::command]
fn activate_pack(runtime: State<'_, AppRuntime>, id: String) -> Result<AppSnapshot, String> {
    runtime
        .activate_pack(&id)
        .map_err(|error| error.to_string())?;
    Ok(runtime.snapshot())
}

#[tauri::command]
fn preview_pack(runtime: State<'_, AppRuntime>, id: String) -> Result<(), String> {
    runtime.preview_pack(&id).map_err(|error| error.to_string())
}

#[tauri::command]
fn import_pack(runtime: State<'_, AppRuntime>, path: String) -> Result<AppSnapshot, String> {
    runtime
        .import_pack(&path)
        .map_err(|error| error.to_string())?;
    Ok(runtime.snapshot())
}

#[tauri::command]
fn save_preset(runtime: State<'_, AppRuntime>, name: String) -> Result<AppSnapshot, String> {
    runtime
        .save_preset(name)
        .map_err(|error| error.to_string())?;
    Ok(runtime.snapshot())
}

#[tauri::command]
fn load_preset(runtime: State<'_, AppRuntime>, name: String) -> Result<AppSnapshot, String> {
    runtime
        .load_preset(&name)
        .map_err(|error| error.to_string())?;
    Ok(runtime.snapshot())
}

#[tauri::command]
fn duplicate_preset(runtime: State<'_, AppRuntime>, name: String) -> Result<AppSnapshot, String> {
    runtime
        .duplicate_preset(&name)
        .map_err(|error| error.to_string())?;
    Ok(runtime.snapshot())
}

#[tauri::command]
fn reset_effects(runtime: State<'_, AppRuntime>) -> Result<AppSnapshot, String> {
    runtime.reset_effects().map_err(|error| error.to_string())?;
    Ok(runtime.snapshot())
}

#[tauri::command]
fn select_output_device(
    runtime: State<'_, AppRuntime>,
    name: Option<String>,
) -> Result<AppSnapshot, String> {
    runtime
        .select_output_device(name)
        .map_err(|error| error.to_string())?;
    Ok(runtime.snapshot())
}

#[tauri::command]
fn test_sound(runtime: State<'_, AppRuntime>) {
    runtime.audio().trigger(KeyEvent {
        key: KeyCode::Space,
        state: KeyState::Pressed,
        timestamp: Instant::now(),
    });
}

#[tauri::command]
fn open_keyboard_settings(runtime: State<'_, AppRuntime>) -> Result<AppSnapshot, String> {
    runtime
        .open_keyboard_settings()
        .map_err(|error| error.to_string())?;
    Ok(runtime.snapshot())
}

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let engine = CheckMenuItem::with_id(app, "engine", "Engine Enabled", true, true, None::<&str>)?;
    let creamy = MenuItem::with_id(app, "pack-creamy", "Creamy", true, None::<&str>)?;
    let clicky = MenuItem::with_id(app, "pack-clicky", "Clicky", true, None::<&str>)?;
    let retro = MenuItem::with_id(app, "pack-retro", "Retro Terminal", true, None::<&str>)?;
    let deep_thock = MenuItem::with_id(app, "pack-deep-thock", "Deep Thock", true, None::<&str>)?;
    let tactile = MenuItem::with_id(app, "pack-tactile", "Tactile Workshop", true, None::<&str>)?;
    let alloy = MenuItem::with_id(app, "pack-alloy", "Alloy Linear", true, None::<&str>)?;
    let spring = MenuItem::with_id(app, "pack-spring", "Spring Clack", true, None::<&str>)?;
    let pack_menu = Submenu::with_items(
        app,
        "Sound Pack",
        true,
        &[
            &creamy,
            &clicky,
            &retro,
            &deep_thock,
            &tactile,
            &alloy,
            &spring,
        ],
    )?;
    let volume_100 = MenuItem::with_id(app, "volume-100", "100%", true, None::<&str>)?;
    let volume_75 = MenuItem::with_id(app, "volume-75", "75%", true, None::<&str>)?;
    let volume_50 = MenuItem::with_id(app, "volume-50", "50%", true, None::<&str>)?;
    let volume_25 = MenuItem::with_id(app, "volume-25", "25%", true, None::<&str>)?;
    let volume_menu = Submenu::with_items(
        app,
        "Volume",
        true,
        &[&volume_100, &volume_75, &volume_50, &volume_25],
    )?;
    let open = MenuItem::with_id(app, "open", "Open Keytone", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[&engine, &separator, &pack_menu, &volume_menu, &open, &quit],
    )?;
    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("Keytone")
        .menu(&menu);
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "quit" => {
                app.state::<QuitState>().0.store(true, Ordering::Release);
                app.exit(0);
            }
            "open" => show_window(app),
            "engine" => {
                let runtime = app.state::<AppRuntime>();
                let _ = runtime.set_engine(!runtime.audio().is_enabled());
            }
            id if id.starts_with("pack-") => {
                let runtime = app.state::<AppRuntime>();
                let _ = runtime.activate_pack(id.trim_start_matches("pack-"));
            }
            id if id.starts_with("volume-") => {
                if let Ok(percent) = id.trim_start_matches("volume-").parse::<f32>() {
                    let runtime = app.state::<AppRuntime>();
                    let mut effects = runtime.snapshot().settings.effects;
                    effects.master_volume = percent / 100.0;
                    let _ = runtime.update_effects(effects);
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(event, tauri::tray::TrayIconEvent::Click { .. }) {
                show_window(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

struct QuitState(AtomicBool);

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "keytone=info".into()),
        )
        .init();
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let runtime = AppRuntime::initialize(app.handle())
                .map_err(|error| Box::<dyn std::error::Error>::from(error.to_string()))?;
            app.manage(runtime);
            app.manage(QuitState(AtomicBool::new(false)));
            setup_tray(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let quitting = window.state::<QuitState>().0.load(Ordering::Acquire);
                let keep_running = window.try_state::<AppRuntime>().map_or(true, |runtime| {
                    runtime.snapshot().settings.keep_running_on_close
                });
                if !quitting && keep_running {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            set_engine,
            update_effects,
            update_preferences,
            activate_pack,
            preview_pack,
            import_pack,
            save_preset,
            load_preset,
            duplicate_preset,
            reset_effects,
            select_output_device,
            test_sound,
            open_keyboard_settings,
        ])
        .run(tauri::generate_context!());
    if let Err(error) = result {
        tracing::error!(%error, "Keytone application runtime failed");
    }
}
