# 🖥️ Desktop Overlay for Windows

A lightweight, modular and click-through **Windows desktop overlay built with Rust and egui/eframe**.

The goal of this project is to provide a modern alternative to traditional desktop widget systems: fast, native, customizable and extensible.

> 🚧 **Early development**
>
> The project is currently under active development.
> The widget system and Windows overlay engine are functional, but many features are still being built.

---

## ✨ Overview

Desktop Overlay runs as a transparent layer above the Windows desktop.

It can display multiple independent widgets while allowing mouse clicks to pass through the overlay when it is locked.

Press **F8** at any time to switch between:

**🔒 Locked mode**

- Transparent overlay
- Click-through enabled
- Widgets stay visible
- Your desktop remains fully usable

**🛠️ Edit mode**

- Click-through disabled
- Widget configuration available
- Widgets can be moved and configured
- Overlay settings become accessible

Press **F8** again to return to locked mode.

---

# 🚀 Features

## 🪟 Native Windows overlay

- Transparent desktop overlay
- Always-on-top window
- Borderless window
- Hidden from the taskbar
- Click-through support
- Global F8 edit shortcut
- Low-overhead Rust application
- WGPU accelerated rendering

## 🧩 Modular widget architecture

Widgets are designed as independent modules.

The long-term goal is to make it possible to enable, disable and configure widgets without modifying the overlay engine.

Planned widget categories include:

- Service monitoring
- System monitoring
- Network monitoring
- Weather
- Clock and calendar
- Media controls
- Server monitoring
- Development tools
- Custom HTTP/API widgets
- Notes and shortcuts

---

# 🚨 Service Monitor

The first widget included in the project is **Service Monitor**.

It monitors online services and displays their current state directly on your desktop.

Possible states include:

```text
🟢 Operational
🟠 Degraded
🔴 Outage
```

The widget can display:

- Service name
- Current status
- Last check
- Monitoring errors
- Recent outage state
- Multiple monitored services

The monitoring system runs separately from the UI so network requests do not freeze the overlay.

---

# 📡 Downdetector support

The project contains a provider designed for the official **Downdetector API**.

Supported status values include:

```text
success
warning
danger
```

The Downdetector API requires access to Downdetector Enterprise.

The architecture does not depend exclusively on Downdetector. Other providers can be added without rewriting the widget.

For example:

```text
ServiceStatusWidget
        │
        ▼
ServiceStatusProvider
        │
        ├── Downdetector
        ├── Generic HTTP
        ├── Official status pages
        ├── Custom API
        └── Future providers
```

---

# 🌐 Generic HTTP monitoring

A generic HTTP provider is also included.

This makes it possible to monitor services without requiring Downdetector.

Example use cases:

```text
Discord API
Personal websites
VPS servers
REST APIs
Game servers
Self-hosted applications
Home servers
Monitoring endpoints
```

More monitoring methods will be added later.

---

# 🎮 Controls

| Key  | Action                    |
| ---- | ------------------------- |
| `F8` | Toggle Edit / Locked mode |

### Locked mode

```text
Overlay visible
       │
       ├── Widgets visible
       ├── Transparent background
       └── Mouse clicks pass through
```

### Edit mode

```text
Press F8
   │
   ▼
Click-through disabled
   │
   ├── Settings available
   ├── Widgets editable
   └── Widgets movable
```

---

# 🛠️ Requirements

Currently supported:

```text
Windows 10
Windows 11
```

Development requirements:

```text
Rust
Cargo
Windows
```

The application currently targets Windows because the overlay engine uses native Windows window behavior.

Cross-platform support may be investigated later.

---

# 📦 Installation

Clone the repository:

```bash
git clone YOUR_REPOSITORY
cd desktop-overlay-rust
```

Run the development build:

```bash
cargo run
```

Or:

```bash
cargo run --release
```

---

# ⚡ Release build

Create an optimized executable with:

```bash
cargo build --release
```

The executable will then be generated inside:

```text
target/release/
```

---

# 🗂️ Project structure

```text
desktop-overlay-rust/
│
├── Cargo.toml
├── Cargo.lock
├── README.md
│
└── src/
    ├── main.rs
    ├── app.rs
    │
    ├── core/
    │   ├── mod.rs
    │   ├── monitor.rs
    │   └── windows_overlay.rs
    │
    ├── providers/
    │   ├── mod.rs
    │   ├── status_provider.rs
    │   └── downdetector.rs
    │
    └── widgets/
        ├── mod.rs
        └── service_status.rs
```

The architecture intentionally separates:

```text
Window / Overlay engine
        │
Widget manager
        │
Widgets
        │
Providers / APIs
```

This keeps the project maintainable as more widgets are introduced.

---

# 🧠 Architecture

The application is built around several independent layers.

## Overlay engine

Responsible for:

- Transparent Windows window
- Click-through
- Always-on-top behavior
- Edit mode
- Rendering

## Widget layer

Responsible for:

- Widget rendering
- Widget position
- Widget state
- Widget configuration

## Monitoring layer

Responsible for:

- Background service checks
- Network requests
- Status updates
- Communication with the UI

## Provider layer

Responsible for communicating with external monitoring sources.

Example:

