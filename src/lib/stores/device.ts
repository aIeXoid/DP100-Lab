import { get, writable } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { t } from "$lib/i18n";

export interface Telemetry {
  input_voltage: number;
  output_voltage: number;
  output_current: number;
  max_voltage: number;
  temperature1: number;
  temperature2: number;
  rail_5v: number;
  output_mode: string;
  work_state: string;
  power: number;
}

export interface OutputSettings {
  enabled: boolean;
  voltage: number;
  current: number;
  ovp: number;
  ocp: number;
}

// Aliases for compatibility with new backend types

export interface SystemSettings {
  otp: number;
  opp: number;
  backlight: number;
  volume: number;
  reverse_protection: boolean;
  auto_output: boolean;
}

export interface DeviceInfo {
  name: string;
  hardware_version: number;
  firmware_version: number;
  serial: string;
  year: number;
  month: number;
  day: number;
}

export interface PresetData {
  index: number;
  voltage: number;
  current: number;
  ovp: number;
  ocp: number;
}

export interface PresetsPayload {
  presets: PresetData[];
  active_index: number;
}

export interface HistoryPoint {
  time: number;
  voltage: number;
  current: number;
  power: number;
}

export const connected = writable(false);
export const connecting = writable(false);
export const telemetry = writable<Telemetry | null>(null);
export const settings = writable<OutputSettings | null>(null);
export const systemSettings = writable<SystemSettings | null>(null);
export const deviceInfo = writable<DeviceInfo | null>(null);
export const error = writable<string | null>(null);
export const presets = writable<PresetData[]>([]);
export const activePresetIndex = writable<number>(0);

const MAX_HISTORY = 1200;
export const history = writable<HistoryPoint[]>([]);

let listenersSetup = false;

export function setupListeners() {
  if (listenersSetup) return;
  listenersSetup = true;

  listen<Telemetry>("telemetry", (event) => {
    const t = event.payload;
    telemetry.set(t);

    history.update((h) => {
      h.push({
        time: Date.now() / 1000,
        voltage: t.output_voltage,
        current: t.output_current,
        power: t.power,
      });
      if (h.length > MAX_HISTORY) {
        h = h.slice(h.length - MAX_HISTORY);
      }
      return h;
    });

  });

  listen<OutputSettings>("settings", (event) => {
    settings.set(event.payload);
  });

  listen<SystemSettings>("system-settings", (event) => {
    systemSettings.set(event.payload);
  });

  listen<DeviceInfo>("device-info", (event) => {
    deviceInfo.set(event.payload);
  });

  listen<PresetsPayload>("presets", (event) => {
    presets.set(event.payload.presets);
    activePresetIndex.set(event.payload.active_index);
  });

  listen<ScanStatus>("scan-status", (event) => {
    scanStatus.set(event.payload);
  });

  listen<LoggingStatus>("logging-status", (event) => {
    loggingStatus.set(event.payload);
  });

  listen<boolean>("connection-status", (event) => {
    connected.set(event.payload);
    connecting.set(false);
    if (!event.payload) {
      telemetry.set(null);
    }
  });
}

export async function connectDevice() {
  error.set(null);
  connecting.set(true);
  try {
    await invoke("connect");
  } catch (e) {
    connecting.set(false);
    error.set(String(e));
    throw e;
  }
}

export async function disconnectDevice() {
  try {
    await invoke("disconnect");
    connected.set(false);
    telemetry.set(null);
    settings.set(null);
    history.set([]);
  } catch (e) {
    error.set(String(e));
  }
}

export async function setOutput(
  enabled: boolean,
  voltage: number,
  current: number,
) {
  try {
    await invoke("set_output", { enabled, voltage, current });
  } catch (e) {
    error.set(String(e));
  }
}

export async function getAllPresets() {
  try {
    await invoke("get_all_presets");
  } catch (e) {
    error.set(String(e));
  }
}

export async function savePreset(
  index: number,
  voltage: number,
  current: number,
  ovp: number,
  ocp: number,
) {
  try {
    await invoke("save_preset", { index, voltage, current, ovp, ocp });
  } catch (e) {
    error.set(String(e));
  }
}

export async function activatePreset(index: number) {
  try {
    await invoke("activate_preset", { index });
  } catch (e) {
    error.set(String(e));
  }
}

// CSV Logging
export interface LoggingStatus {
  active: boolean;
  samples: number;
  duration_secs: number;
  path: string;
}
export const loggingStatus = writable<LoggingStatus>({
  active: false,
  samples: 0,
  duration_secs: 0,
  path: "",
});

export async function startLogging() {
  const { save } = await import("@tauri-apps/plugin-dialog");
  const translate = get(t);
  const path = await save({
    title: translate("saveCsvLog"),
    defaultPath: `dp100_log_${new Date().toISOString().slice(0, 19).replace(/:/g, "-")}.csv`,
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  if (!path) return;
  try {
    await invoke("start_logging", { path });
  } catch (e) {
    error.set(String(e));
  }
}

export async function startScan(scanMode: number, outVal: number, start: number, end: number, step: number, delayMs: number) {
  try {
    await invoke("start_scan", { scanMode, outVal, start, end, step, delayMs });
  } catch (e) { error.set(String(e)); }
}

export async function stopScan() {
  try { await invoke("stop_scan"); } catch (e) { error.set(String(e)); }
}

export interface ScanStatus {
  active: boolean;
  current_step: number;
  total_steps: number;
  current_value: number;
}
export const scanStatus = writable<ScanStatus>({ active: false, current_step: 0, total_steps: 0, current_value: 0 });

export async function setPollRate(ms: number) {
  try {
    await invoke("set_poll_rate", { ms });
  } catch (e) {
    error.set(String(e));
  }
}

export async function stopLogging() {
  try {
    await invoke("stop_logging");
  } catch (e) {
    error.set(String(e));
  }
}

export async function setProtection(ovp: number, ocp: number) {
  try {
    await invoke("set_protection", { ovp, ocp });
  } catch (e) {
    error.set(String(e));
  }
}

export async function setSystemSettings(
  otp: number,
  opp: number,
  backlight: number,
  volume: number,
  reverseProtection: boolean,
  autoOutput: boolean,
) {
  try {
    await invoke("set_system_settings", {
      otp,
      opp,
      backlight,
      volume,
      reverseProtection,
      autoOutput,
    });
  } catch (e) {
    error.set(String(e));
  }
}
