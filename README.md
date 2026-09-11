<div align="center">

# 🧩 Desktop Overlay Rust

### A modular, transparent and click-through Windows desktop overlay built with Rust + egui

**System monitoring · Service outage monitoring · GPU/CPU/RAM/Disk widgets · System tray · Professional settings UI · Extensible architecture**

![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust&logoColor=white)
![Windows](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D4?logo=windows11&logoColor=white)
![egui](https://img.shields.io/badge/egui%20%2F%20eframe-0.36.2-8A2BE2)
![Status](https://img.shields.io/badge/status-active%20development-yellow)

</div>

---

## ✨ Overview

**Desktop Overlay Rust** is a lightweight Windows desktop overlay designed to display useful widgets while staying out of the way.

The overlay is:

- transparent;
- borderless;
- always on top;
- click-through when locked;
- configurable through a dedicated Windows settings window;
- controlled through a system tray icon.

The goal is to create a modern, native and extensible alternative to desktop widget tools such as Rainmeter, but built entirely in Rust.

---

## 🚀 Features

### 🪟 Overlay engine

- Transparent Windows overlay
- Borderless fullscreen window
- Always on top
- Hidden from the taskbar
- Click-through locked mode
- Editable widget layout
- Persistent widget positions
- WGPU accelerated rendering
- Win32 integration for stable click-through behavior

---

### 🧩 Modular widget system

Each widget can be independently:

- enabled;
- disabled;
- moved;
- configured;
- reset to its default position.

The widget configuration is stored in:

```text
overlay-config.json
```

---

## 🖥️ Current widgets

### 🚨 Service Monitor

Monitors external services and APIs.

Supported providers:

- Generic HTTP health checks
- Downdetector Enterprise API

Possible states:

```text
🟢 Operational
🟠 Degraded
🔴 Outage
```

The widget can display:

- service name;
- current status;
- status details;
- last check time;
- degradation/outage alerts.

Each monitored service can also be enabled or disabled independently.

---

### 🕒 Clock

Displays local Windows time.

Available options:

- show/hide seconds;
- show/hide date.

---

### 📊 System Monitor

Displays live computer statistics.

Current metrics:

- CPU usage
- GPU usage
- RAM usage
- Disk usage
- Disk read/write activity
- System uptime
- Hostname

Each metric can be individually enabled or disabled.

Example:

```text
┌──────────────────────────────────────┐
│ SYSTÈME                              │
│ MY-PC                                │
│                                      │
│ CPU                           12 %    │
│ ███░░░░░░░░░░░░░░░░░░░░░░░░░      │
│                                      │
│ GPU                           34 %    │
│ ████████░░░░░░░░░░░░░░░░░░░░      │
│ NVIDIA GeForce ...                   │
│                                      │
│ RAM                    11 / 32 GiB   │
│ █████████░░░░░░░░░░░░░░░░░░░      │
│                                      │
│ Disk activity                21 %    │
│ Read  18 MiB/s                       │
│ Write  4 MiB/s                       │
│                                      │
│ C:                     423 / 931 GiB │
│ ███████████░░░░░░░░░░░░░░░░       │
│                                      │
│ Uptime: 4 h 27 min                   │
└──────────────────────────────────────┘
```

---

## 🖱️ System Tray Control

The application is controlled through an icon in the Windows notification area.

Right-click the tray icon to access:

```text
Desktop Overlay
├── Settings...
├── Edit layout
├──────────────
├── Refresh services
├──────────────
└── Quit
```

### Settings

Opens a dedicated Windows settings window.

### Edit layout

Temporarily unlocks the overlay so widgets can be moved.

When layout editing is enabled:

- click-through is disabled;
- widgets become draggable;
- a small editing toolbar appears;
- the settings window is temporarily hidden.

When editing is finished:

- the overlay is locked again;
- click-through is restored;
- the settings window can reopen automatically.

---

## ⚙️ Settings Interface

The project includes a dedicated settings window instead of putting configuration directly inside the overlay.

The interface is split into sections:

```text
General
Widgets
System
Services
```

### General

Contains:

- overlay lock/edit status;
- edit layout control;
- configuration path;
- system tray information.

### Widgets

Allows users to:

- enable/disable every widget;
- reset widget positions;
- enable/disable clock options.

### System

Allows users to configure:

- CPU display;
- GPU display;
- RAM display;
- disk display;
- disk I/O;
- uptime;
- refresh interval.

### Services

Allows users to:

- configure Downdetector;
- configure HTTP monitoring;
- enable/disable services;
- add/remove monitored services;
- change polling interval;
- manually refresh service status.

---

## 🎯 Project Vision

The long-term goal is to build a native Windows desktop widget platform where users can enable only the components they need.

Planned categories include:

| Category    | Planned widgets                               |
| ----------- | --------------------------------------------- |
| 🖥️ System   | CPU, GPU, RAM, disks, temperatures, processes |
| 🌐 Network  | Ping, bandwidth, packet loss, public IP, VPN  |
| ☁️ Services | Downdetector, HTTP endpoints, status pages    |
| 🐳 DevOps   | Docker, VPS, GitHub, CI/CD                    |
| 🎵 Media    | Spotify, currently playing media              |
| 🌦️ Daily    | Weather, clock, calendar                      |
| 📝 Utility  | Notes, timers, shortcuts                      |
| 🔌 Custom   | REST/JSON widgets, plugins                    |

---

## 🏗️ Architecture

```text
src/
├── main.rs
├── app.rs
│
├── core/
│   ├── config.rs
│   ├── monitor.rs
│   ├── system_stats.rs
│   ├── gpu_stats.rs
│   ├── disk_stats.rs
│   ├── tray.rs
│   ├── widget_registry.rs
│   ├── windows_overlay.rs
│   └── mod.rs
│
├── providers/
│   ├── downdetector.rs
│   ├── http.rs
│   └── mod.rs
│
└── widgets/
    ├── service_status.rs
    ├── clock.rs
    ├── system_monitor.rs
    └── mod.rs
```

---

## 🧠 Internal Architecture

The project is separated into multiple layers:

```text
Windows / Overlay layer
        │
        ▼
Widget Registry
        │
        ▼
Widgets
        │
        ▼
Monitoring Workers
        │
        ▼
External Providers / System APIs
```

This makes it easier to add new widgets without modifying the core overlay logic.

---

## 🪟 Windows Overlay Behavior

The overlay is created as:

- transparent;
- borderless;
- maximized;
- always on top;
- hidden from the taskbar;
- mouse-pass-through when locked.

The application uses a transparent WGPU surface.

The native Win32 helper only changes the `WS_EX_TRANSPARENT` flag when switching between locked and edit mode.

This is intentional.

Changing the complete native window style at runtime can break transparency on Windows, especially after WGPU has already created its rendering surface.

---

## 📡 Service Monitoring

### Generic HTTP provider

The HTTP provider checks an endpoint and compares the returned HTTP status code.

Example:

```json
{
  "id": "my-api",
  "name": "My API",
  "enabled": true,
  "provider": "http",
  "url": "https://example.com/health",
  "expected_status": 200
}
```

---

### Downdetector

The project supports the official Downdetector Enterprise API.

Example endpoint:

```text
GET https://downdetectorapi.com/v2/companies/{company_id}/status
Authorization: Bearer <token>
```

Supported Downdetector states:

| Downdetector | Overlay     |
| ------------ | ----------- |
| `success`    | Operational |
| `warning`    | Degraded    |
| `danger`     | Outage      |

For security, the token can be provided through an environment variable:

```powershell
$env:DOWNDETECTOR_TOKEN="YOUR_TOKEN"
cargo run --release
```

The token entered through the settings interface remains in memory only.

---

## 📊 System Monitoring

System information is collected using Windows and Rust system APIs.

### CPU / RAM

Collected through `sysinfo`.

### Disk

The application displays:

- total space;
- used space;
- per-disk usage;
- disk read activity;
- disk write activity.

### GPU

GPU usage is collected using Windows GPU performance information.

GPU collection is performed outside the rendering path so it does not block the overlay UI.

> GPU usage availability can depend on the GPU driver and Windows performance counters.

---

## 🚀 Getting Started

### Requirements

- Windows 10 or Windows 11
- Rust stable
- Cargo
- Visual Studio Build Tools with MSVC if required by your Rust installation

Install Rust:

```text
https://rustup.rs/
```

Verify installation:

```powershell
rustc --version
cargo --version
```

---

### Clone

```powershell
git clone https://github.com/YOUR_USERNAME/desktop-overlay-rust.git
cd desktop-overlay-rust
```

---

### Development

```powershell
cargo run
```

---

### Optimized build

```powershell
cargo run --release
```

Or:

```powershell
cargo build --release
```

The executable will be created in:

```text
target/release/desktop-overlay.exe
```

---

## 🧪 Development Checks

Before committing:

```powershell
cargo fmt
cargo check
```

Optional:

```powershell
cargo clippy
```

---

## 🔐 Security

Never commit private API tokens.

Recommended `.gitignore` entries:

```gitignore
/target/

overlay-config.json

.env
.env.*

*.log

.idea/
.vscode/

Thumbs.db
Desktop.ini
```

Keep `Cargo.lock` committed for this application.

---

## 🛣️ Roadmap

### v0.1 — Overlay foundation

- [x] Transparent overlay
- [x] Click-through mode
- [x] Always-on-top
- [x] Service Monitor
- [x] HTTP monitoring
- [x] Downdetector provider
- [x] Configuration persistence

### v0.2 — Widget system

- [x] Widget registry
- [x] Enable/disable widgets
- [x] Persistent widget positions
- [x] Clock widget
- [x] CPU/RAM monitoring
- [x] Per-widget options

### v0.3 — Desktop application UX

- [x] System tray icon
- [x] Tray context menu
- [x] Dedicated settings window
- [x] Professional settings navigation
- [x] Layout editing mode
- [x] GPU usage
- [x] Disk usage
- [x] Disk I/O monitoring
- [x] Individual system metric toggles

### v0.4 — Network monitoring

- [ ] Ping widget
- [ ] Packet loss
- [ ] Download speed
- [ ] Upload speed
- [ ] Network adapter selection
- [ ] Public IP
- [ ] VPN status

### v0.5 — Hardware monitoring

- [ ] CPU temperature
- [ ] GPU temperature
- [ ] Fan speeds
- [ ] VRAM usage
- [ ] Per-core CPU usage
- [ ] Disk temperature

### Future

- [ ] Themes
- [ ] Multiple monitors
- [ ] Snap-to-grid
- [ ] Widget resizing
- [ ] Multiple widget instances
- [ ] Custom REST/JSON widgets
- [ ] Plugin SDK
- [ ] Community widgets
- [ ] Automatic updates
- [ ] Start with Windows
- [ ] Import/export layouts
- [ ] Native notifications

---

## 🤝 Contributing

Contributions are welcome.

Useful contribution areas include:

- new widgets;
- monitoring providers;
- Windows overlay fixes;
- performance improvements;
- UI/UX improvements;
- documentation;
- hardware monitoring;
- network monitoring.

Recommended workflow:

```powershell
git checkout -b feature/my-widget

cargo fmt
cargo check

git add .
git commit -m "feat: add my widget"
git push -u origin feature/my-widget
```

Then open a pull request.

---

## 🐛 Bug Reports

When reporting a bug, please include:

```text
Windows version
GPU model
GPU driver version
Rust version
Application version
Steps to reproduce
Expected behavior
Actual behavior
Console output
Screenshot
```

For overlay/transparency problems, GPU and driver information are especially useful.

---

## 🔎 GitHub Topics

Recommended repository topics:

```text
rust
windows
overlay
desktop-overlay
desktop-widgets
egui
eframe
wgpu
win32
system-monitor
monitoring
gpu-monitor
cpu-monitor
desktop-app
rust-gui
```

---

## ⭐ Support

If you find the project useful, starring the repository helps other developers discover it.

Bug reports, feature requests and pull requests are welcome.

---

<div align="center">

Built with **Rust**, **egui/eframe**, **WGPU** and the **Win32 API**.

</div>