```text
              ┌─────────────────┐
              │ Desktop Overlay │
              └────────┬────────┘
                       │
                ┌──────▼──────┐
                │   Widgets   │
                └──────┬──────┘
                       │
                ┌──────▼──────┐
                │   Monitor   │
                └──────┬──────┘
                       │
            ┌──────────▼──────────┐
            │ Status Providers    │
            └──────┬──────┬──────┘
                   │      │
           Downdetector  HTTP
```

---

# 🔐 Security

Do **not** commit API keys or private access tokens to GitHub.

Local configuration files containing secrets should remain ignored by Git.

For example:

```text
overlay-config.json
.env
```

If an API token is accidentally committed to a public repository, consider it compromised and replace it immediately.

---

# 🧩 Planned widgets

## System

- [ ] CPU usage
- [ ] GPU usage
- [ ] RAM usage
- [ ] CPU temperature
- [ ] GPU temperature
- [ ] Disk usage
- [ ] Running processes

## Network

- [ ] Ping
- [ ] Download speed
- [ ] Upload speed
- [ ] Packet loss
- [ ] Public IP
- [ ] VPN status

## Services

- [x] Service monitoring engine
- [x] Downdetector provider
- [x] HTTP provider
- [ ] Discord status
- [ ] Steam status
- [ ] GitHub status
- [ ] Cloudflare status
- [ ] Microsoft services
- [ ] Twitch status

## Desktop

- [ ] Clock
- [ ] Calendar
- [ ] Weather
- [ ] Notes
- [ ] Application shortcuts
- [ ] Timer
- [ ] Stopwatch

## Media

- [ ] Current media
- [ ] Spotify integration
- [ ] Playback controls
- [ ] Volume controls

## Developer tools

- [ ] GitHub notifications
- [ ] Git repository status
- [ ] Docker containers
- [ ] Server monitoring
- [ ] HTTP endpoint monitoring
- [ ] Custom API widgets

---

# 🗺️ Roadmap

## v0.1 — Overlay foundation

- [x] Rust application
- [x] egui / eframe rendering
- [x] Transparent Windows overlay
- [x] Click-through
- [x] Global F8 shortcut
- [x] Edit mode
- [x] Service monitoring widget
- [x] Background monitoring
- [x] Configuration persistence

## v0.2 — Widget engine

- [ ] Generic widget manager
- [ ] Drag and drop
- [ ] Resize widgets
- [ ] Widget enable/disable menu
- [ ] Widget configuration interface
- [ ] Snap-to-grid
- [ ] Multiple layouts
- [ ] Import/export layouts

## v0.3 — System monitoring

- [ ] CPU widget
- [ ] GPU widget
- [ ] RAM widget
- [ ] Temperature sensors
- [ ] Network statistics
- [ ] Performance graphs

## v0.4 — Customization

- [ ] Themes
- [ ] Accent colors
- [ ] Widget opacity
- [ ] Widget backgrounds
- [ ] Font configuration
- [ ] Animations
- [ ] Multiple monitors

## Future

- [ ] Widget SDK
- [ ] Plugin system
- [ ] Community widgets
- [ ] Automatic updates
- [ ] Widget marketplace
- [ ] Layout sharing
- [ ] Windows startup option

---

# 🤝 Contributing

Contributions are welcome.

You can contribute by:

- Reporting bugs
- Suggesting widgets
- Improving documentation
- Improving Windows compatibility
- Creating monitoring providers
- Creating new widgets
- Optimizing performance
- Improving the UI

For large changes, creating an issue describing the proposed feature before implementing it is recommended.

---

# 🐛 Bug reports

When reporting a bug, please include:

```text
Windows version
Rust version
Application version
Steps to reproduce
Expected behavior
Actual behavior
Console output
Screenshots if relevant
```

This makes debugging considerably easier.

---

# 💡 Widget ideas

Have an idea for a useful desktop widget?

Open an issue and describe:

```text
What the widget displays
Where the data comes from
How often it should update
What configuration options it needs
```

Community widget ideas are welcome.

---

# ⚙️ Technology

The project currently uses:

```text
Rust
egui
eframe
WGPU
Win32 APIs
Serde
Reqwest
```

Rust provides native performance and memory safety while egui makes it possible to rapidly build dynamic desktop interfaces.

---

# 🎯 Project goals

Desktop Overlay aims to be:

**Fast**

Minimal CPU and memory usage.

**Modular**

Widgets should remain independent from the overlay engine.

**Customizable**

Users should eventually be able to build their own desktop layout.

**Native**

No browser runtime or Electron dependency.

**Extensible**

New widgets and monitoring providers should be easy to implement.

**Open**

The long-term goal is to allow community-created widgets and integrations.

---

# ⭐ Support the project

If you find the project useful, consider starring the repository.

Stars help other developers discover the project and give useful feedback on which features should be prioritized.

Bug reports, feature requests and pull requests are also greatly appreciated.

---

# 📄 License

A license has not yet been finalized.

For an open-source project, the MIT or Apache-2.0 licenses are good candidates.

---

## GitHub Topics

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
widgets
system-monitor
monitoring
downdetector
windows-11
rust-gui
win32
```

---

Built with 🦀 Rust.
