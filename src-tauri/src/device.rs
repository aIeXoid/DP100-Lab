use dp100_proto::{DP100, Preset, SystemSettings as DevSysSettings, Telemetry as DevTelemetry};
use serde::Serialize;
use std::io::Write;
use std::sync::{mpsc, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use tauri::Emitter;

static DEBUG_LOG: AtomicBool = AtomicBool::new(false);

pub fn set_debug_logging(enabled: bool) {
    DEBUG_LOG.store(enabled, Ordering::Relaxed);
}

fn debug_log_path() -> std::path::PathBuf {
    let dir = dirs::home_dir()
        .unwrap_or_default()
        .join("Library/Logs/DP100Lab");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("debug.log")
}

pub fn get_debug_log_path() -> String {
    debug_log_path().to_string_lossy().to_string()
}

macro_rules! dlog {
    ($($arg:tt)*) => {
        if DEBUG_LOG.load(Ordering::Relaxed) {
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true).append(true)
                .open(debug_log_path())
            {
                let ts = chrono::Local::now().format("%H:%M:%S%.3f");
                let _ = writeln!(f, "[{}] {}", ts, format!($($arg)*));
            }
        }
    };
}

#[derive(Debug, Clone, Serialize)]
pub struct Telemetry {
    pub input_voltage: f64,
    pub output_voltage: f64,
    pub output_current: f64,
    pub max_voltage: f64,
    pub temperature1: f64,
    pub temperature2: f64,
    pub rail_5v: f64,
    pub output_mode: String,
    pub work_state: String,
    pub power: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Settings {
    pub enabled: bool,
    pub voltage: f64,
    pub current: f64,
    pub ovp: f64,
    pub ocp: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SysSettings {
    pub otp: f64,
    pub opp: f64,
    pub backlight: u8,
    pub volume: u8,
    pub reverse_protection: bool,
    pub auto_output: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfoData {
    pub name: String,
    pub hardware_version: f64,
    pub firmware_version: f64,
    pub serial: String,
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct PresetData {
    pub index: u8,
    pub voltage: f64,
    pub current: f64,
    pub ovp: f64,
    pub ocp: f64,
}

pub enum Command {
    SetOutput { enabled: bool, voltage: f64, current: f64 },
    SetProtection { ovp: f64, ocp: f64 },
    ReadSettings,
    GetAllPresets,
    SavePreset { index: u8, voltage: f64, current: f64, ovp: f64, ocp: f64 },
    ActivatePreset { index: u8 },
    SetSystemSettings { otp: f64, opp: f64, backlight: u8, volume: u8, reverse_protection: bool, auto_output: bool },
    StartLogging { path: String },
    StopLogging,
    SetPollRate { ms: u64 },
    StartScan { scan_mode: u8, out_val: f64, start: f64, end: f64, step: f64, delay_ms: u64 },
    StopScan,
    Stop,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanStatus {
    pub active: bool,
    pub current_step: u32,
    pub total_steps: u32,
    pub current_value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoggingStatus {
    pub active: bool,
    pub samples: u64,
    pub duration_secs: f64,
    pub path: String,
}

pub struct DeviceHandle {
    cmd_tx: mpsc::Sender<Command>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl DeviceHandle {
    pub fn connect(app_handle: tauri::AppHandle) -> Result<Self, String> {
        let count = DP100::device_count()?;
        if count == 0 {
            return Err("DP100 not found. Check USB connection.".into());
        }
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let thread = std::thread::spawn(move || poll_loop(cmd_rx, app_handle));
        Ok(Self { cmd_tx, thread: Some(thread) })
    }

    pub fn send(&self, cmd: Command) -> Result<(), String> {
        self.cmd_tx.send(cmd).map_err(|e| e.to_string())
    }

    pub fn is_finished(&self) -> bool {
        self.thread.as_ref().map(|t| t.is_finished()).unwrap_or(true)
    }

    pub fn shutdown(&mut self) {
        let _ = self.cmd_tx.send(Command::Stop);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for DeviceHandle {
    fn drop(&mut self) { self.shutdown(); }
}

fn telemetry_from(t: &DevTelemetry) -> Telemetry {
    let vout = t.vout as f64 / 1000.0;
    let iout = t.iout as f64 / 1000.0;
    Telemetry {
        input_voltage: t.vin as f64 / 1000.0,
        output_voltage: vout,
        output_current: iout,
        max_voltage: t.vo_max as f64 / 1000.0,
        temperature1: t.temp1 as f64 / 10.0,
        temperature2: t.temp2 as f64 / 10.0,
        rail_5v: t.dc_5v as f64 / 1000.0,
        output_mode: match t.out_mode {
            0 => "OFF", 1 => "CV", 2 => "CC", _ => "Unknown",
        }.into(),
        work_state: match t.work_st {
            0 => "Normal", 1 => "OVP", 2 => "OCP", 3 => "OPP",
            4 => "OTP", 5 => "REP", 6 => "UVP", _ => "Unknown",
        }.into(),
        power: vout * iout,
    }
}

fn settings_from(p: &Preset) -> Settings {
    Settings {
        enabled: p.state != 0,
        voltage: p.voltage as f64 / 1000.0,
        current: p.current as f64 / 1000.0,
        ovp: p.ovp as f64 / 1000.0,
        ocp: p.ocp as f64 / 1000.0,
    }
}

fn preset_data_from(p: &Preset) -> PresetData {
    PresetData {
        index: p.index,
        voltage: p.voltage as f64 / 1000.0,
        current: p.current as f64 / 1000.0,
        ovp: p.ovp as f64 / 1000.0,
        ocp: p.ocp as f64 / 1000.0,
    }
}

fn sys_from(s: &DevSysSettings) -> SysSettings {
    SysSettings {
        otp: s.otp as f64,
        opp: s.opp as f64 / 10.0,
        backlight: s.backlight,
        volume: s.volume,
        reverse_protection: s.reverse_protection,
        auto_output: s.auto_output,
    }
}

fn mv(v: f64) -> u16 { (v * 1000.0).round().max(0.0) as u16 }

fn poll_loop(cmd_rx: mpsc::Receiver<Command>, app: tauri::AppHandle) {
    let mut dp = match DP100::open(0) {
        Ok(d) => d,
        Err(_) => {
            let _ = app.emit("connection-status", false);
            return;
        }
    };

    dp.set_log_callback(Box::new(|msg| {
        dlog!("PROTO {}", msg);
    }));

    // Wait for device to be ready (important after USB reconnect)
    let mut ready = false;
    for attempt in 0..10 {
        if dp.basic_info().is_some() {
            ready = true;
            break;
        }
        dlog!("Device not ready, attempt {}/10", attempt + 1);
        std::thread::sleep(Duration::from_millis(300));
    }
    if !ready {
        dlog!("Device not responding after 10 attempts");
        let _ = app.emit("connection-status", false);
        return;
    }

    // Init: device info
    if let Some(info) = dp.device_info() {
        let _ = app.emit("device-info", DeviceInfoData {
            name: info.name,
            hardware_version: info.hardware_version,
            firmware_version: info.firmware_version,
            serial: info.serial,
            year: info.year,
            month: info.month,
            day: info.day,
        });
    }

    // Init: system settings
    if let Some(sys) = dp.system_settings() {
        let _ = app.emit("system-settings", sys_from(&sys));
    }

    // Init: current preset → track OVP/OCP
    let mut cur_ovp: u16 = 30500;
    let mut cur_ocp: u16 = 5050;
    if let Some(preset) = dp.current_preset() {
        cur_ovp = preset.ovp;
        cur_ocp = preset.ocp;
        let _ = app.emit("settings", settings_from(&preset));
    }

    let _ = app.emit("connection-status", true);

    let mut poll_interval = Duration::from_millis(50); // ~20Hz default

    // Scan state
    let mut scan_params: Option<(u8, f64, f64, f64, f64, u64)> = None; // mode, out_val, start, end, step, delay
    let mut scan_value: f64 = 0.0;
    let mut scan_step_num: u32 = 0;
    let mut scan_total: u32 = 0;
    let mut scan_last_step = Instant::now();

    // CSV logging state
    let mut csv_writer: Option<std::io::BufWriter<std::fs::File>> = None;
    let mut csv_samples: u64 = 0;
    let mut csv_start: Option<Instant> = None;
    let mut csv_path = String::new();

    let emit_log_status = |app: &tauri::AppHandle, active: bool, samples: u64, start: Option<Instant>, path: &str| {
        let _ = app.emit("logging-status", LoggingStatus {
            active,
            samples,
            duration_secs: start.map(|s| s.elapsed().as_secs_f64()).unwrap_or(0.0),
            path: path.to_string(),
        });
    };

    loop {
        // Process commands
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                Command::Stop => {
                    csv_writer.take(); // close file
                    let _ = app.emit("connection-status", false);
                    return;
                }
                Command::SetOutput { enabled, voltage, current } => {
                    dlog!("CMD SetOutput en={} v={} i={}", enabled, voltage, current);
                    dp.set_output(enabled, mv(voltage), mv(current), cur_ovp, cur_ocp);
                    std::thread::sleep(Duration::from_millis(200));
                    dp.flush();
                    if let Some(p) = dp.current_preset() {
                        cur_ovp = p.ovp;
                        cur_ocp = p.ocp;
                        let _ = app.emit("settings", settings_from(&p));
                    }
                    dp.flush();
                }
                Command::SetProtection { ovp, ocp } => {
                    dlog!("CMD SetProtection ovp={} ocp={}", ovp, ocp);
                    cur_ovp = mv(ovp);
                    cur_ocp = mv(ocp);
                    dp.set_protection(cur_ovp, cur_ocp);
                    dp.flush();
                    if let Some(p) = dp.current_preset() {
                        cur_ovp = p.ovp;
                        cur_ocp = p.ocp;
                        let _ = app.emit("settings", settings_from(&p));
                    }
                }
                Command::ReadSettings => {
                    dp.flush();
                    if let Some(p) = dp.current_preset() {
                        cur_ovp = p.ovp;
                        cur_ocp = p.ocp;
                        let _ = app.emit("settings", settings_from(&p));
                    }
                }
                Command::GetAllPresets => {
                    dp.flush();
                    let mut presets = Vec::new();
                    for i in 0..10u8 {
                        if let Some(p) = dp.read_preset(i) {
                            presets.push(preset_data_from(&p));
                        }
                    }
                    if let Some(active) = dp.current_preset() {
                        let _ = app.emit("presets", serde_json::json!({
                            "presets": presets,
                            "active_index": active.index,
                        }));
                    }
                }
                Command::SavePreset { index, voltage, current, ovp, ocp } => {
                    dlog!("CMD SavePreset P{} v={} i={} ovp={} ocp={}", index, voltage, current, ovp, ocp);
                    dp.save_preset(index, mv(voltage), mv(current), mv(ovp), mv(ocp));
                    std::thread::sleep(Duration::from_millis(200));
                }
                Command::ActivatePreset { index } => {
                    dlog!("CMD ActivatePreset P{}", index);
                    dp.flush();
                    if let Some(preset) = dp.read_preset(index) {
                        dp.activate_preset(
                            index, 0,
                            preset.voltage, preset.current,
                            preset.ovp, preset.ocp,
                        );
                        cur_ovp = preset.ovp;
                        cur_ocp = preset.ocp;
                    }
                    std::thread::sleep(Duration::from_millis(200));
                    dp.flush();
                    if let Some(p) = dp.current_preset() {
                        cur_ovp = p.ovp;
                        cur_ocp = p.ocp;
                        let _ = app.emit("settings", settings_from(&p));
                    }
                }
                Command::StartLogging { path } => {
                    dlog!("CMD StartLogging path={}", path);
                    if let Ok(file) = std::fs::File::create(&path) {
                        let mut writer = std::io::BufWriter::new(file);
                        let _ = writeln!(writer, "Timestamp,Voltage(V),Current(A),Power(W),Vin(V),Temp1(°C),Temp2(°C),Mode,State");
                        csv_writer = Some(writer);
                        csv_samples = 0;
                        csv_start = Some(Instant::now());
                        csv_path = path;
                        emit_log_status(&app, true, 0, csv_start, &csv_path);
                    }
                }
                Command::StartScan { scan_mode, out_val, start, end, step, delay_ms } => {
                    dlog!("CMD StartScan mode={} out={} {}->{} step={} delay={}ms", scan_mode, out_val, start, end, step, delay_ms);
                    // Validate: step must move toward end
                    let valid = (step > 0.0 && start <= end) || (step < 0.0 && start >= end);
                    if valid && step.abs() > 0.001 {
                        scan_params = Some((scan_mode, out_val, start, end, step, delay_ms));
                        scan_value = start;
                        scan_step_num = 0;
                        let total = ((end - start).abs() / step.abs()).ceil() as u32 + 1;
                        scan_total = total;
                        scan_last_step = Instant::now() - Duration::from_millis(delay_ms);
                        let _ = app.emit("scan-status", ScanStatus { active: true, current_step: 0, total_steps: total, current_value: start });
                    }
                }
                Command::StopScan => {
                    dlog!("CMD StopScan");
                    scan_params = None;
                    let _ = app.emit("scan-status", ScanStatus { active: false, current_step: 0, total_steps: 0, current_value: 0.0 });
                }
                Command::SetPollRate { ms } => {
                    dlog!("CMD SetPollRate {}ms", ms);
                    poll_interval = Duration::from_millis(ms);
                }
                Command::StopLogging => {
                    dlog!("CMD StopLogging samples={}", csv_samples);
                    if let Some(mut w) = csv_writer.take() {
                        let _ = w.flush();
                    }
                    emit_log_status(&app, false, csv_samples, None, &csv_path);
                    csv_samples = 0;
                    csv_start = None;
                }
                Command::SetSystemSettings { otp, opp, backlight, volume, reverse_protection, auto_output } => {
                    dlog!("CMD SetSysSettings otp={} opp={} blk={} vol={} rep={} auto={}", otp, opp, backlight, volume, reverse_protection, auto_output);
                    dp.set_system_settings(&DevSysSettings {
                        otp: otp as u16,
                        opp: (opp * 10.0) as u16,
                        backlight,
                        volume,
                        reverse_protection,
                        auto_output,
                    });
                    std::thread::sleep(Duration::from_millis(100));
                    if let Some(sys) = dp.system_settings() {
                        let _ = app.emit("system-settings", sys_from(&sys));
                    }
                }
            }
        }

        // Execute scan step if active
        if let Some((mode, out_val, _start, end, step, delay_ms)) = scan_params {
            if scan_last_step.elapsed() >= Duration::from_millis(delay_ms) {
                if (step > 0.0 && scan_value <= end) || (step < 0.0 && scan_value >= end) {
                    dlog!("SCAN step {} value={}", scan_step_num, scan_value);
                    if mode == 1 {
                        // Voltage scan: fixed current, sweep voltage
                        dp.set_output(true, mv(scan_value), mv(out_val), cur_ovp, cur_ocp);
                    } else {
                        // Current scan: fixed voltage, sweep current
                        dp.set_output(true, mv(out_val), mv(scan_value), cur_ovp, cur_ocp);
                    }
                    std::thread::sleep(Duration::from_millis(200));
                    dp.flush();
                    let _ = app.emit("scan-status", ScanStatus {
                        active: true, current_step: scan_step_num, total_steps: scan_total, current_value: scan_value,
                    });
                    scan_value += step;
                    scan_step_num += 1;
                    scan_last_step = Instant::now();
                } else {
                    // Scan complete
                    dlog!("SCAN complete after {} steps", scan_step_num);
                    scan_params = None;
                    let _ = app.emit("scan-status", ScanStatus { active: false, current_step: scan_step_num, total_steps: scan_total, current_value: 0.0 });
                }
            }
        }

        // Poll telemetry — detect disconnection
        let poll_result = match dp.basic_info() {
            Some(info) => Some(info),
            None => {
                // Retry once before declaring disconnect
                std::thread::sleep(Duration::from_millis(100));
                dp.basic_info()
            }
        };
        if poll_result.is_none() {
            dlog!("Device disconnected (no telemetry response)");
            csv_writer.take();
            let _ = app.emit("connection-status", false);
            return;
        }
        if let Some(info) = poll_result {
            let telem = telemetry_from(&info);

            // Write to CSV if logging
            if let Some(ref mut w) = csv_writer {
                let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                let _ = writeln!(w,
                    "{},{:.3},{:.4},{:.3},{:.3},{:.1},{:.1},{},{}",
                    ts,
                    telem.output_voltage, telem.output_current, telem.power,
                    telem.input_voltage, telem.temperature1, telem.temperature2,
                    telem.output_mode, telem.work_state,
                );
                csv_samples += 1;
                if csv_samples % 20 == 0 {
                    let _ = w.flush();
                    emit_log_status(&app, true, csv_samples, csv_start, &csv_path);
                }
            }

            let _ = app.emit("telemetry", telem);
        }

        std::thread::sleep(poll_interval);
    }
}
