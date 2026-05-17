mod device;

use device::{Command, DeviceHandle};
use std::sync::Mutex;

struct AppState {
    device: Mutex<Option<DeviceHandle>>,
}

#[tauri::command]
fn connect(state: tauri::State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    let mut device = state.device.lock().unwrap_or_else(|e| e.into_inner());
    // Clean up stale handle if thread already exited (e.g. USB disconnect)
    if let Some(ref handle) = *device {
        if handle.is_finished() {
            drop(device.take());
        } else {
            return Err("Already connected".into());
        }
    }
    match DeviceHandle::connect(app) {
        Ok(handle) => {
            *device = Some(handle);
            Ok(())
        }
        Err(e) => {
            Err(e)
        }
    }
}

#[tauri::command]
fn disconnect(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut device = state.device.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(mut handle) = device.take() {
        handle.shutdown();
    }
    Ok(())
}

#[tauri::command]
fn set_output(
    state: tauri::State<'_, AppState>,
    enabled: bool,
    voltage: f64,
    current: f64,
) -> Result<(), String> {
    let device = state.device.lock().unwrap_or_else(|e| e.into_inner());
    device
        .as_ref()
        .ok_or("Not connected")?
        .send(Command::SetOutput {
            enabled,
            voltage,
            current,
        })
}

#[tauri::command]
fn set_protection(
    state: tauri::State<'_, AppState>,
    ovp: f64,
    ocp: f64,
) -> Result<(), String> {
    let device = state.device.lock().unwrap_or_else(|e| e.into_inner());
    device
        .as_ref()
        .ok_or("Not connected")?
        .send(Command::SetProtection { ovp, ocp })
}

#[tauri::command]
fn read_settings(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let device = state.device.lock().unwrap_or_else(|e| e.into_inner());
    device
        .as_ref()
        .ok_or("Not connected")?
        .send(Command::ReadSettings)
}

#[tauri::command]
fn get_all_presets(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let device = state.device.lock().unwrap_or_else(|e| e.into_inner());
    device
        .as_ref()
        .ok_or("Not connected")?
        .send(Command::GetAllPresets)
}

#[tauri::command]
fn save_preset(
    state: tauri::State<'_, AppState>,
    index: u8,
    voltage: f64,
    current: f64,
    ovp: f64,
    ocp: f64,
) -> Result<(), String> {
    let device = state.device.lock().unwrap_or_else(|e| e.into_inner());
    device
        .as_ref()
        .ok_or("Not connected")?
        .send(Command::SavePreset {
            index,
            voltage,
            current,
            ovp,
            ocp,
        })
}

#[tauri::command]
fn set_system_settings(
    state: tauri::State<'_, AppState>,
    otp: f64,
    opp: f64,
    backlight: u8,
    volume: u8,
    reverse_protection: bool,
    auto_output: bool,
) -> Result<(), String> {
    let device = state.device.lock().unwrap_or_else(|e| e.into_inner());
    device
        .as_ref()
        .ok_or("Not connected")?
        .send(Command::SetSystemSettings {
            otp, opp, backlight, volume, reverse_protection, auto_output,
        })
}

#[tauri::command]
fn start_scan(
    state: tauri::State<'_, AppState>,
    scan_mode: u8, out_val: f64, start: f64, end: f64, step: f64, delay_ms: u64,
) -> Result<(), String> {
    let device = state.device.lock().unwrap_or_else(|e| e.into_inner());
    device.as_ref().ok_or("Not connected")?.send(Command::StartScan { scan_mode, out_val, start, end, step, delay_ms })
}

#[tauri::command]
fn stop_scan(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let device = state.device.lock().unwrap_or_else(|e| e.into_inner());
    device.as_ref().ok_or("Not connected")?.send(Command::StopScan)
}

#[tauri::command]
fn set_poll_rate(state: tauri::State<'_, AppState>, ms: u64) -> Result<(), String> {
    let device = state.device.lock().unwrap_or_else(|e| e.into_inner());
    device.as_ref().ok_or("Not connected")?.send(Command::SetPollRate { ms })
}

#[tauri::command]
fn start_logging(state: tauri::State<'_, AppState>, path: String) -> Result<(), String> {
    let device = state.device.lock().unwrap_or_else(|e| e.into_inner());
    device
        .as_ref()
        .ok_or("Not connected")?
        .send(Command::StartLogging { path })
}

#[tauri::command]
fn set_debug_log(enabled: bool) -> Result<String, String> {
    device::set_debug_logging(enabled);
    Ok(device::get_debug_log_path())
}

#[tauri::command]
fn get_debug_log_path() -> String {
    device::get_debug_log_path()
}

#[tauri::command]
fn stop_logging(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let device = state.device.lock().unwrap_or_else(|e| e.into_inner());
    device
        .as_ref()
        .ok_or("Not connected")?
        .send(Command::StopLogging)
}

#[tauri::command]
fn activate_preset(state: tauri::State<'_, AppState>, index: u8) -> Result<(), String> {
    let device = state.device.lock().unwrap_or_else(|e| e.into_inner());
    device
        .as_ref()
        .ok_or("Not connected")?
        .send(Command::ActivatePreset { index })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            device: Mutex::new(None),
        })
        .setup(|_app| {
            #[cfg(target_os = "macos")]
            {
                use tauri::Manager;
                if let Some(window) = _app.get_webview_window("main") {
                    use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
                    let _ = apply_vibrancy(&window, NSVisualEffectMaterial::Sidebar, None, None);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connect,
            disconnect,
            set_output,
            set_protection,
            read_settings,
            get_all_presets,
            save_preset,
            activate_preset,
            set_system_settings,
            start_scan,
            stop_scan,
            set_poll_rate,
            start_logging,
            stop_logging,
            set_debug_log,
            get_debug_log_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
