# DP100 Lab

## Fork status / Fork 当前进展

**English**

This fork is based on upstream `v0.1.0` and is currently maintained on the `i18n-linux-build` development branch.

- Added a Linux x86_64 runnable build for Arch Linux and similar desktop Linux systems.
- Added an English / Chinese UI language switch with persisted preference.
- Localized the main dashboard, settings sheet, chart controls, and CSV save dialog.
- Added `bun run build:app` to build a runnable Tauri binary without AppImage bundling.
- Published a Linux test release: [v0.1.0-linux-i18n](https://github.com/HosakaSu/DP100-Lab/releases/tag/v0.1.0-linux-i18n).

Linux quick start:

```bash
tar -xzf DP100-Lab-v0.1.0-linux-i18n-x86_64.tar.gz
cd DP100-Lab-v0.1.0-linux-i18n-x86_64
./dp100-app
```

The DP100 is accessed through USB HID. On Linux, make sure the `hidraw` node for vendor `2e3c` and product `af01` is readable by the current user.

**中文**

这个 fork 基于上游 `v0.1.0`，当前主要修改维护在 `i18n-linux-build` 开发分支。

- 增加了适用于 Arch Linux 及类似桌面 Linux 系统的 x86_64 可运行构建。
- 增加了英文 / 中文界面切换，并持久保存语言偏好。
- 已本地化主界面、设置面板、图表控件和 CSV 保存对话框。
- 增加 `bun run build:app`，用于构建可直接运行的 Tauri 二进制文件并跳过 AppImage 打包。
- 已发布 Linux 测试版：[v0.1.0-linux-i18n](https://github.com/HosakaSu/DP100-Lab/releases/tag/v0.1.0-linux-i18n)。

Linux 快速运行：

```bash
tar -xzf DP100-Lab-v0.1.0-linux-i18n-x86_64.tar.gz
cd DP100-Lab-v0.1.0-linux-i18n-x86_64
./dp100-app
```

DP100 通过 USB HID 访问。在 Linux 上，请确保 vendor `2e3c`、product `af01` 对应的 `hidraw` 设备节点可由当前用户读取。

---

Native macOS app for the **Alientek DP100** digital power supply. Replaces the Windows-only official software with a modern, fast, and beautiful interface built for macOS.

![DP100 Lab Screenshot](docs/screenshot.png)

## Features

- **Real-time monitoring** — Voltage, current, power, temperature at 20Hz
- **Live chart** — V/I/P traces with pause, hover tooltips, series toggle, speed control
- **Output control** — ON/OFF, set voltage/current with validation
- **10 presets** (P0-P9) — View, edit, save to device, activate
- **System settings** — OPP, OTP, backlight, volume, reverse protection, auto-output
- **CSV data logging** — Record telemetry to disk with millisecond timestamps
- **Voltage/Current scanning** — Automated sweep with configurable range and step
- **Protocol debug logging** — Full TX/RX packet log for troubleshooting
- **macOS native** — Vibrancy, overlay titlebar, dark/light mode, SF Pro font

## Requirements

- macOS 12+ (Apple Silicon)
- Alientek DP100 connected via USB-A (slave mode)

## Install

Download the latest `.dmg` from [Releases](../../releases).

> **Note:** The app is not signed with an Apple Developer certificate. After installing, run:
> ```bash
> xattr -cr "/Applications/DP100 Lab.app"
> ```
> This removes the macOS quarantine flag. Alternatively, right-click the app → Open → Open.

Or build from source:

```bash
# Prerequisites
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
curl -fsSL https://bun.sh/install | bash

# Build
git clone https://github.com/aIeXoid/DP100-Lab.git
cd DP100-Lab
bun install
bun run tauri build
```

The `.app` bundle will be in `src-tauri/target/release/bundle/macos/`.

## Development

```bash
bun install
bun run tauri dev
```

## Architecture

```
dp100-lab/
├── src/                    # Frontend (Svelte 5 + TypeScript)
│   ├── lib/
│   │   ├── components/     # MetricCard, RealtimeChart, SettingsSheet
│   │   └── stores/         # Device state, telemetry, presets
│   └── routes/             # Main dashboard page
├── src-tauri/
│   ├── src/                # Backend (Rust + Tauri v2)
│   │   ├── device.rs       # Device communication, polling, commands
│   │   └── lib.rs          # Tauri command handlers
│   └── dp100_proto/        # USB HID protocol library
│       └── src/lib.rs      # CRC-16, framing, all device operations
└── .github/workflows/      # CI/CD (build + release)
```

### Protocol

Custom USB HID protocol library (`dp100_proto`) built from scratch through hardware testing and manufacturer DLL analysis. Key discoveries:

| Flag | Operation |
|------|-----------|
| `0x20` | Apply to output (immediate) |
| `0x40` | Save to preset storage (flash) |
| `0x80` | Activate preset (switch) |
| `0x40` opcode len=0 | Read system settings |
| `0x40` opcode len=8 | Write system settings |

## Support

If this tool saves you time, consider buying me a coffee:

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-ffdd00?style=flat&logo=buy-me-a-coffee&logoColor=black)](https://buymeacoffee.com/aleXoid)

## License

MIT
