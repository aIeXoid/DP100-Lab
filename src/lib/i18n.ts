import { derived, writable } from "svelte/store";

export type Language = "en" | "zh-CN";

const STORAGE_KEY = "dp100-lab-language";

const messages = {
  en: {
    activate: "Activate",
    advanced: "Advanced",
    about: "About",
    apply: "Apply",
    autoOutputOnBoot: "Auto Output on Boot",
    backlight: "Backlight",
    buyMeACoffee: "Buy Me a Coffee",
    cancel: "Cancel",
    clearChart: "Clear chart",
    closeSettings: "Close settings",
    connect: "Connect",
    connectDeviceInfo: "Connect device to view info",
    connectDevicePresets: "Connect device to view presets",
    connectDp100: "Connect DP100",
    connectViaUsb: "Connect your Alientek DP100 power supply via USB",
    connecting: "Connecting...",
    current: "Current",
    currentScan: "Current Scan (CC)",
    debug: "Debug",
    delay: "Delay",
    device: "Device",
    disconnect: "Disconnect",
    editPreset: "Edit P{index}",
    end: "End",
    energy: "Energy",
    fast: "Fast",
    firmware: "Firmware",
    fixedCurrent: "Fixed Current",
    fixedVoltage: "Fixed Voltage",
    hardware: "Hardware",
    input: "Input",
    language: "Language",
    license: "License",
    logPath: "Log: {path}",
    lookingForDp100: "Looking for DP100...",
    manufactured: "Manufactured",
    max: "max {value}",
    mid: "Mid",
    mode: "Mode",
    model: "Model",
    ocp: "OCP",
    ocpAmp: "OCP (A)",
    off: "OFF",
    on: "ON",
    openSourceDesc: "Open-source macOS app for Alientek DP100 digital power supply.",
    output: "Output",
    overPower: "Over-Power (OPP)",
    overTemperature: "Over-Temperature (OTP)",
    ovp: "OVP",
    ovpVolt: "OVP (V)",
    pause: "Pause",
    pollingRate: "Polling rate",
    power: "Power",
    presets: "Presets",
    protection: "Protection",
    protocolLogging: "Protocol Logging",
    rail5v: "5V Rail",
    recordToCsv: "Record to CSV",
    recording: "REC",
    recordingInfo: "{samples} samples · {duration}s",
    resume: "Resume",
    reverseProtection: "Reverse Protection",
    save: "Save",
    saveCsvLog: "Save CSV Log",
    serial: "Serial",
    set: "Set",
    setCurrent: "Set Current",
    setVoltage: "Set Voltage",
    settings: "Settings",
    slow: "Slow",
    stack: "Stack",
    start: "Start",
    startScan: "Start Scan",
    state: "State",
    status: "Status",
    step: "Step",
    stopRecording: "Stop recording",
    stopScan: "Stop Scan",
    temp: "Temp",
    tooltipPreset: "Click to edit - Double-click to activate",
    voltage: "Voltage",
    voltageCurrentScanning: "Voltage / Current Scanning",
    voltageScan: "Voltage Scan (CV)",
    volume: "Volume",
    now: "now",
  },
  "zh-CN": {
    activate: "启用",
    advanced: "高级",
    about: "关于",
    apply: "应用",
    autoOutputOnBoot: "开机自动输出",
    backlight: "背光",
    buyMeACoffee: "请作者喝咖啡",
    cancel: "取消",
    clearChart: "清空图表",
    closeSettings: "关闭设置",
    connect: "连接",
    connectDeviceInfo: "连接设备后查看信息",
    connectDevicePresets: "连接设备后查看预设",
    connectDp100: "连接 DP100",
    connectViaUsb: "通过 USB 连接你的 Alientek DP100 数控电源",
    connecting: "正在连接...",
    current: "电流",
    currentScan: "电流扫描 (CC)",
    debug: "调试",
    delay: "延时",
    device: "设备",
    disconnect: "断开连接",
    editPreset: "编辑 P{index}",
    end: "结束",
    energy: "电能",
    fast: "快速",
    firmware: "固件",
    fixedCurrent: "固定电流",
    fixedVoltage: "固定电压",
    hardware: "硬件",
    input: "输入",
    language: "语言",
    license: "许可证",
    logPath: "日志：{path}",
    lookingForDp100: "正在查找 DP100...",
    manufactured: "生产日期",
    max: "最大 {value}",
    mid: "中速",
    mode: "模式",
    model: "型号",
    ocp: "过流",
    ocpAmp: "过流保护 (A)",
    off: "关",
    on: "开",
    openSourceDesc: "面向 Alientek DP100 数控电源的开源 macOS 上位机。",
    output: "输出",
    overPower: "过功率保护 (OPP)",
    overTemperature: "过温保护 (OTP)",
    ovp: "过压",
    ovpVolt: "过压保护 (V)",
    pause: "暂停",
    pollingRate: "轮询速率",
    power: "功率",
    presets: "预设",
    protection: "保护",
    protocolLogging: "协议日志",
    rail5v: "5V 供电",
    recordToCsv: "记录为 CSV",
    recording: "记录",
    recordingInfo: "{samples} 条 · {duration}s",
    resume: "继续",
    reverseProtection: "反接保护",
    save: "保存",
    saveCsvLog: "保存 CSV 日志",
    serial: "序列号",
    set: "设置",
    setCurrent: "设置电流",
    setVoltage: "设置电压",
    settings: "设置",
    slow: "慢速",
    stack: "技术栈",
    start: "起始",
    startScan: "开始扫描",
    state: "状态",
    status: "状态",
    step: "步进",
    stopRecording: "停止记录",
    stopScan: "停止扫描",
    temp: "温度",
    tooltipPreset: "单击编辑 - 双击启用",
    voltage: "电压",
    voltageCurrentScanning: "电压 / 电流扫描",
    voltageScan: "电压扫描 (CV)",
    volume: "音量",
    now: "现在",
  },
} as const;

type MessageKey = keyof typeof messages.en;

function initialLanguage(): Language {
  if (typeof localStorage === "undefined") return "en";
  const stored = localStorage.getItem(STORAGE_KEY);
  return stored === "zh-CN" || stored === "en" ? stored : "en";
}

export const language = writable<Language>(initialLanguage());

language.subscribe((value) => {
  if (typeof localStorage !== "undefined") {
    localStorage.setItem(STORAGE_KEY, value);
  }
});

export const languages: { code: Language; label: string }[] = [
  { code: "en", label: "English" },
  { code: "zh-CN", label: "中文" },
];

export function setLanguage(value: Language) {
  language.set(value);
}

export const t = derived(language, ($language) => {
  return (key: MessageKey, params: Record<string, string | number> = {}) => {
    let text: string = messages[$language][key] ?? messages.en[key] ?? key;
    for (const [name, value] of Object.entries(params)) {
      text = text.replaceAll(`{${name}}`, String(value));
    }
    return text;
  };
});
