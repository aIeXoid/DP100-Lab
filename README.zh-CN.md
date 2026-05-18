# DP100 Lab

[English README](README.md)

DP100 Lab 是面向 **Alientek DP100** 数控电源的原生桌面上位机。它提供实时监控、输出控制、图表、预设、系统设置、CSV 记录和协议调试等功能。

![DP100 Lab Screenshot](docs/screenshot.png)

## 功能

- **实时监控**：电压、电流、功率、温度，最高 20Hz
- **实时图表**：电压 / 电流 / 功率曲线，支持暂停、悬停提示、曲线开关和刷新速度控制
- **输出控制**：输出开关、电压 / 电流设置和输入校验
- **10 组预设**：查看、编辑、保存到设备、启用
- **系统设置**：OPP、OTP、背光、音量、反接保护、开机自动输出
- **CSV 数据记录**：以毫秒时间戳记录遥测数据
- **电压 / 电流扫描**：按范围和步进自动扫描
- **协议调试日志**：记录完整 TX/RX 数据包，便于排查问题
- **中英文切换**：界面语言可在应用标题栏中切换，并会保存偏好设置

## 系统要求

- macOS 12+（Apple Silicon）或 Linux x86_64
- Alientek DP100 通过 USB-A 连接，并处于从机模式

## 安装

### macOS

从 [Releases](../../releases) 下载最新 `.dmg`。

> **注意：** 应用没有使用 Apple Developer 证书签名。安装后可以运行：
> ```bash
> xattr -cr "/Applications/DP100 Lab.app"
> ```
> 这会移除 macOS quarantine 标记。也可以右键应用，选择“打开”。

### Linux

从 [Releases](../../releases) 下载 Linux x86_64 压缩包，解压后运行：

```bash
mkdir dp100-lab
tar -xzf DP100-Lab_*_x86_64.tar.gz -C dp100-lab --strip-components=1
./dp100-lab/dp100-app
```

DP100 通过 USB HID 访问。如果应用无法连接设备，请确认 vendor `2e3c`、product `af01` 对应的 `hidraw` 设备节点可由当前用户读取。可以使用 udev 规则授权：

```text
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="2e3c", ATTRS{idProduct}=="af01", MODE="0666", TAG+="uaccess"
```

添加规则后重新加载并重新连接设备：

```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### 从源码构建

```bash
# 准备依赖
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
curl -fsSL https://bun.sh/install | bash

# 构建
git clone https://github.com/aIeXoid/DP100-Lab.git
cd DP100-Lab
bun install
bun run tauri build
```

macOS `.app` 会生成在 `src-tauri/target/release/bundle/macos/`。

Linux 可使用以下命令构建可直接运行的二进制文件，并跳过 AppImage 打包：

```bash
bun run build:app
```

## 开发

```bash
bun install
bun run tauri dev
```

## 架构

```
dp100-lab/
├── src/                    # 前端（Svelte 5 + TypeScript）
│   ├── lib/
│   │   ├── components/     # MetricCard, RealtimeChart, SettingsSheet
│   │   └── stores/         # 设备状态、遥测、预设
│   └── routes/             # 主仪表盘页面
├── src-tauri/
│   ├── src/                # 后端（Rust + Tauri v2）
│   │   ├── device.rs       # 设备通信、轮询、命令
│   │   └── lib.rs          # Tauri 命令处理
│   └── dp100_proto/        # USB HID 协议库
│       └── src/lib.rs      # CRC-16、帧封装、设备操作
└── .github/workflows/      # CI/CD（构建和发布）
```

### 协议

自定义 USB HID 协议库（`dp100_proto`）基于硬件测试和厂商 DLL 分析实现。关键发现：

| Flag | 操作 |
|------|------|
| `0x20` | 应用到输出（立即生效） |
| `0x40` | 保存到预设存储（Flash） |
| `0x80` | 启用预设（切换） |
| `0x40` opcode len=0 | 读取系统设置 |
| `0x40` opcode len=8 | 写入系统设置 |

## 支持

如果这个工具节省了你的时间，可以考虑请作者喝杯咖啡：

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-ffdd00?style=flat&logo=buy-me-a-coffee&logoColor=black)](https://buymeacoffee.com/aleXoid)

## 许可证

MIT
