# NewSquirrelHome — Comprehensive Feature Plan

**Project**: SelfHost Helper
**Version target**: 1.0.0
**Date**: 2026-06-15
**Status**: Planning (no code)

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Architecture Overview](#2-architecture-overview)
3. [System 1: Cross-Platform Performance Engine](#3-system-1-cross-platform-performance-engine)
4. [System 2: Memory Management & Leak Prevention](#4-system-2-memory-management--leak-prevention)
5. [System 3: GPU Optimization](#5-system-3-gpu-optimization)
6. [System 4: Display Server Support](#6-system-4-display-server-support-wayland--x11--windows--macos)
7. [System 5: UI Customization Engine](#7-system-5-ui-customization-engine)
8. [System 6: Network Management](#8-system-6-network-management)
9. [System 7: Notification System](#9-system-7-notification-system)
10. [System 8: Per-Project Deep Configuration](#10-system-8-per-project-deep-configuration)
11. [System 9: Performance Monitoring Dashboard](#11-system-9-performance-monitoring-dashboard)
12. [System 10: Platform-Specific Optimizations](#12-system-10-platform-specific-optimizations)
13. [System 11: Security & Hardening](#13-system-11-security--hardening)
14. [System 12: Build & Release Pipeline](#14-system-12-build--release-pipeline)
15. [System 13: Accessibility](#15-system-13-accessibility)
16. [Implementation Phases](#16-implementation-phases)
17. [Database Migration Plan](#17-database-migration-plan)
18. [Risk Assessment](#18-risk-assessment)
19. [Estimated Effort](#19-estimated-effort)
20. [Full TODO Checklist](#20-full-todo-checklist)

---

## 1. Executive Summary

**NewSquirrelHome** is a ground-up evolution of SelfHost Helper from a functional local dev project manager into a production-grade, cross-platform development environment platform. It transforms the app from "runs my Node projects" to "is my development command center" — with intelligent resource management, deep OS integration, universal notifications, and a customizable UI that adapts to any machine from a Raspberry Pi to a 96-core workstation.

### Vision Statement

> Every developer deserves a local dev tool that respects their machine, their workflow, and their time. NewSquirrelHome makes SelfHost Helper the tool that *knows* your hardware, *protects* your resources, and *adapts* to how you work.

### Core Principles

1. **Adaptive** — detect hardware at startup, auto-tune, never assume a "normal" machine
2. **Non-invasive** — resource limits, monitoring, and notifications are opt-in and configurable
3. **Universal** — Linux (Wayland/X11), macOS, Windows 11, all GPU vendors, headless/VM
4. **Composable** — every system is independently useful; no forced coupling
5. **Observable** — every action logged, every resource measured, every state visible

---

## 2. Architecture Overview

### 2.1 Current State

```
src-tauri/src/
  lib.rs              — AppState, 95 invoke_handler commands
  error.rs            — AppError enum with safe_message()
  db/                 — rusqlite repos (projects, categories, settings)
  services/           — 11 service modules (backup, git, lsp, runtime, search, settings, tunnel, update, file_watcher, log_store)
  native/             — tray
  utils/              — crypto, path_security, media_allowlist, sidecar, ignore_patterns

src/                  — React + Vite + Jotai renderer
  components/         — 21 components
  pages/              — Dashboard, Settings (6 sections)
  store/              — Jotai atoms (projects, categories, logs, stats, tunnels, editor state)
```

### 2.2 New Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│                    Frontend (React + Jotai)              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │ Theme    │ │ Dashboard│ │ Settings │ │ Notif UI │  │
│  │ Engine   │ │ Panels   │ │ Sections │ │ Builder  │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘  │
│       │             │             │             │        │
│  ┌────┴─────────────┴─────────────┴─────────────┴────┐  │
│  │              Jotai Atoms (new: perf, mem, net,     │  │
│  │              gpu, theme, notif, dashboard state)   │  │
│  └───────────────────────┬───────────────────────────┘  │
├──────────────────────────┼──────────────────────────────┤
│                    IPC Bridge (Tauri commands)           │
├──────────────────────────┼──────────────────────────────┤
│                    Backend (Rust)                        │
│  ┌───────────────────────┴───────────────────────────┐  │
│  │              Core Services (existing)              │  │
│  │  backup, git, lsp, runtime, search, settings,     │  │
│  │  tunnel, update, file_watcher, log_store          │  │
│  └───────────────────────┬───────────────────────────┘  │
│  ┌───────────────────────┴───────────────────────────┐  │
│  │           New Service Layer (NSH)                  │  │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │  │
│  │  │ Performance │ │ Memory      │ │ GPU         │ │  │
│  │  │ Manager     │ │ Monitor     │ │ Detector    │ │  │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ │  │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │  │
│  │  │ Display     │ │ Network     │ │ Notification│ │  │
│  │  │ Detector    │ │ Monitor     │ │ Engine      │ │  │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ │  │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │  │
│  │  │ Process     │ │ Audit       │ │ Platform    │ │  │
│  │  │ Supervisor  │ │ Logger      │ │ Integrator  │ │  │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ │  │
│  └───────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────┐  │
│  │           Platform Abstraction Layer (PAL)         │  │
│  │  Linux (cgroups, /proc, nftables, D-Bus)          │  │
│  │  macOS (Metal, GCD, LaunchAgent, Spotlight)       │  │
│  │  Windows (Job Objects, WMI, BITS, DWM)            │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### 2.3 Module Organization (New Files)

```
src-tauri/src/
  services/
    performance/
      mod.rs                    — PerformanceManager (orchestrator)
      detector.rs               — System capability detection
      profiles.rs               — Profile definitions (potato..monster)
      gpu.rs                    — GPU detection and vendor profiles
      display.rs                — Display server detection (Wayland/X11)
    memory/
      mod.rs                    — MemoryManager
      monitor.rs                — Periodic RSS sampling
      leak_detector.rs          — Growth rate analysis
      limiter.rs                — Per-project and global limits
    network/
      mod.rs                    — NetworkManager
      monitor.rs                — Bandwidth/connection tracking
      limiter.rs                — Per-project and global limits
      platform/                 — Platform-specific implementations
        linux.rs
        macos.rs
        windows.rs
    notification/
      mod.rs                    — NotificationEngine
      channels/                 — Channel implementations
        in_app.rs
        email.rs
        webhook.rs
        websocket.rs
        desktop.rs
        telegram.rs
        discord.rs
        slack.rs
      rules.rs                  — Rules engine
      templates.rs              — Message templating
      history.rs                — Audit trail
    dashboard/
      mod.rs                    — DashboardManager
      collector.rs              — System metrics collection
      alerting.rs               — Alert management
      exporter.rs               — CSV/JSON export
    audit/
      mod.rs                    — AuditLogger
    process_supervisor/
      mod.rs                    — Enhanced process lifecycle
      resource_limiter.rs       — cgroup/Job Object enforcement
    platform/
      mod.rs                    — Platform trait + factory
      linux.rs                  — Linux PAL
      macos.rs                  — macOS PAL
      windows.rs                — Windows PAL
  native/
    tray.rs                     — (existing, enhanced)
    notifications.rs            — OS-native notification bridge

src/
  components/
    theme/                      — Theme engine components
      ThemeProvider.jsx
      ColorPicker.jsx
      ThemeImportExport.jsx
    dashboard/                  — Dashboard panels
      SystemOverview.jsx
      ProcessPanel.jsx
      NetworkPanel.jsx
      AlertsPanel.jsx
      MemoryGraph.jsx
      GPUPanel.jsx
      DashboardGrid.jsx
    notifications/              — Notification UI
      NotificationBuilder.jsx
      ChannelConfig.jsx
      RulesEditor.jsx
      NotificationHistory.jsx
    settings/                   — New settings sections
      PerformanceSection.jsx
      MemorySection.jsx
      NetworkSection.jsx
      NotificationSection.jsx
      DashboardSection.jsx
      AccessibilitySection.jsx
      ThemeSection.jsx
      FeatureTogglesSection.jsx
    project-settings/           — Deep per-project config
      CommandConfig.jsx
      ResourceLimits.jsx
      MonitoringConfig.jsx
      ProjectNotifications.jsx
  store/
    atoms.js                    — (existing, extended with new atoms)
    perfAtoms.js                — Performance state
    memoryAtoms.js              — Memory monitoring state
    networkAtoms.js             — Network monitoring state
    notificationAtoms.js        — Notification state
    dashboardAtoms.js           — Dashboard state
    themeAtoms.js               — Theme state
```

---

## 3. System 1: Cross-Platform Performance Engine

### 3.1 Purpose

Automatically detect system capabilities and select an appropriate performance profile so the app runs well on everything from a 2GB RAM Raspberry Pi to a 128GB workstation with an RTX 4090.

### 3.2 Machine Classification

| Tier | CPU Cores | RAM | GPU | Description |
|------|-----------|-----|-----|-------------|
| `potato` | 1-2 | ≤2GB | None/very old | Headless, VM, Raspberry Pi Zero |
| `low-end` | 2-4 | 2-4GB | Intel integrated (old) | Budget laptops, old desktops |
| `mid-range` | 4-8 | 4-16GB | Intel/AMD integrated, old NVIDIA | Typical dev machines |
| `high-end` | 8-16 | 16-32GB | NVIDIA/AMD discrete (recent) | Good dev machines |
| `monster` | 16+ | 32GB+ | NVIDIA 40xx/AMD 7xxx or better | Workstations, gaming rigs |

Classification logic: take the minimum of CPU-score and RAM-score, then apply GPU modifier (+1 tier for good GPU, -1 for no GPU).

### 3.3 Performance Profiles

| Setting | potato | low-end | mid-range | high-end | monster |
|---------|--------|---------|-----------|----------|---------|
| Animation speed | off | fast | normal | normal | normal |
| Transition effects | none | fade | slide | slide | scale |
| Background blur | off | off | off (intensity 0) | on (intensity 0.5) | on (intensity 1.0) |
| Gradient rendering | off | off | on | on | on |
| Shadow quality | none | simple | blur | blur | gaussian |
| Icon rendering | simple | simple | standard | high-res | high-res+animated |
| Auto-refresh interval | 30s | 10s | 5s | 5s | 1s |
| Log retention | 100 | 500 | 1000 | 1000 | 5000 |
| Max concurrent processes | 4 | 8 | 16 | 32 | 64 |
| Terminal buffer size | 1000 | 1000 | 5000 | 5000 | 10000 |

### 3.4 Data Structures (Conceptual)

```
SystemCapabilities:
  cpu_cores: u32
  cpu_score: f64            // benchmark-derived
  ram_total_mb: u64
  ram_available_mb: u64
  gpu_vendor: GpuVendor     // None, Intel, NVIDIA, AMD
  gpu_model: String
  gpu_vram_mb: Option<u64>
  gpu_driver_version: Option<String>
  display_server: DisplayServer  // Wayland, X11, Windows, macOS
  os: OperatingSystem
  arch: Architecture         // x86_64, aarch64

MachineTier: enum { Potato, LowEnd, MidRange, HighEnd, Monster }

PerformanceProfile:
  animation_speed: AnimationSpeed
  transition_effects: TransitionEffects
  background_blur: BackgroundBlurConfig
  gradient_rendering: bool
  shadow_quality: ShadowQuality
  icon_rendering: IconRendering
  auto_refresh_interval: Duration
  log_retention_count: usize
  max_concurrent_processes: usize
  terminal_buffer_size: usize

PerformanceConfig:
  auto_detect: bool              // default: true
  manual_tier_override: Option<MachineTier>
  manual_profile: Option<PerformanceProfile>  // overrides everything
  per_setting_overrides: HashMap<String, Value>
```

### 3.5 Storage Schema

New table `performance_config`:
```
performance_config (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
)
```

Keys: `auto_detect`, `tier_override`, `animation_speed`, `transition_effects`, `blur_enabled`, `blur_intensity`, `gradient_rendering`, `shadow_quality`, `icon_rendering`, `auto_refresh_interval`, `log_retention`, `max_concurrent_processes`, `terminal_buffer_size`

### 3.6 IPC Commands

| Command | Description |
|---------|-------------|
| `detect_system_capabilities` | Run full hardware detection, return SystemCapabilities |
| `get_performance_profile` | Get current active profile |
| `set_performance_profile` | Set a named profile (potato..monster) |
| `set_performance_setting` | Override a single setting |
| `reset_performance_settings` | Revert to auto-detected defaults |
| `get_performance_config` | Get full config including overrides |

### 3.7 Frontend Components

- `PerformanceSettings.jsx` — Settings page section with tier display, per-setting sliders, auto-detect toggle
- `SystemInfoCard.jsx` — Read-only display of detected hardware

### 3.8 Platform Considerations

- **Linux**: Read `/proc/cpuinfo`, `/proc/meminfo`, `/sys/class/drm/card*/device/vendor`, `lspci` for GPU
- **macOS**: `sysctl hw`, `system_profiler SPDisplaysDataType`, `sysctl hw.memsize`
- **Windows**: WMI (`Win32_Processor`, `Win32_VideoController`, `Win32_ComputerSystem`)
- **Display server detection**: `WAYLAND_DISPLAY` / `XDG_SESSION_TYPE` env vars, or just check running processes

### 3.9 Dependencies

- System 3 (GPU Optimization) for GPU detection — shares detection code
- System 4 (Display Server) for display server detection — shares detection code
- Independent of all other systems

---

## 4. System 2: Memory Management & Leak Prevention

### 4.1 Purpose

Prevent the app and managed processes from consuming unbounded memory. Provide per-project limits, global caps, leak detection, and a real-time memory dashboard.

### 4.2 Data Structures

```
MemoryConfig:
  global_memory_cap_mb: u64              // max total for app + all processes
  monitor_interval_ms: u64               // how often to sample (default 2000)
  leak_detection_window_minutes: u64     // monotonically increasing threshold
  leak_growth_threshold_percent: f64     // % growth over window to trigger alert
  oom_headroom_mb: u64                   // free memory required before starting new process
  enabled: bool

ProjectMemoryConfig:
  project_id: String
  soft_limit_mb: u64                     // warn when exceeded
  hard_limit_mb: u64                     // auto-restart or stop
  on_hard_limit_exceeded: HardLimitAction  // Stop | Restart | Notify
  enabled: bool

MemorySample:
  timestamp: DateTime
  app_rss_mb: f64
  process_rss_mb: HashMap<String, f64>   // project_id -> RSS
  total_rss_mb: f64
  system_total_mb: u64
  system_available_mb: u64

MemoryAlert:
  alert_type: MemoryAlertType            // SoftLimit | HardLimit | LeakDetected | OOMRisk
  project_id: Option<String>
  current_mb: f64
  limit_mb: f64
  message: String
  timestamp: DateTime
  acknowledged: bool
```

### 4.3 Platform-Specific Memory Sampling

| Platform | Method | Details |
|----------|--------|---------|
| Linux | `/proc/[pid]/status` | VmRSS field, read every interval |
| macOS | `task_info()` | `resident_size` via `mach_task_self()` |
| Windows | `GetProcessMemoryInfo()` | `WorkingSetSize` from `psapi.dll` |

For the app's own memory: same APIs applied to `std::process::id()`.

### 4.4 Leak Detection Algorithm

1. Sample RSS every `monitor_interval_ms`
2. Keep a ring buffer of samples for the `leak_detection_window_minutes` window
3. Fit a linear regression to the samples
4. If slope > `leak_growth_threshold_percent` per hour AND the trend is strictly non-decreasing for >80% of intervals, flag as potential leak
5. Emit a `MemoryAlert { type: LeakDetected }` — never auto-kill (false positive risk is high)
6. Reset detection after alert is acknowledged

### 4.5 OOM Prevention

Before starting any new process:
1. Check `system_available_mb` against `oom_headroom_mb`
2. If insufficient, block start and emit `MemoryAlert { type: OOMRisk }`
3. Allow user to override (force start anyway)

### 4.6 Storage Schema

New tables:
```
memory_config (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
)

project_memory_config (
  project_id             TEXT PRIMARY KEY,
  soft_limit_mb          INTEGER NOT NULL DEFAULT 512,
  hard_limit_mb          INTEGER NOT NULL DEFAULT 1024,
  on_hard_limit_exceeded TEXT NOT NULL DEFAULT 'stop',
  enabled                INTEGER NOT NULL DEFAULT 1
)

memory_samples (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp   TEXT NOT NULL,
  app_rss_mb  REAL NOT NULL,
  total_rss_mb REAL NOT NULL,
  system_total_mb    INTEGER NOT NULL,
  system_available_mb INTEGER NOT NULL,
  details_json TEXT  -- per-process breakdown
)

memory_alerts (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  alert_type  TEXT NOT NULL,
  project_id  TEXT,
  current_mb  REAL,
  limit_mb    REAL,
  message     TEXT NOT NULL,
  timestamp   TEXT NOT NULL,
  acknowledged INTEGER NOT NULL DEFAULT 0
)
```

### 4.7 IPC Commands

| Command | Description |
|---------|-------------|
| `get_memory_config` | Get global memory settings |
| `set_memory_config` | Update global memory settings |
| `get_project_memory_config` | Get per-project memory limits |
| `set_project_memory_config` | Set per-project memory limits |
| `get_memory_samples` | Get recent memory samples for graphing |
| `get_memory_alerts` | Get active/recent memory alerts |
| `acknowledge_memory_alert` | Mark alert as read |
| `get_process_memory_usage` | Instant snapshot of all process RSS values |

### 4.8 Frontend Components

- `MemorySettings.jsx` — Global config + per-project defaults
- `MemoryDashboard.jsx` — Real-time graph of app + process memory, history
- `ProjectMemoryConfig.jsx` — Per-project memory limits in project settings
- `MemoryAlerts.jsx` — Alert list with acknowledge/dismiss

### 4.9 Dependencies

- System 1 (Performance Engine) — provides `max_concurrent_processes` and `auto_refresh_interval` defaults
- System 8 (Per-Project Config) — memory limits are part of per-project settings

---

## 5. System 3: GPU Optimization

### 5.1 Purpose

Detect the GPU at startup, classify it by vendor/generation/capabilities, and feed this information to the Performance Engine and WebView configuration for optimal rendering.

### 5.2 GPU Detection

| Platform | Method | Fallback |
|----------|--------|----------|
| Linux | `/sys/class/drm/card*/device/vendor` + `/sys/class/drm/card*/device/device`, `glxinfo` | `lspci \| grep VGA` |
| macOS | `system_profiler SPDisplaysDataType` | `system_profiler SPDisplaysDataType -json` |
| Windows | `Win32_VideoController` via WMI | DXDiag registry keys |

### 5.3 Vendor Profiles

```
GpuVendor: enum { None, Intel, NVIDIA, AMD }
GpuGeneration: enum { Unknown, VeryOld, Old, Current, New, Latest }

GpuProfile:
  vendor: GpuVendor
  generation: GpuGeneration
  model: String
  vram_mb: Option<u64>
  driver_version: Option<String>
  supports_hardware_decode: bool
  supports_vulkan: bool           // Linux
  supports_metal: bool            // macOS
  supports_directx: u32           // Windows (11, 12)
  recommended: RenderProfile

RenderProfile:
  use_hardware_acceleration: bool
  use_compositing: bool
  use_layer_acceleration: bool
  blur_capable: bool
  shadow_quality_max: ShadowQuality
  texture_quality: TextureQuality  // Full, Reduced, Minimal
  animation_framerate_cap: u32     // 0 = unlimited, 30, 60
  force_software_fallback: bool
```

### 5.4 Vendor-Specific Rules

| Vendor + Generation | HW Accel | Blur | Shadows | Textures | Animations |
|---------------------|----------|------|---------|----------|------------|
| NVIDIA Latest (50xx) | Yes | Yes | Gaussian | Full | 60fps |
| NVIDIA New (40xx/30xx) | Yes | Yes | Blur | Full | 60fps |
| NVIDIA Old (Pascal 10xx) | Yes | Reduced | Blur | Full | 60fps |
| NVIDIA Very Old (Kepler) | Partial | No | Simple | Reduced | 30fps |
| AMD Latest (RDNA4) | Yes | Yes | Gaussian | Full | 60fps |
| AMD New (RDNA2/3) | Yes | Yes | Blur | Full | 60fps |
| AMD Old (RDNA1/GCN) | Yes | No | Simple | Reduced | 30fps |
| AMD Very Old (ATI era) | No | No | None | Minimal | Off |
| Intel Arc | Yes | Yes | Blur | Full | 60fps |
| Intel Recent (UHD 6xx+) | Partial | No | Simple | Reduced | 30fps |
| Intel Old (HD 4xxx) | No | No | None | Minimal | Off |
| None (headless/VM) | No | No | None | Minimal | Off |

### 5.5 WebView Optimization

Tauri v2 uses platform-native WebViews:
- **Windows**: WebView2 (Chromium-based) — already has good HW accel
- **macOS**: WKWebView — Metal compositing
- **Linux**: webkit2gtk — varies by GPU driver

Tuning via WebView config and CSS hints:
- `-webkit-transform: translateZ(0)` to force GPU compositing when capable
- `will-change: transform` for animated elements
- Disable backdrop-filter (blur) via CSS when GPU profile says no
- `prefers-reduced-motion` media query integration

### 5.6 GPU Blacklist

Maintain a list of known buggy driver versions that should force software rendering:
- NVIDIA 470.xx on Wayland (known compositing bug)
- Mesa 22.x on某些 Intel iGPU (blur corruption)
- WebView2 runtime < 100 on Windows (missing GPU features)

### 5.7 Storage Schema

New tables:
```
gpu_config (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
)

gpu_blacklist (
  vendor      TEXT NOT NULL,
  model_pattern TEXT NOT NULL,
  driver_pattern TEXT NOT NULL,
  reason      TEXT NOT NULL
)
```

### 5.8 IPC Commands

| Command | Description |
|---------|-------------|
| `detect_gpu` | Run GPU detection, return GpuProfile |
| `get_gpu_profile` | Get cached GPU profile from startup |
| `get_gpu_config` | Get GPU-specific overrides |
| `set_gpu_config` | Override GPU settings |
| `add_gpu_blacklist_entry` | Add driver to blacklist |
| `remove_gpu_blacklist_entry` | Remove driver from blacklist |

### 5.9 Frontend Components

- `GPUInfo.jsx` — Display detected GPU, vendor profile, current render settings
- Integrated into `PerformanceSettings.jsx` — shows GPU-aware recommendations

### 5.10 Dependencies

- System 1 (Performance Engine) — consumes GPU profile for auto-classification
- System 4 (Display Server) — Wayland restrictions affect GPU rendering choices

---

## 6. System 4: Display Server Support (Wayland / X11 / Windows / macOS)

### 6.1 Purpose

Detect the display server and adapt behavior accordingly, especially for Linux where Wayland and X11 have fundamentally different capabilities.

### 6.2 Detection

| Display Server | Detection Method |
|----------------|------------------|
| Wayland | `WAYLAND_DISPLAY` env var set, or `XDG_SESSION_TYPE=wayland` |
| X11 | `DISPLAY` env var set AND `XDG_SESSION_TYPE=x11` |
| Windows | Always DWM (compositor built into Explorer) |
| macOS | Always Quartz Compositor |

### 6.3 Feature Matrix

| Feature | Wayland | X11 | Windows 11 | macOS |
|---------|---------|-----|------------|-------|
| Always-on-top | No (compositor restriction) | Yes | Yes | Yes (NSWindow level) |
| System tray | StatusNotifierItem (SNI) | systemtray protocol | Shell_NotifyIcon | NSStatusItem |
| File picker | xdg-desktop-portal | GTK dialog | COM IOpenFileDialog | NSOpenPanel |
| Screen capture | xdg-desktop-portal | XComposite | BitBlt / DXGI | ScreenCaptureKit |
| Fractional scaling | Native (Wayland) | xrdb / xrandr | DWM scaling | Retina native |
| Transparency | Compositor-dependent | Compositor-dependent | DWM / Mica | NSVisualEffectView |
| Input methods | ibus / fcitx5 portal | XIM / ibus / fcitx | TSF | native |
| Notifications | D-Bus + portal | D-Bus | PowerShell toast | NSUserNotification |
| Window decorations | Server-side (CSD) | Client or server | DWM | native |

### 6.4 Wayland-Specific Adaptations

Since always-on-top is not available on Wayland:
- Use `xdg-toplevel` state requests (tiled, maximized) instead
- For "keep visible" use case, provide a notification instead
- Document the limitation in settings UI

For tray icons on Wayland:
- Use StatusNotifierItem (SNI) protocol via `libappindicator` or D-Bus
- Fallback: embed tray info in main window title bar

### 6.5 X11-Specific Features

- Compositor hints: `_MOTIF_WM_HINTS` for disable animations
- `_NET_WM_STATE_ABOVE` for always-on-top
- XComposite for offscreen rendering
- XRandR for multi-monitor aware positioning

### 6.6 Windows 11-Specific

- DWM: `DwmExtendFrameIntoClientArea` for Mica/Acrylic
- DWM: `DwmSetWindowAttribute` for dark mode title bar
- Snap layouts: handle `WM_SIZE` with `SIZE_MAXIMIZED`
- Taskbar: `ITaskbarList3` for progress indicator
- Power throttling: `SetProcessInformation` with `ProcessPowerThrottling`

### 6.7 macOS-Specific

- Native title bar: `NSWindow.titleVisibility` and `NSWindow.titlebarAppearsTransparent`
- Spotlight: `NSUserActivity` for indexing
- Notification Center: `UNUserNotificationCenter`
- Touch Bar: `NSTouchBar` (if applicable, low priority)
- Menu bar: `NSMenu` for app menu

### 6.8 Storage Schema

```
display_config (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
)
```

Keys: `detection_method`, `force_x11_fallback`, `wayland_tray_protocol`, `transparency_enabled`, `always_on_top_fallback`

### 6.9 IPC Commands

| Command | Description |
|---------|-------------|
| `detect_display_server` | Return current display server info |
| `get_display_config` | Get display-specific settings |
| `set_display_config` | Override display settings |
| `get_display_capabilities` | Return feature availability for current display server |

### 6.10 Frontend Components

- `DisplayInfo.jsx` — Shows detected display server, available features, limitations
- Integrated into `PerformanceSettings.jsx`

### 6.11 Dependencies

- System 1 (Performance Engine) — classification uses display server
- System 3 (GPU) — display server affects GPU rendering path

---

## 7. System 5: UI Customization Engine

### 7.1 Purpose

Let users fully customize the look, feel, and behavior of the UI, from color themes to animation granularity to feature toggles.

### 7.2 Theme Engine

```
Theme:
  name: String
  variant: ThemeVariant          // Light | Dark | System | Custom
  colors: ThemeColors
  imported_at: Option<DateTime>

ThemeColors:
  primary: HSLColor
  secondary: HSLColor
  accent: HSLColor
  background: HSLColor
  surface: HSLColor
  error: HSLColor
  warning: HSLColor
  success: HSLColor
  info: HSLColor
  muted: HSLColor
  border: HSLColor
  ring: HSLColor
  foreground: HSLColor
  destructive: HSLColor

HSLColor:
  h: u16      // 0-360
  s: u8       // 0-100
  l: u8       // 0-100
```

Themes are stored as JSON files and can be imported/exported. Pre-built themes:
- Default Dark, Default Light
- High Contrast Dark, High Contrast Light
- Solarized Dark, Solarized Light
- Nord, Dracula, Catppuccin, Tokyo Night

### 7.3 Animation Control

```
AnimationConfig:
  page_transitions: AnimationSetting     // on/off + speed (ms)
  hover_effects: HoverEffect             // off | scale | glow | color_shift
  loading_spinners: AnimationSetting
  toast_notifications: ToastAnimation     // on/off + position (4 corners) + animation (slide/fade/bounce)
  modal_open_close: ModalAnimation        // off | fade | slide_up | scale
  sidebar_expand_collapse: AnimationSetting
  project_list_reorder: AnimationSetting
  log_scrolling: LogScrollingMode         // Smooth | Instant
  process_status_indicator: StatusIndicator // Pulse | Static | Off
  tray_icon_animation: AnimationSetting

AnimationSetting:
  enabled: bool
  speed_ms: u32                     // transition duration

ToastAnimation:
  enabled: bool
  position: ToastPosition           // TopLeft | TopRight | BottomLeft | BottomRight
  animation: ToastTransition        // Slide | Fade | Bounce
```

### 7.4 Layout Customization

```
LayoutConfig:
  sidebar_width_px: u32             // min 180, max 400, default 250
  panel_positions: HashMap<PanelId, PanelPosition>  // top/bottom/left/right
  project_view_mode: ViewMode       // Grid | List
  density: Density                  // Compact | Comfortable | Spacious
  font_size_px: f64                 // 12.0 - 20.0, 0 = system default
  font_family: FontFamily           // System | Monospace | Custom(String)

PanelId: enum { Sidebar, ProjectList, Editor, Terminal, Logs, Git, Search, Tunnel }
PanelPosition: enum { Top, Bottom, Left, Right }
ViewMode: enum { Grid, List }
Density: enum { Compact, Comfortable, Spacious }
FontFamily: enum { System, Monospace, Custom(String) }
```

### 7.5 Feature Toggles

```
FeatureToggles:
  git_integration: bool             // default: true
  search: bool                      // default: true
  tunnel_management: bool           // default: true
  runtime_management: bool          // default: true
  backup_restore: bool              // default: true
  tray_icon: bool                   // default: true
  file_watcher: bool                // default: true
  lsp_bridge: bool                  // default: true
  auto_updates: bool                // default: true
  performance_monitoring: bool      // default: true (new)
  memory_monitoring: bool           // default: true (new)
  network_monitoring: bool          // default: true (new)
  notifications: bool               // default: true (new)
```

When a toggle is off:
- The corresponding service is not initialized at startup (saves memory)
- UI tabs/panels are hidden
- IPC commands for that feature return a clear "disabled" message

### 7.6 Storage Schema

```
themes (
  id        INTEGER PRIMARY KEY AUTOINCREMENT,
  name      TEXT NOT NULL UNIQUE,
  variant   TEXT NOT NULL,          -- light/dark/system/custom
  colors    TEXT NOT NULL,          -- JSON blob
  is_default INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)

ui_config (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
)
```

UI config keys: `active_theme_id`, `animation_*`, `layout_*`, `density`, `font_size`, `font_family`, `feature_toggle_*`

### 7.7 IPC Commands

| Command | Description |
|---------|-------------|
| `get_themes` | List all available themes |
| `get_theme` | Get theme by ID |
| `create_theme` | Create a new custom theme |
| `update_theme` | Update existing theme |
| `delete_theme` | Delete custom theme |
| `set_active_theme` | Apply a theme |
| `import_theme` | Import from JSON file |
| `export_theme` | Export to JSON file |
| `get_animation_config` | Get animation settings |
| `set_animation_config` | Update animation settings |
| `get_layout_config` | Get layout settings |
| `set_layout_config` | Update layout settings |
| `get_feature_toggles` | Get all toggle states |
| `set_feature_toggle` | Toggle a specific feature |
| `get_font_config` | Get font settings |
| `set_font_config` | Update font settings |

### 7.8 Frontend Components

- `ThemeProvider.jsx` — Context provider that applies CSS variables from active theme
- `ColorPicker.jsx` — HSL color picker for custom themes
- `ThemeImportExport.jsx` — File import/export UI
- `ThemePreview.jsx` — Live preview of theme changes
- `AnimationSettings.jsx` — Per-element animation toggles and speed controls
- `LayoutSettings.jsx` — Sidebar width, density, font, view mode
- `FeatureToggles.jsx` — Toggle switches for each feature
- `Settings/ThemeSection.jsx` — Complete theme management page
- `Settings/AccessibilitySection.jsx` — Reduced motion, high contrast

### 7.9 Dependencies

- System 1 (Performance Engine) — provides defaults for animation/blur/shadow based on tier
- System 13 (Accessibility) — reduced motion overrides animations, high contrast overrides colors

---

## 8. System 6: Network Management

### 8.1 Purpose

Monitor and control network usage per-process and globally. Provide bandwidth limits, connection tracking, and alerts.

### 8.2 Data Structures

```
NetworkConfig:
  global_bandwidth_limit_up_kbps: Option<u64>     // None = unlimited
  global_bandwidth_limit_down_kbps: Option<u64>
  per_interface_limits: HashMap<String, InterfaceLimit>
  monitor_interval_ms: u64                         // default 2000
  enabled: bool

InterfaceLimit:
  interface_name: String                           // eth0, wlan0
  upload_limit_kbps: Option<u64>
  download_limit_kbps: Option<u64>

ProjectNetworkConfig:
  project_id: String
  upload_limit_kbps: Option<u64>
  download_limit_kbps: Option<u64>
  total_transfer_cap_gb: Option<f64>
  transfer_cap_reset: TransferCapReset             // Daily | Weekly | Monthly | Never
  enabled: bool

TransferCapReset: enum { Daily, Weekly, Monthly, Never }

ProcessNetworkSample:
  project_id: String
  pid: u32
  timestamp: DateTime
  rx_bytes_per_sec: u64
  tx_bytes_per_sec: u64
  connection_count: u32
  dns_queries: u32

NetworkAlert:
  alert_type: NetworkAlertType
  project_id: Option<String>
  message: String
  timestamp: DateTime
  acknowledged: bool

NetworkAlertType: enum {
  BandwidthExceeded,
  UnexpectedConnection,
  HighLatency,
  DataCapApproaching,
  ConnectionFailure,
}

ConnectionInfo:
  local_addr: String
  remote_addr: String
  state: String               // ESTABLISHED, LISTEN, etc.
  pid: Option<u32>
  process_name: Option<String>
  rx_bytes: u64
  tx_bytes: u64
```

### 8.3 Platform-Specific Network Monitoring

| Platform | Bandwidth | Connections | Rate Limiting |
|----------|-----------|-------------|---------------|
| Linux | `/proc/net/dev` (bytes in/out per interface) | `ss -tunap` | `tc` (traffic control) or nftables |
| macOS | `netstat -ib` or `ifstat` | `lsof -i -P -n` | `dnctl` (dummynet) or `pf` |
| Windows | `Get-NetAdapterStatistics` | `Get-NetTCPConnection` + `Get-Process` | `tc` (Windows) or WFP callout driver |

For per-process bandwidth:
- **Linux**: Read `/proc/[pid]/net/dev` (per-namespace network stats)
- **macOS**: `lsof -i -a -p [pid]` + `netstat` delta
- **Windows**: `Get-NetTCPConnection` + owner PID mapping

### 8.4 Network Quality Metrics

- **Latency**: Periodic ICMP ping to configurable hosts (default: 8.8.8.8, 1.1.1.1)
- **Packet loss**: Count ping failures over window
- **DNS resolution time**: Time `dig` or `nslookup` takes

### 8.5 Enforcement

Bandwidth limiting via platform-native tools:
- **Linux**: `tc qdisc` + `tbf` (token bucket filter) per interface
- **macOS**: `dnctl` (dummynet) pipes
- **Windows**: `tc` command or QoS policies via PowerShell

Note: Enforcement is advisory and best-effort. The app cannot guarantee hard limits without kernel-level integration.

### 8.6 Storage Schema

```
network_config (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
)

project_network_config (
  project_id          TEXT PRIMARY KEY,
  upload_limit_kbps   INTEGER,
  download_limit_kbps INTEGER,
  total_transfer_cap_gb REAL,
  transfer_cap_reset  TEXT NOT NULL DEFAULT 'never',
  enabled             INTEGER NOT NULL DEFAULT 1,
  transfer_used_bytes INTEGER NOT NULL DEFAULT 0,
  transfer_reset_at   TEXT
)

network_samples (
  id                INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp         TEXT NOT NULL,
  project_id        TEXT,
  pid               INTEGER,
  rx_bytes_per_sec  INTEGER NOT NULL,
  tx_bytes_per_sec  INTEGER NOT NULL,
  connection_count  INTEGER NOT NULL DEFAULT 0
)

network_alerts (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  alert_type  TEXT NOT NULL,
  project_id  TEXT,
  message     TEXT NOT NULL,
  timestamp   TEXT NOT NULL,
  acknowledged INTEGER NOT NULL DEFAULT 0
)
```

### 8.7 IPC Commands

| Command | Description |
|---------|-------------|
| `get_network_config` | Get global network settings |
| `set_network_config` | Update global network settings |
| `get_project_network_config` | Get per-project network limits |
| `set_project_network_config` | Set per-project network limits |
| `get_network_samples` | Get recent bandwidth samples |
| `get_network_connections` | List active connections |
| `get_network_alerts` | Get network alerts |
| `acknowledge_network_alert` | Dismiss alert |
| `get_network_quality` | Get latency/packet loss metrics |
| `set_interface_limit` | Set per-interface bandwidth limit |

### 8.8 Frontend Components

- `NetworkSettings.jsx` — Global and per-project network config
- `NetworkDashboard.jsx` — Real-time bandwidth graph, connection table
- `NetworkAlerts.jsx` — Alert list
- `ConnectionTable.jsx` — Sortable table of active connections

### 8.9 Dependencies

- System 1 (Performance Engine) — provides `auto_refresh_interval` for monitoring
- System 7 (Notification System) — network alerts trigger notifications
- System 8 (Per-Project Config) — network limits are part of per-project settings

---

## 9. System 7: Notification System

### 9.1 Purpose

Universal notification engine that delivers alerts through any channel: in-app toasts, email, webhooks, WebSocket, desktop notifications, and third-party integrations (Telegram, Discord, Slack).

### 9.2 Notification Channels

```
NotificationChannel: enum {
  InApp,
  Email(EmailConfig),
  Webhook(WebhookConfig),
  WebSocket(WebSocketConfig),
  Desktop,
  Telegram(TelegramConfig),
  Discord(DiscordConfig),
  Slack(SlackConfig),
}

EmailConfig:
  provider: EmailProvider          // Gmail | Outlook | Apple | CustomSMTP
  smtp_host: String
  smtp_port: u16
  username: String
  password: String                 // encrypted at rest
  use_oauth2: bool
  oauth2_token: Option<String>    // encrypted
  from_address: String
  to_addresses: Vec<String>
  use_tls: bool

EmailProvider: enum { Gmail, Outlook, Apple, CustomSMTP }

WebhookConfig:
  url: String
  method: String                   // POST, PUT
  headers: HashMap<String, String>
  hmac_secret: Option<String>      // for signing
  retry_attempts: u32              // default 3
  retry_backoff_ms: u64            // exponential

WebSocketConfig:
  url: String
  auth_token: Option<String>
  reconnect: bool
  reconnect_interval_ms: u64

TelegramConfig:
  bot_token: String
  chat_id: String

DiscordConfig:
  webhook_url: String

SlackConfig:
  webhook_url: String
  channel: Option<String>
```

### 9.3 Rules Engine

```
NotificationRule:
  id: String
  name: String
  enabled: bool
  priority: u32                    // higher = processed first
  event_filter: EventFilter
  conditions: Vec<Condition>
  actions: Vec<NotificationAction>
  rate_limit: RateLimitConfig
  quiet_hours: Option<QuietHours>

EventFilter:
  event_types: Vec<EventType>      // empty = all events
  project_filter: Option<String>   // regex pattern

EventType: enum {
  ProcessStarted,
  ProcessStopped,
  ProcessCrashed,
  MemoryAlert,
  MemoryLeakDetected,
  TunnelConnected,
  TunnelDisconnected,
  GitPush,
  GitCommit,
  BackupComplete,
  BackupFailed,
  UpdateAvailable,
  ErrorOccurred,
  NetworkAlert,
  DiskSpaceLow,
}

Condition:
  field: String                    // e.g., "severity", "project_name"
  operator: ConditionOperator      // eq, neq, gt, lt, contains, matches
  value: String

ConditionOperator: enum { Eq, Neq, Gt, Lt, Contains, Matches }

NotificationAction:
  channel_id: String
  template_id: Option<String>
  custom_message: Option<String>

RateLimitConfig:
  max_per_minute: Option<u32>
  max_per_hour: Option<u32>
  max_per_day: Option<u32>

QuietHours:
  start_hour: u8                   // 0-23
  start_minute: u8
  end_hour: u8
  end_minute: u8
  timezone: String                 // IANA timezone
```

### 9.4 Templates

```
NotificationTemplate:
  id: String
  name: String
  channel_type: String              // email, webhook, in_app, etc.
  subject_template: Option<String>  // for email
  body_template: String             // supports {project}, {event}, {time}, {details}, {severity}
  preview: String                   // rendered preview

TemplateVariables:
  {project}     -> project name
  {event}       -> event type name
  {time}        -> formatted timestamp
  {details}     -> event details JSON
  {severity}    -> alert severity
  {app_version} -> current app version
```

### 9.5 Notification History

```
NotificationLog:
  id: INTEGER PRIMARY KEY AUTOINCREMENT
  rule_id: String
  channel: String                   // email, webhook, etc.
  event_type: String
  project_id: Option<String>
  subject: Option<String>
  body: String
  status: NotificationStatus        // Sent | Failed | RateLimited | QuietHours
  error_message: Option<String>
  timestamp: DateTime
  retry_count: u32

NotificationStatus: enum { Sent, Failed, RateLimited, QuietHours, Pending }
```

### 9.6 Storage Schema

```
notification_channels (
  id        TEXT PRIMARY KEY,
  name      TEXT NOT NULL,
  type      TEXT NOT NULL,          -- email, webhook, websocket, desktop, telegram, discord, slack
  config    TEXT NOT NULL,          -- JSON blob (encrypted for sensitive fields)
  enabled   INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL
)

notification_rules (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  enabled     INTEGER NOT NULL DEFAULT 1,
  priority    INTEGER NOT NULL DEFAULT 0,
  event_filter TEXT NOT NULL,       -- JSON
  conditions  TEXT NOT NULL,        -- JSON array
  actions     TEXT NOT NULL,        -- JSON array
  rate_limit  TEXT NOT NULL,        -- JSON
  quiet_hours TEXT,                 -- JSON or null
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL
)

notification_templates (
  id              TEXT PRIMARY KEY,
  name            TEXT NOT NULL,
  channel_type    TEXT NOT NULL,
  subject_template TEXT,
  body_template   TEXT NOT NULL,
  created_at      TEXT NOT NULL
)

notification_log (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  rule_id     TEXT NOT NULL,
  channel     TEXT NOT NULL,
  event_type  TEXT NOT NULL,
  project_id  TEXT,
  subject     TEXT,
  body        TEXT NOT NULL,
  status      TEXT NOT NULL,
  error_message TEXT,
  timestamp   TEXT NOT NULL,
  retry_count INTEGER NOT NULL DEFAULT 0
)
```

### 9.7 IPC Commands

| Command | Description |
|---------|-------------|
| `get_notification_channels` | List all configured channels |
| `create_notification_channel` | Add a new channel |
| `update_notification_channel` | Update channel config |
| `delete_notification_channel` | Remove channel |
| `test_notification_channel` | Send test notification to channel |
| `get_notification_rules` | List all rules |
| `create_notification_rule` | Add a new rule |
| `update_notification_rule` | Update rule |
| `delete_notification_rule` | Remove rule |
| `get_notification_templates` | List templates |
| `create_notification_template` | Add template |
| `update_notification_template` | Update template |
| `delete_notification_template` | Remove template |
| `get_notification_log` | Get notification history (paginated) |
| `retry_notification` | Retry a failed notification |
| `get_notification_stats` | Get stats (sent/failed per channel) |

### 9.8 Frontend Components

- `NotificationSettings.jsx` — Main notification config page
- `ChannelConfig.jsx` — Per-channel configuration (SMTP wizard, webhook test, etc.)
- `RulesEditor.jsx` — Visual rule builder with drag-and-drop
- `TemplateEditor.jsx` — Template editor with variable autocomplete and preview
- `NotificationHistory.jsx` — Filterable log table
- `NotificationTest.jsx` — Send test notification UI
- `QuietHoursConfig.jsx` — Quiet hours time range picker

### 9.9 Dependencies

- System 8 (Per-Project Config) — per-project notification overrides
- System 9 (Dashboard) — alert panel triggers notifications
- System 2 (Memory) — memory alerts trigger notifications
- System 6 (Network) — network alerts trigger notifications

---

## 10. System 8: Per-Project Deep Configuration

### 10.1 Purpose

Extend the current minimal `ProjectSettings` (runtime, node_version, python_version) with comprehensive per-project configuration covering commands, resources, monitoring, git automation, and notification overrides.

### 10.2 Data Structures

```
ProjectConfig:
  // Identity
  project_id: String
  name: String

  // Command Configuration
  start_command: Option<String>
  stop_command: Option<String>       // default: SIGTERM / taskkill
  restart_command: Option<String>    // default: stop + start
  working_directory: Option<String>  // default: project path
  shell: ShellType                   // Bash | Zsh | Sh | PowerShell | Cmd
  env_file: Option<String>           // path to .env file
  env_vars: HashMap<String, String>  // additional env vars

  // Resource Limits
  memory: Option<ProjectMemoryConfig>
  cpu_nice_value: Option<i8>         // Linux: -20..19, Windows/macOS: priority class
  disk_io_limit_bytes_per_sec: Option<u64>
  network: Option<ProjectNetworkConfig>

  // Monitoring
  health_check_url: Option<String>
  health_check_interval_secs: Option<u64>
  process_alive_check_interval_secs: Option<u64>  // default: 5
  auto_restart_on_crash: bool
  auto_restart_max_retries: u32      // default: 3
  auto_restart_delay_ms: u64         // default: 5000
  log_rotation_max_size_mb: u64      // default: 10
  log_rotation_max_files: u32        // default: 5

  // Git Automation
  git_auto_commit_enabled: bool
  git_auto_commit_interval_secs: Option<u64>
  git_auto_push_after_commit: bool
  git_watch_branch: Option<String>   // branch to watch for auto-deploy

  // Notifications
  notification_overrides_enabled: bool
  custom_webhook_url: Option<String>
  notification_recipients: Vec<String>  // email addresses

  // Editor
  default_editor: EditorType          // Monaco | External(String)
  editor_tab_size: Option<u32>
  editor_word_wrap: Option<bool>
  editor_theme: Option<String>

ShellType: enum { Bash, Zsh, Sh, PowerShell, Cmd }
EditorType: enum { Monaco, External(String) }
```

### 10.3 Storage Schema

Expand `project_settings` table:
```
project_settings (
  project_id                    TEXT PRIMARY KEY,
  -- existing columns (keep for backward compat)
  runtime                       TEXT,
  node_version                  TEXT,
  python_version                TEXT,
  install_date                  TEXT,
  last_used                     TEXT,
  -- new columns
  start_command                 TEXT,
  stop_command                  TEXT,
  restart_command               TEXT,
  working_directory             TEXT,
  shell                         TEXT NOT NULL DEFAULT 'bash',
  env_file                      TEXT,
  env_vars_json                 TEXT,          -- JSON object
  memory_soft_limit_mb          INTEGER,
  memory_hard_limit_mb          INTEGER,
  on_hard_limit_exceeded        TEXT NOT NULL DEFAULT 'stop',
  cpu_nice_value                INTEGER,
  disk_io_limit_bytes_per_sec   INTEGER,
  network_upload_limit_kbps     INTEGER,
  network_download_limit_kbps   INTEGER,
  network_transfer_cap_gb       REAL,
  network_transfer_cap_reset    TEXT NOT NULL DEFAULT 'never',
  health_check_url              TEXT,
  health_check_interval_secs    INTEGER,
  process_alive_check_interval_secs INTEGER NOT NULL DEFAULT 5,
  auto_restart_on_crash         INTEGER NOT NULL DEFAULT 0,
  auto_restart_max_retries      INTEGER NOT NULL DEFAULT 3,
  auto_restart_delay_ms         INTEGER NOT NULL DEFAULT 5000,
  log_rotation_max_size_mb      INTEGER NOT NULL DEFAULT 10,
  log_rotation_max_files        INTEGER NOT NULL DEFAULT 5,
  git_auto_commit_enabled       INTEGER NOT NULL DEFAULT 0,
  git_auto_commit_interval_secs INTEGER,
  git_auto_push_after_commit    INTEGER NOT NULL DEFAULT 0,
  git_watch_branch              TEXT,
  notification_overrides_enabled INTEGER NOT NULL DEFAULT 0,
  custom_webhook_url            TEXT,
  notification_recipients_json  TEXT,          -- JSON array
  default_editor                TEXT NOT NULL DEFAULT 'monaco',
  editor_tab_size               INTEGER,
  editor_word_wrap              INTEGER,
  editor_theme                  TEXT
)
```

### 10.4 IPC Commands

| Command | Description |
|---------|-------------|
| `get_project_config` | Get full project config |
| `set_project_config` | Update full project config |
| `update_project_config` | Partial update of project config |
| `reset_project_config` | Reset to defaults |
| `import_project_env` | Import .env file into project config |
| `get_project_env_vars` | Get resolved env vars |

### 10.5 Frontend Components

- `ProjectConfigDialog.jsx` — Multi-tab dialog (Commands, Resources, Monitoring, Git, Notifications, Editor)
- `CommandConfig.jsx` — Start/stop/restart command configuration
- `ResourceLimits.jsx` — Memory, CPU, disk I/O, network limits
- `MonitoringConfig.jsx` — Health check, auto-restart, log rotation
- `GitAutomation.jsx` — Auto-commit, auto-push, branch watch
- `ProjectNotifications.jsx` — Per-project notification overrides
- `ProjectEditorConfig.jsx` — Editor preferences per project

### 10.6 Dependencies

- System 2 (Memory) — memory limits are consumed here
- System 6 (Network) — network limits are consumed here
- System 7 (Notification) — notification overrides reference channels/rules
- Existing system: `settings_service.rs` — extends the existing ProjectSettings

---

## 11. System 9: Performance Monitoring Dashboard

### 11.1 Purpose

Real-time system monitoring dashboard with draggable panels, configurable refresh rates, historical data, and alert management.

### 11.2 Dashboard Panels

```
DashboardPanel: enum {
  SystemOverview,       // CPU, memory, disk, network summary
  ProcessPanel,         // Per-process CPU%, MEM%, PID, uptime, restarts
  NetworkPanel,         // Bandwidth graph, connection count
  MemoryPanel,          // Memory usage graph, per-process breakdown
  GPUPanel,             // GPU utilization (if available)
  AlertsPanel,          // Active alerts sorted by severity
  DiskPanel,            // Disk usage and I/O
  LogPanel,             // Recent log entries
}

DashboardLayout:
  panels: Vec<DashboardPanelConfig>
  columns: u32                // grid columns (default 4)
  row_height: u32             // default 200px

DashboardPanelConfig:
  panel_type: DashboardPanel
  position: GridPosition      // x, y, w, h in grid units
  visible: bool
  collapsed: bool

GridPosition:
  x: u32
  y: u32
  w: u32
  h: u32

DashboardConfig:
  refresh_rate_ms: u64        // 500ms to 60000ms, default 2000ms
  data_retention_hours: u32   // 1 to 720 (30 days), default 24
  layout: DashboardLayout
  export_format: ExportFormat // CSV | JSON

ExportFormat: enum { CSV, JSON }
```

### 11.3 System Metrics Collection

```
SystemMetrics:
  timestamp: DateTime
  cpu: CpuMetrics
  memory: MemoryMetrics
  disk: DiskMetrics
  network: NetworkMetrics
  gpu: Option<GpuMetrics>

CpuMetrics:
  overall_percent: f64
  per_core_percent: Vec<f64>
  load_average: Option<(f64, f64, f64)>  // 1min, 5min, 15min (Linux/macOS)

MemoryMetrics:
  total_mb: u64
  used_mb: u64
  available_mb: u64
  app_rss_mb: f64
  swap_used_mb: Option<u64>

DiskMetrics:
  total_gb: f64
  used_gb: f64
  io_read_bytes_per_sec: u64
  io_write_bytes_per_sec: u64

NetworkMetrics:
  rx_bytes_per_sec: u64
  tx_bytes_per_sec: u64
  active_connections: u32

GpuMetrics:
  utilization_percent: Option<f64>
  memory_used_mb: Option<u64>
  temperature_celsius: Option<f64>
```

### 11.4 Platform-Specific Metrics Collection

| Metric | Linux | macOS | Windows |
|--------|-------|-------|---------|
| CPU | `/proc/stat` | `host_processor_info()` | `GetSystemTimes()` |
| Memory | `/proc/meminfo` | `host_statistics64()` | `GlobalMemoryStatusEx()` |
| Disk I/O | `/proc/diskstats` | `IOKit` | `GetPerformanceInfo()` |
| Network | `/proc/net/dev` | `ifstat` | `GetNetworkLayoutParams()` |
| GPU | `nvidia-smi`, `/sys/class/drm` | `IOKit` | `WMI` |
| Load average | `/proc/loadavg` | `getloadavg()` | N/A (use Task Manager API) |

### 11.5 Alerting

```
Alert:
  id: String
  severity: AlertSeverity          // Info | Warning | Critical
  source: String                   // "memory", "network", "process", "disk"
  title: String
  message: String
  project_id: Option<String>
  timestamp: DateTime
  acknowledged: bool
  dismissed: bool

AlertRule:
  id: String
  name: String
  enabled: bool
  metric: String                   // "cpu_percent", "memory_percent", etc.
  condition: AlertCondition
  threshold: f64
  duration_secs: u64               // how long condition must persist
  severity: AlertSeverity
  actions: Vec<NotificationAction>  // reference System 7

AlertCondition: enum { GreaterThan, LessThan, Equals }
```

### 11.6 Storage Schema

```
dashboard_config (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
)

dashboard_metrics (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp   TEXT NOT NULL,
  metrics_json TEXT NOT NULL         -- serialized SystemMetrics
)

alert_rules (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  enabled     INTEGER NOT NULL DEFAULT 1,
  metric      TEXT NOT NULL,
  condition   TEXT NOT NULL,
  threshold   REAL NOT NULL,
  duration_secs INTEGER NOT NULL DEFAULT 0,
  severity    TEXT NOT NULL,
  actions     TEXT NOT NULL,         -- JSON array
  created_at  TEXT NOT NULL
)

alerts (
  id          TEXT PRIMARY KEY,
  severity    TEXT NOT NULL,
  source      TEXT NOT NULL,
  title       TEXT NOT NULL,
  message     TEXT NOT NULL,
  project_id  TEXT,
  timestamp   TEXT NOT NULL,
  acknowledged INTEGER NOT NULL DEFAULT 0,
  dismissed   INTEGER NOT NULL DEFAULT 0
)
```

### 11.7 IPC Commands

| Command | Description |
|---------|-------------|
| `get_dashboard_config` | Get dashboard layout and settings |
| `set_dashboard_config` | Update dashboard config |
| `get_system_metrics` | Get current system metrics snapshot |
| `get_metrics_history` | Get historical metrics (for graphs) |
| `get_process_metrics` | Get per-process metrics |
| `get_alert_rules` | List alert rules |
| `create_alert_rule` | Add alert rule |
| `update_alert_rule` | Update alert rule |
| `delete_alert_rule` | Remove alert rule |
| `get_active_alerts` | Get unacknowledged alerts |
| `acknowledge_alert` | Mark alert as read |
| `dismiss_alert` | Dismiss alert |
| `export_metrics` | Export metrics as CSV/JSON |
| `clear_metrics_history` | Purge old metrics data |

### 11.8 Frontend Components

- `DashboardPage.jsx` — Main dashboard view with draggable grid layout
- `SystemOverviewPanel.jsx` — CPU/memory/disk/network summary cards
- `ProcessPanel.jsx` — Sortable table with CPU%, MEM%, PID, uptime, sparklines
- `NetworkPanel.jsx` — Real-time bandwidth graph
- `MemoryPanel.jsx` — Memory usage graph with per-process breakdown
- `GPUPanel.jsx` — GPU utilization and temperature
- `AlertsPanel.jsx` — Active alerts with acknowledge/dismiss
- `DiskPanel.jsx` — Disk usage and I/O stats
- `DashboardGrid.jsx` — Grid layout manager (drag, resize, collapse)
- `MetricGraph.jsx` — Reusable graph component (5s, 30s, 5min, 1hr rolling)
- `Sparkline.jsx` — Inline mini-graph for process list

### 11.9 Dependencies

- System 1 (Performance Engine) — default refresh rate and data retention
- System 2 (Memory) — memory data feeds into memory panel
- System 3 (GPU) — GPU data feeds into GPU panel
- System 6 (Network) — network data feeds into network panel
- System 7 (Notification) — alerts trigger notifications
- System 13 (Accessibility) — screen reader labels for graphs

---

## 12. System 10: Platform-Specific Optimizations

### 12.1 Purpose

Deep OS integration for resource management, security, and native behavior on each platform.

### 12.2 Linux

| Feature | Implementation | Notes |
|---------|---------------|-------|
| cgroup v2 resource limits | Write to `/sys/fs/cgroup/` per-process | Requires root or `unshare` |
| systemd user service | `systemctl --user` for process management | Optional, toggle in settings |
| Procfs monitoring | `/proc/[pid]/status`, `/proc/[pid]/io`, `/proc/[pid]/schedstat` | Already partially used |
| nftables/iptables | `nft` command for network rate limiting | Fallback: `tc` |
| D-Bus notifications | `org.freedesktop.Notifications` via zbus crate | Replaces `notify-send` |
| XDG directories | `dirs` crate (already used) | Compliance check |
| Flatpak sandboxing | Detect `/.flatpak-info`, adjust paths | Awareness only |
| AppArmor/SELinux | Detect enforcement mode, log warnings | Informational |

**cgroup v2 Integration Detail:**

For managed processes, create a per-process cgroup under the user's session:
```
/sys/fs/cgroup/user.slice/user-<uid>.slice/app.selfhosthelper.slice/
  └── project-<id>.slice/
      ├── cpu.max          (quota/period)
      ├── memory.max       (hard limit)
      ├── memory.high      (soft limit, triggers reclaim)
      ├── io.max           (block I/O limits)
      └── pids.max         (process count limit)
```

### 12.3 macOS

| Feature | Implementation | Notes |
|---------|---------------|-------|
| Metal API | WebView2 via WKWebView uses Metal by default | Verify enablement |
| Grand Central Dispatch | `dispatch_queue` for background tasks | Use `tokio` runtime affinity |
| Instruments integration | Export `.dtrace` scripts for profiling | Debug mode only |
| Energy efficiency | `ProcessActivityState` — background when idle | Reduces fan noise |
| LaunchAgent | `~/Library/LaunchAgents/com.selfhosthelper.plist` | Auto-start option |
| Spotlight indexing | `NSUserActivity` for searchable app state | Low priority |

### 12.4 Windows 11

| Feature | Implementation | Notes |
|---------|---------------|-------|
| Process Mitigation Policies | `SetProcessMitigationPolicy()` via `windows-sys` | ACG, CIG for sandboxed processes |
| Windows Job Objects | Already implemented in `electron/job/` | Migrate to Rust `windows-sys` |
| Power Throttling | `SetProcessInformation` with `ProcessPowerThrottling` | Avoid throttling UI process |
| Performance Counters | `PdhOpenQuery` / `PdhCollectQueryData` | Richer metrics than `/proc` |
| BITS updates | Background Intelligent Transfer Service | For large updates |
| Windows Terminal | Detect and integrate with `wt.exe` | Optional external terminal |
| MSIX packaging | `msix-builder` crate or manual manifest | Alternative to NSIS |

### 12.5 Storage Schema

```
platform_config (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
)
```

Platform-specific keys are namespaced: `linux.cgroup_enabled`, `linux.systemd_service`, `macos.launch_agent`, `windows.job_objects`, etc.

### 12.6 IPC Commands

| Command | Description |
|---------|-------------|
| `get_platform_config` | Get platform-specific settings |
| `set_platform_config` | Update platform-specific settings |
| `get_platform_capabilities` | Return available platform features |
| `install_launch_agent` | macOS: install LaunchAgent |
| `remove_launch_agent` | macOS: remove LaunchAgent |
| `install_systemd_service` | Linux: install systemd user service |
| `remove_systemd_service` | Linux: remove systemd user service |
| `get_cgroup_status` | Linux: check cgroup v2 availability |

### 12.7 Frontend Components

- `PlatformSettings.jsx` — Platform-specific options (only shows relevant ones)
- `PlatformCapabilities.jsx` — Read-only display of what the platform supports

### 12.8 Dependencies

- System 1 (Performance Engine) — platform capabilities feed into classification
- System 2 (Memory) — cgroup integration for memory limits
- System 6 (Network) — nftables for network limiting
- System 11 (Security) — process mitigation policies
- System 14 (Build Pipeline) — MSIX packaging

---

## 13. System 11: Security & Hardening

### 13.1 Purpose

Defense-in-depth security beyond the existing audit. Add process sandboxing, encrypted config, audit logging, and certificate pinning.

### 13.2 Process Sandboxing

Each managed process runs in an isolated context:
- **Linux**: cgroup v2 + namespaces (PID, mount, network optionally)
- **macOS**: `sandbox_init()` with profile
- **Windows**: Restricted token + job object + ACG mitigation

Sandbox levels:
```
SandboxLevel: enum {
  None,                    // no isolation (default for dev convenience)
  Basic,                   // resource limits only
  Restricted,              // resource limits + filesystem read-only except project dir
  Strict,                  // full isolation (for untrusted code)
}
```

### 13.3 Encrypted Configuration

Beyond encrypted backups (already implemented), encrypt sensitive config values at rest:
- Email passwords
- OAuth2 tokens
- Webhook HMAC secrets
- API keys (Telegram bot tokens, etc.)

Implementation: derive key from machine-specific seed (hw UUID + app salt) using PBKDF2, encrypt with AES-256-GCM (reuse existing crypto module).

### 13.4 Audit Logging

Every significant action logged to a dedicated audit table:
```
audit_log (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp   TEXT NOT NULL,
  actor       TEXT NOT NULL,         // "user", "system", "auto-restart"
  action      TEXT NOT NULL,         // "project.start", "settings.update", etc.
  target      TEXT,                  // project_id, setting_key, etc.
  details     TEXT,                  // JSON with old/new values
  ip_address  TEXT,                  // if applicable
  session_id  TEXT
)
```

Tracked actions:
- Project start/stop/restart
- Settings changes
- Backup create/restore/delete
- Tunnel start/stop
- Git operations (commit, push, branch changes)
- Notification sent
- Alert triggered/acknowledged
- Security events (path violation, auth failure)
- Feature toggle changes

### 13.5 Certificate Pinning

For update checks (already using Tauri updater):
- Pin the GitHub Releases TLS certificate fingerprint
- On mismatch, show warning and allow manual override
- Store pinned fingerprints in config, update with app updates

### 13.6 Clipboard Security

When copying sensitive data (tokens, passwords):
- Clear clipboard after configurable timeout (default: 30s)
- Show toast: "Sensitive data cleared from clipboard"
- Track clipboard contents for clearing

### 13.7 Storage Schema

```
audit_log (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp   TEXT NOT NULL,
  actor       TEXT NOT NULL,
  action      TEXT NOT NULL,
  target      TEXT,
  details     TEXT,
  ip_address  TEXT,
  session_id  TEXT
)

security_config (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
)
```

Security config keys: `sandbox_level`, `clipboard_clear_timeout_secs`, `certificate_pins`, `encrypted_config_enabled`

### 13.8 IPC Commands

| Command | Description |
|---------|-------------|
| `get_audit_log` | Get audit log (paginated, filterable) |
| `get_security_config` | Get security settings |
| `set_security_config` | Update security settings |
| `get_certificate_pins` | Get pinned certificates |
| `add_certificate_pin` | Pin a certificate |
| `remove_certificate_pin` | Remove a pin |
| `clear_clipboard` | Manually clear clipboard |

### 13.9 Frontend Components

- `SecuritySettings.jsx` — Sandbox level, clipboard timeout, cert pins
- `AuditLog.jsx` — Filterable audit log table
- `CertificatePinning.jsx` — Manage pinned certificates

### 13.10 Dependencies

- System 10 (Platform) — sandboxing uses platform-specific primitives
- System 7 (Notification) — audit logging for notification events
- Existing: `crypto.rs` — reuses AES-256-GCM + PBKDF2

---

## 14. System 12: Build & Release Pipeline

### 14.1 Purpose

Automated CI/CD for building, testing, signing, and releasing across all platforms.

### 14.2 Build Matrix

| Platform | Arch | Runner | Output |
|----------|------|--------|--------|
| Linux | x86_64 | `ubuntu-latest` | AppImage, .deb, .rpm |
| Linux | aarch64 | `ubuntu-latest` (cross-compile) | AppImage, .deb, .rpm |
| macOS | x86_64 | `macos-13` | .dmg, .app |
| macOS | aarch64 | `macos-14` (M1 runner) | .dmg, .app |
| Windows | x86_64 | `windows-latest` | NSIS installer, MSIX |

### 14.3 Pipeline Stages

```
1. Lint & Type Check
   ├── cargo clippy (Rust)
   ├── eslint (JS/JSX)
   ├── tsc --noEmit (type check)
   └── prettier --check (formatting)

2. Build
   ├── npm run build:web (Vite)
   ├── cargo build --release (Tauri)
   └── tauri build (bundle)

3. Test
   ├── cargo test (Rust unit tests)
   └── (manual smoke test placeholder — no automated E2E yet)

4. Sign
   ├── Linux: GPG sign .deb, .rpm, AppImage
   ├── macOS: codesign + notarize (Apple Developer ID)
   └── Windows: Authenticode sign (EV certificate)

5. Publish
   ├── Create GitHub Release with tag
   ├── Upload artifacts
   ├── Generate latest.json (Tauri updater)
   └── Update release notes

6. Beta Channel (optional)
   ├── Tag with -beta suffix
   └── Publish to beta endpoint
```

### 14.4 Build Optimizations

| Optimization | Description | Impact |
|-------------|-------------|--------|
| LTO (Link-Time Optimization) | `lto = true` in Cargo.toml release profile | -20-30% binary size, +5-10% compile time |
| Strip debug symbols | `strip = true` in Cargo.toml | -40-60% binary size |
| UPX compression | Optional post-build compression | -50-70% binary size, +100ms startup |
| Delta updates | Tauri supports differential updates | -80-90% download size for updates |
| Cache cargo dependencies | `actions/cache` for `~/.cargo` | -50% build time on cache hit |
| Parallel platform builds | Matrix strategy in GitHub Actions | Build all platforms simultaneously |

### 14.5 Signing Details

**Linux GPG:**
```yaml
gpg --import --batch <<< "$GPG_PRIVATE_KEY"
gpg --batch --detach-sign --armor release_files/*
```

**macOS codesign:**
```bash
codesign --force --sign "Developer ID Application: ..." --options runtime --timestamp *.app
xcrun notarytool submit *.dmg --apple-id "..." --team-id "..." --password "..."
xcrun stapler staple *.dmg
```

**Windows Authenticode:**
```powershell
signtool sign /f certificate.pfx /p $CERT_PASSWORD /tr http://timestamp.digicert.com /td sha256 /fd sha256 *.exe
```

### 14.6 Storage Schema

Not applicable — CI/CD configuration lives in `.github/workflows/`.

### 14.7 GitHub Actions Workflow

New file: `.github/workflows/release.yml`

```yaml
name: Release
on:
  push:
    tags: ['v*']
  workflow_dispatch:
    inputs:
      channel:
        description: 'Release channel'
        required: false
        default: 'stable'
        type: choice
        options: ['stable', 'beta']
```

### 14.8 Frontend Components

- None (backend/CI only)

### 14.9 Dependencies

- System 10 (Platform) — MSIX packaging for Windows
- System 13 (Accessibility) — ensure a11y features are in the build

---

## 15. System 13: Accessibility

### 15.1 Purpose

Ensure the app is usable by everyone, including users with motor, visual, or cognitive impairments.

### 15.2 Keyboard Navigation

- Every interactive element must be reachable via Tab/Shift+Tab
- Arrow keys for list/grid navigation
- Enter/Space to activate buttons
- Escape to close modals/dialogs
- Ctrl+K or Cmd+K for command palette (quick access to any feature)
- Customizable keyboard shortcuts for all major actions

### 15.3 Screen Reader Support

- All interactive elements have `aria-label` or `aria-labelledby`
- Dynamic content changes announced via `aria-live` regions
- Status indicators have `role="status"` or `role="alert"`
- Tables have proper `role="table"`, `role="row"`, `role="cell"`
- Graphs have text alternatives (tabular data summary)
- Loading states announced

### 15.4 Visual Accessibility

**High Contrast Mode:**
- Override all colors with high-contrast palette
- Minimum 4.5:1 contrast ratio for text (WCAG AA)
- 3:1 contrast ratio for non-text elements
- Borders on all interactive elements
- No color-only indicators (always pair with icon/text)

**Reduced Motion Mode:**
- Respect `prefers-reduced-motion` system setting
- All animations disabled or replaced with instant transitions
- No auto-scrolling, no auto-playing content
- Loading spinners become static text

**Font Scaling:**
- Respect system DPI scaling
- User-configurable font size (12-20px)
- No fixed pixel sizes in layout (use rem/em)
- Minimum touch target size: 44x44px

### 15.5 Color Blind Friendly

- Avoid relying solely on red/green for status
- Use shape + color + text for all indicators
- Provide palette options: Deuteranopia, Protanopia, Tritanopia
- Test with Sim Daltonism or equivalent

### 15.6 Storage Schema

```
accessibility_config (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
)
```

Keys: `high_contrast_mode`, `reduced_motion`, `font_size`, `font_family`, `color_blind_mode`, `keyboard_shortcuts_json`, `screen_reader_announcements`

### 15.7 IPC Commands

| Command | Description |
|---------|-------------|
| `get_accessibility_config` | Get accessibility settings |
| `set_accessibility_config` | Update accessibility settings |
| `get_system_accessibility_settings` | Detect OS accessibility settings (prefers-reduced-motion, high contrast) |

### 15.8 Frontend Components

- `AccessibilitySettings.jsx` — High contrast, reduced motion, font size, color blind mode
- `KeyboardShortcuts.jsx` — View and customize keyboard shortcuts
- `CommandPalette.jsx` — Ctrl+K command palette for quick access
- `A11yAnnouncer.jsx` — Screen reader live region component

### 15.9 Dependencies

- System 5 (UI Customization) — accessibility overrides theme and animation settings
- System 9 (Dashboard) — screen reader labels for graphs and panels

---

## 16. Implementation Phases

### Phase 1: Foundation (Weeks 1-4)

**Goal:** Hardware detection, performance engine, GPU detection, display server detection — the "brain" that all other systems depend on.

| Task | System | Effort |
|------|--------|--------|
| Create `services/performance/` module structure | S1 | 2h |
| Implement `detector.rs` — CPU, RAM detection | S1 | 1d |
| Implement GPU detection (all platforms) | S3 | 2d |
| Implement display server detection | S4 | 1d |
| Implement `profiles.rs` — tier classification + profile selection | S1 | 1d |
| Add `performance_config` table + repo | S1 | 4h |
| Add IPC commands for detection and config | S1, S3, S4 | 1d |
| Frontend: SystemInfoCard, PerformanceSettings | S1 | 1d |
| Write unit tests for detection logic | S1, S3, S4 | 1d |
| **Phase 1 Total** | | **~9 days** |

### Phase 2: Resource Management (Weeks 5-8)

**Goal:** Memory management, network monitoring, process resource limits — protect the machine.

| Task | System | Effort |
|------|--------|--------|
| Create `services/memory/` module | S2 | 2h |
| Implement per-platform memory sampling | S2 | 2d |
| Implement memory monitoring loop | S2 | 1d |
| Implement leak detection algorithm | S2 | 1d |
| Implement OOM prevention | S2 | 4h |
| Add memory database tables | S2 | 4h |
| Add memory IPC commands | S2 | 4h |
| Create `services/network/` module | S6 | 2h |
| Implement per-platform bandwidth monitoring | S6 | 2d |
| Implement network connection tracking | S6 | 1d |
| Implement per-project network limits | S6 | 1d |
| Add network database tables | S6 | 4h |
| Add network IPC commands | S6 | 4h |
| Extend ProjectSettings with resource limits | S8 | 1d |
| Frontend: MemorySettings, MemoryDashboard | S2 | 1d |
| Frontend: NetworkSettings, NetworkDashboard | S6 | 1d |
| **Phase 2 Total** | | **~14 days** |

### Phase 3: UI Customization (Weeks 9-12)

**Goal:** Theme engine, animation control, layout customization, feature toggles — make it yours.

| Task | System | Effort |
|------|--------|--------|
| Create theme engine (CSS variable system) | S5 | 2d |
| Implement HSL color picker | S5 | 1d |
| Implement theme import/export | S5 | 4h |
| Create pre-built themes (8 themes) | S5 | 1d |
| Implement animation config system | S5 | 1d |
| Implement layout config system | S5 | 1d |
| Implement feature toggle system | S5 | 1d |
| Add theme/UI database tables | S5 | 4h |
| Add theme/UI IPC commands | S5 | 1d |
| Frontend: ThemeProvider, ColorPicker, ThemeImportExport | S5 | 2d |
| Frontend: AnimationSettings, LayoutSettings, FeatureToggles | S5 | 1d |
| Frontend: Settings/ThemeSection | S5 | 1d |
| **Phase 3 Total** | | **~12 days** |

### Phase 4: Notification System (Weeks 13-16)

**Goal:** Universal notification delivery — never miss an event.

| Task | System | Effort |
|------|--------|--------|
| Create `services/notification/` module | S7 | 2h |
| Implement in-app toast channel | S7 | 4h |
| Implement email channel (SMTP) | S7 | 2d |
| Implement email OAuth2 (Gmail, Outlook) | S7 | 2d |
| Implement webhook channel | S7 | 1d |
| Implement WebSocket channel | S7 | 1d |
| Implement desktop notification channel | S7 | 4h |
| Implement Telegram bot channel | S7 | 1d |
| Implement Discord webhook channel | S7 | 4h |
| Implement Slack webhook channel | S7 | 4h |
| Implement rules engine | S7 | 2d |
| Implement template system | S7 | 1d |
| Implement notification history | S7 | 4h |
| Implement rate limiting | S7 | 4h |
| Implement quiet hours | S7 | 4h |
| Add notification database tables | S7 | 4h |
| Add notification IPC commands | S7 | 1d |
| Frontend: NotificationSettings, ChannelConfig | S7 | 2d |
| Frontend: RulesEditor, TemplateEditor | S7 | 2d |
| Frontend: NotificationHistory | S7 | 1d |
| **Phase 4 Total** | | **~19 days** |

### Phase 5: Dashboard (Weeks 17-20)

**Goal:** Real-time monitoring with draggable panels, graphs, and alerts.

| Task | System | Effort |
|------|--------|--------|
| Create `services/dashboard/` module | S9 | 2h |
| Implement metrics collection (all platforms) | S9 | 2d |
| Implement metrics storage and rotation | S9 | 1d |
| Implement alert rules engine | S9 | 1d |
| Implement CSV/JSON export | S9 | 4h |
| Add dashboard database tables | S9 | 4h |
| Add dashboard IPC commands | S9 | 1d |
| Frontend: DashboardPage, DashboardGrid | S9 | 2d |
| Frontend: SystemOverviewPanel, ProcessPanel | S9 | 2d |
| Frontend: NetworkPanel, MemoryPanel, GPUPanel | S9 | 2d |
| Frontend: AlertsPanel, MetricGraph, Sparkline | S9 | 2d |
| Frontend: DashboardConfig UI | S9 | 4h |
| **Phase 5 Total** | | **~14 days** |

### Phase 6: Platform Hardening (Weeks 21-24)

**Goal:** Security, sandboxing, cgroups, audit logging — lock it down.

| Task | System | Effort |
|------|--------|--------|
| Create `services/platform/` module | S10 | 2h |
| Implement Linux cgroup v2 integration | S10 | 2d |
| Implement Linux systemd service management | S10 | 1d |
| Implement macOS LaunchAgent management | S10 | 4h |
| Implement Windows Job Objects (Rust) | S10 | 1d |
| Implement process mitigation policies (Windows) | S10 | 4h |
| Implement process sandboxing | S11 | 2d |
| Implement encrypted config at rest | S11 | 1d |
| Implement audit logging | S11 | 1d |
| Implement certificate pinning | S11 | 4h |
| Implement clipboard security | S11 | 4h |
| Add platform/security database tables | S10, S11 | 4h |
| Add platform/security IPC commands | S10, S11 | 1d |
| Frontend: PlatformSettings, SecuritySettings | S10, S11 | 1d |
| Frontend: AuditLog | S11 | 1d |
| **Phase 6 Total** | | **~13 days** |

### Phase 7: Build Pipeline (Weeks 25-26)

**Goal:** Automated CI/CD, signing, delta updates.

| Task | System | Effort |
|------|--------|--------|
| Create GitHub Actions workflow (matrix build) | S12 | 1d |
| Set up Linux GPG signing | S12 | 4h |
| Set up macOS codesign + notarization | S12 | 1d |
| Set up Windows Authenticode signing | S12 | 1d |
| Configure LTO + strip in Cargo.toml | S12 | 4h |
| Set up UPX compression (optional) | S12 | 2h |
| Set up delta updates | S12 | 4h |
| Create beta channel workflow | S12 | 4h |
| Test full pipeline end-to-end | S12 | 1d |
| **Phase 7 Total** | | **~7 days** |

### Phase 8: Accessibility & Polish (Weeks 27-30)

**Goal:** WCAG compliance, keyboard navigation, screen reader support, final polish.

| Task | System | Effort |
|------|--------|--------|
| Audit all components for keyboard navigation | S13 | 2d |
| Add ARIA labels to all interactive elements | S13 | 2d |
| Implement reduced motion mode | S13 | 1d |
| Implement high contrast mode | S13 | 1d |
| Implement font scaling | S13 | 4h |
| Implement color blind modes | S13 | 1d |
| Implement Command Palette (Ctrl+K) | S13 | 1d |
| Implement keyboard shortcuts system | S13 | 1d |
| Per-project deep config UI | S8 | 2d |
| Cross-system integration testing | All | 2d |
| Performance profiling and optimization | All | 2d |
| Documentation (README, user guide) | All | 1d |
| **Phase 8 Total** | | **~15 days** |

---

## 17. Database Migration Plan

### 17.1 Migration Strategy

Use a versioned migration system. Each migration is a numbered SQL file:

```
migrations/
  001_initial_schema.sql          — existing tables (projects, categories, settings)
  002_performance_config.sql     — System 1
  003_memory_tables.sql          — System 2
  004_gpu_config.sql             — System 3
  005_display_config.sql         — System 4
  006_themes_and_ui.sql          — System 5
  007_network_tables.sql         — System 6
  008_notification_tables.sql    — System 7
  009_project_settings_expansion.sql — System 8
  010_dashboard_tables.sql       — System 9
  011_platform_config.sql        — System 10
  012_security_tables.sql        — System 11
  013_accessibility_config.sql   — System 13
```

Migration runner:
- Store `schema_version` in a `meta` table
- On startup, check current version and apply any pending migrations
- Each migration is idempotent (uses `CREATE TABLE IF NOT EXISTS`, `ALTER TABLE ADD COLUMN` with existence check)
- Never drop columns — only add or modify

### 17.2 New Tables Summary

| Phase | Tables Added | Count |
|-------|-------------|-------|
| Phase 1 | `performance_config`, `gpu_config`, `gpu_blacklist`, `display_config` | 4 |
| Phase 2 | `memory_config`, `project_memory_config`, `memory_samples`, `memory_alerts`, `network_config`, `project_network_config`, `network_samples`, `network_alerts` | 8 |
| Phase 3 | `themes`, `ui_config` | 2 |
| Phase 4 | `notification_channels`, `notification_rules`, `notification_templates`, `notification_log` | 4 |
| Phase 5 | `dashboard_config`, `dashboard_metrics`, `alert_rules`, `alerts` | 4 |
| Phase 6 | `platform_config`, `security_config`, `audit_log` | 3 |
| Phase 8 | `accessibility_config` | 1 |
| **Total** | | **26 new tables** |

### 17.3 Modified Tables

| Table | Change | Phase |
|-------|--------|-------|
| `project_settings` | Add 30+ new columns for deep config | Phase 2+ |
| `settings` | Continue using for simple key-value settings | — |

### 17.4 Backward Compatibility

- Old `project_settings` rows with only runtime/version columns continue to work
- New columns have sensible defaults, so existing rows don't break
- Old frontend reads existing columns; new frontend reads all columns
- Migration is one-way (forward only)

---

## 18. Risk Assessment

### 18.1 High Risk

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| cgroup v2 requires root | Feature unusable for most users | Medium | Make cgroup optional; fall back to `nice`/`ulimit` |
| Wayland tray icon broken | Linux Wayland users lose tray | High | Test extensively; provide title bar fallback |
| Memory monitoring overhead | Monitoring itself uses too much CPU | Medium | Use 2s+ intervals; adaptive based on tier |
| Email OAuth2 complexity | Gmail/Outlook OAuth is complex to implement | High | Start with SMTP + app passwords; OAuth in v2 |
| macOS codesigning requires paid Apple Developer account | Cannot sign for distribution | Low | Already handled; document for contributors |
| Breaking existing settings | Users lose config on migration | Medium | Idempotent migrations; backup before migrate |

### 18.2 Medium Risk

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| GPU detection fails on exotic hardware | Wrong profile selected | Medium | Always allow manual override |
| Network rate limiting unreliable (advisory only) | Users expect hard limits | Low | Document as advisory; suggest OS-level tools for hard limits |
| WebView rendering differences across platforms | UI looks different on Linux vs Windows | Medium | Test on all platforms; use CSS fallbacks |
| Notification delivery failures | Users miss critical alerts | Medium | Retry logic; in-app as fallback; log all failures |
| Dashboard performance with many panels | UI lag with 8+ panels | Medium | Lazy render; reduce refresh rate; virtual scrolling for data |

### 18.3 Low Risk

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Theme import/export JSON parsing errors | Theme fails to load | Low | Validate on import; provide defaults |
| UPX compression causes antivirus false positives | Users scared by AV warnings | Medium | Document; make UPX optional in build |
| Keyboard shortcuts conflict with OS | User can't use shortcuts | Low | Check for conflicts; allow remapping |

---

## 19. Estimated Effort

### Per System

| System | Phase | Estimated Days | Estimated Weeks |
|--------|-------|---------------|-----------------|
| S1: Performance Engine | 1 | 5 | 1 |
| S3: GPU Optimization | 1 | 3 | 0.6 |
| S4: Display Server | 1 | 2 | 0.4 |
| S2: Memory Management | 2 | 5 | 1 |
| S6: Network Management | 2 | 5 | 1 |
| S8: Per-Project Config | 2+8 | 3 | 0.6 |
| S5: UI Customization | 3 | 9 | 1.8 |
| S7: Notification System | 4 | 15 | 3 |
| S9: Dashboard | 5 | 10 | 2 |
| S10: Platform Optimizations | 6 | 6 | 1.2 |
| S11: Security & Hardening | 6 | 5 | 1 |
| S12: Build Pipeline | 7 | 5 | 1 |
| S13: Accessibility | 8 | 10 | 2 |
| **Total** | | **~100 days** | **~20 weeks** |

### Per Phase

| Phase | Weeks | Deliverable |
|-------|-------|-------------|
| Phase 1: Foundation | 1-4 | Hardware detection, auto-tuning, GPU/display awareness |
| Phase 2: Resource Management | 5-8 | Memory limits, network monitoring, process protection |
| Phase 3: UI Customization | 9-12 | Themes, animations, layout, feature toggles |
| Phase 4: Notification System | 13-16 | Email, webhooks, Telegram, Discord, rules engine |
| Phase 5: Dashboard | 17-20 | Real-time monitoring, graphs, alerts |
| Phase 6: Platform Hardening | 21-24 | Security, sandboxing, cgroups, audit |
| Phase 7: Build Pipeline | 25-26 | CI/CD, signing, delta updates |
| Phase 8: Accessibility & Polish | 27-30 | WCAG, keyboard nav, screen reader, final polish |

**Total timeline: ~30 weeks (7.5 months) for one developer**

With 2 developers working in parallel on independent systems: ~16-18 weeks.

---

## 20. Full TODO Checklist

### Phase 1: Foundation

- [ ] Create `src-tauri/src/services/performance/mod.rs`
- [ ] Create `src-tauri/src/services/performance/detector.rs`
- [ ] Create `src-tauri/src/services/performance/profiles.rs`
- [ ] Create `src-tauri/src/services/performance/gpu.rs`
- [ ] Create `src-tauri/src/services/performance/display.rs`
- [ ] Implement CPU core count detection (all platforms)
- [ ] Implement CPU benchmark scoring
- [ ] Implement RAM detection (all platforms)
- [ ] Implement GPU vendor detection — Linux (`/sys/class/drm`, `lspci`)
- [ ] Implement GPU vendor detection — macOS (`system_profiler`)
- [ ] Implement GPU vendor detection — Windows (WMI)
- [ ] Implement GPU generation classification (NVIDIA, AMD, Intel)
- [ ] Implement display server detection — Wayland (`WAYLAND_DISPLAY`)
- [ ] Implement display server detection — X11 (`DISPLAY`)
- [ ] Implement machine tier classification algorithm
- [ ] Implement performance profile selection based on tier
- [ ] Add `performance_config` table migration
- [ ] Add `gpu_config` table migration
- [ ] Add `gpu_blacklist` table migration
- [ ] Add `display_config` table migration
- [ ] Add IPC: `detect_system_capabilities`
- [ ] Add IPC: `get_performance_profile`
- [ ] Add IPC: `set_performance_profile`
- [ ] Add IPC: `set_performance_setting`
- [ ] Add IPC: `reset_performance_settings`
- [ ] Add IPC: `get_performance_config`
- [ ] Add IPC: `detect_gpu`
- [ ] Add IPC: `get_gpu_profile`
- [ ] Add IPC: `get_gpu_config`
- [ ] Add IPC: `set_gpu_config`
- [ ] Add IPC: `detect_display_server`
- [ ] Add IPC: `get_display_config`
- [ ] Add IPC: `set_display_config`
- [ ] Add IPC: `get_display_capabilities`
- [ ] Frontend: `src/components/SystemInfoCard.jsx`
- [ ] Frontend: `src/components/settings/PerformanceSection.jsx`
- [ ] Frontend: SystemInfoCard integrated into settings
- [ ] Unit tests for detection logic
- [ ] Unit tests for tier classification
- [ ] Integration test: startup detection → profile selection

### Phase 2: Resource Management

- [ ] Create `src-tauri/src/services/memory/mod.rs`
- [ ] Create `src-tauri/src/services/memory/monitor.rs`
- [ ] Create `src-tauri/src/services/memory/leak_detector.rs`
- [ ] Create `src-tauri/src/services/memory/limiter.rs`
- [ ] Implement Linux memory sampling (`/proc/[pid]/status`)
- [ ] Implement macOS memory sampling (`task_info`)
- [ ] Implement Windows memory sampling (`GetProcessMemoryInfo`)
- [ ] Implement app self-memory tracking
- [ ] Implement periodic memory sampling loop
- [ ] Implement leak detection (linear regression on RSS history)
- [ ] Implement OOM prevention (headroom check before process start)
- [ ] Implement per-project memory soft/hard limits
- [ ] Implement hard limit actions (stop, restart, notify)
- [ ] Add `memory_config` table migration
- [ ] Add `project_memory_config` table migration
- [ ] Add `memory_samples` table migration
- [ ] Add `memory_alerts` table migration
- [ ] Add IPC: `get_memory_config`
- [ ] Add IPC: `set_memory_config`
- [ ] Add IPC: `get_project_memory_config`
- [ ] Add IPC: `set_project_memory_config`
- [ ] Add IPC: `get_memory_samples`
- [ ] Add IPC: `get_memory_alerts`
- [ ] Add IPC: `acknowledge_memory_alert`
- [ ] Add IPC: `get_process_memory_usage`
- [ ] Create `src-tauri/src/services/network/mod.rs`
- [ ] Create `src-tauri/src/services/network/monitor.rs`
- [ ] Create `src-tauri/src/services/network/limiter.rs`
- [ ] Create `src-tauri/src/services/network/platform/linux.rs`
- [ ] Create `src-tauri/src/services/network/platform/macos.rs`
- [ ] Create `src-tauri/src/services/network/platform/windows.rs`
- [ ] Implement Linux bandwidth monitoring (`/proc/net/dev`)
- [ ] Implement macOS bandwidth monitoring (`netstat -ib`)
- [ ] Implement Windows bandwidth monitoring (`Get-NetAdapterStatistics`)
- [ ] Implement connection tracking (all platforms)
- [ ] Implement per-project bandwidth limits
- [ ] Implement transfer cap tracking
- [ ] Implement network quality metrics (ping, DNS)
- [ ] Add `network_config` table migration
- [ ] Add `project_network_config` table migration
- [ ] Add `network_samples` table migration
- [ ] Add `network_alerts` table migration
- [ ] Add IPC: `get_network_config`
- [ ] Add IPC: `set_network_config`
- [ ] Add IPC: `get_project_network_config`
- [ ] Add IPC: `set_project_network_config`
- [ ] Add IPC: `get_network_samples`
- [ ] Add IPC: `get_network_connections`
- [ ] Add IPC: `get_network_alerts`
- [ ] Add IPC: `acknowledge_network_alert`
- [ ] Add IPC: `get_network_quality`
- [ ] Add IPC: `set_interface_limit`
- [ ] Extend `project_settings` table with resource limit columns
- [ ] Frontend: `src/components/settings/MemorySection.jsx`
- [ ] Frontend: `src/components/MemoryDashboard.jsx`
- [ ] Frontend: `src/components/project-settings/ResourceLimits.jsx`
- [ ] Frontend: `src/components/settings/NetworkSection.jsx`
- [ ] Frontend: `src/components/NetworkDashboard.jsx`
- [ ] Frontend: `src/components/ConnectionTable.jsx`
- [ ] Frontend: `src/components/MemoryAlerts.jsx`
- [ ] Frontend: `src/components/NetworkAlerts.jsx`

### Phase 3: UI Customization

- [ ] Create `src/components/theme/ThemeProvider.jsx`
- [ ] Create `src/components/theme/ColorPicker.jsx`
- [ ] Create `src/components/theme/ThemeImportExport.jsx`
- [ ] Create `src/components/theme/ThemePreview.jsx`
- [ ] Implement CSS variable generation from theme
- [ ] Implement theme switching (light/dark/system/custom)
- [ ] Implement HSL color picker component
- [ ] Implement theme import from JSON
- [ ] Implement theme export to JSON
- [ ] Create pre-built themes: Default Dark, Default Light
- [ ] Create pre-built themes: High Contrast Dark, High Contrast Light
- [ ] Create pre-built themes: Solarized Dark, Solarized Light
- [ ] Create pre-built themes: Nord, Dracula, Catppuccin, Tokyo Night
- [ ] Implement animation config system (per-element toggles)
- [ ] Implement layout config (sidebar width, density, view mode)
- [ ] Implement feature toggle system (13 toggles)
- [ ] Add `themes` table migration
- [ ] Add `ui_config` table migration
- [ ] Add IPC: `get_themes`, `get_theme`, `create_theme`, `update_theme`, `delete_theme`
- [ ] Add IPC: `set_active_theme`, `import_theme`, `export_theme`
- [ ] Add IPC: `get_animation_config`, `set_animation_config`
- [ ] Add IPC: `get_layout_config`, `set_layout_config`
- [ ] Add IPC: `get_feature_toggles`, `set_feature_toggle`
- [ ] Add IPC: `get_font_config`, `set_font_config`
- [ ] Frontend: `src/components/settings/ThemeSection.jsx`
- [ ] Frontend: `src/components/settings/AnimationSettings.jsx`
- [ ] Frontend: `src/components/settings/LayoutSettings.jsx`
- [ ] Frontend: `src/components/settings/FeatureToggles.jsx`

### Phase 4: Notification System

- [ ] Create `src-tauri/src/services/notification/mod.rs`
- [ ] Create `src-tauri/src/services/notification/channels/in_app.rs`
- [ ] Create `src-tauri/src/services/notification/channels/email.rs`
- [ ] Create `src-tauri/src/services/notification/channels/webhook.rs`
- [ ] Create `src-tauri/src/services/notification/channels/websocket.rs`
- [ ] Create `src-tauri/src/services/notification/channels/desktop.rs`
- [ ] Create `src-tauri/src/services/notification/channels/telegram.rs`
- [ ] Create `src-tauri/src/services/notification/channels/discord.rs`
- [ ] Create `src-tauri/src/services/notification/channels/slack.rs`
- [ ] Create `src-tauri/src/services/notification/rules.rs`
- [ ] Create `src-tauri/src/services/notification/templates.rs`
- [ ] Create `src-tauri/src/services/notification/history.rs`
- [ ] Implement in-app toast channel (Tauri events)
- [ ] Implement email channel (SMTP + TLS)
- [ ] Implement Gmail OAuth2 flow
- [ ] Implement Outlook OAuth2 flow
- [ ] Implement webhook channel (POST + retry + HMAC)
- [ ] Implement WebSocket channel (connect + reconnect)
- [ ] Implement desktop notification channel (notify-send / osascript / PowerShell)
- [ ] Implement Telegram bot channel
- [ ] Implement Discord webhook channel
- [ ] Implement Slack webhook channel
- [ ] Implement rules engine (event filter → conditions → actions)
- [ ] Implement template system with variable interpolation
- [ ] Implement notification history logging
- [ ] Implement rate limiting (per minute/hour/day)
- [ ] Implement quiet hours
- [ ] Add `notification_channels` table migration
- [ ] Add `notification_rules` table migration
- [ ] Add `notification_templates` table migration
- [ ] Add `notification_log` table migration
- [ ] Add IPC: `get_notification_channels`
- [ ] Add IPC: `create_notification_channel`
- [ ] Add IPC: `update_notification_channel`
- [ ] Add IPC: `delete_notification_channel`
- [ ] Add IPC: `test_notification_channel`
- [ ] Add IPC: `get_notification_rules`
- [ ] Add IPC: `create_notification_rule`
- [ ] Add IPC: `update_notification_rule`
- [ ] Add IPC: `delete_notification_rule`
- [ ] Add IPC: `get_notification_templates`
- [ ] Add IPC: `create_notification_template`
- [ ] Add IPC: `update_notification_template`
- [ ] Add IPC: `delete_notification_template`
- [ ] Add IPC: `get_notification_log`
- [ ] Add IPC: `retry_notification`
- [ ] Add IPC: `get_notification_stats`
- [ ] Frontend: `src/components/settings/NotificationSection.jsx`
- [ ] Frontend: `src/components/notifications/ChannelConfig.jsx`
- [ ] Frontend: `src/components/notifications/RulesEditor.jsx`
- [ ] Frontend: `src/components/notifications/TemplateEditor.jsx`
- [ ] Frontend: `src/components/notifications/NotificationHistory.jsx`
- [ ] Frontend: `src/components/notifications/NotificationTest.jsx`
- [ ] Frontend: `src/components/notifications/QuietHoursConfig.jsx`

### Phase 5: Dashboard

- [ ] Create `src-tauri/src/services/dashboard/mod.rs`
- [ ] Create `src-tauri/src/services/dashboard/collector.rs`
- [ ] Create `src-tauri/src/services/dashboard/alerting.rs`
- [ ] Create `src-tauri/src/services/dashboard/exporter.rs`
- [ ] Implement CPU metrics collection (Linux `/proc/stat`, macOS `host_processor_info`, Windows `GetSystemTimes`)
- [ ] Implement memory metrics collection
- [ ] Implement disk metrics collection
- [ ] Implement network metrics collection
- [ ] Implement GPU metrics collection (nvidia-smi, IOKit, WMI)
- [ ] Implement metrics storage with rotation
- [ ] Implement alert rules engine (metric → condition → threshold → alert)
- [ ] Implement CSV export
- [ ] Implement JSON export
- [ ] Add `dashboard_config` table migration
- [ ] Add `dashboard_metrics` table migration
- [ ] Add `alert_rules` table migration
- [ ] Add `alerts` table migration
- [ ] Add IPC: `get_dashboard_config`
- [ ] Add IPC: `set_dashboard_config`
- [ ] Add IPC: `get_system_metrics`
- [ ] Add IPC: `get_metrics_history`
- [ ] Add IPC: `get_process_metrics`
- [ ] Add IPC: `get_alert_rules`
- [ ] Add IPC: `create_alert_rule`
- [ ] Add IPC: `update_alert_rule`
- [ ] Add IPC: `delete_alert_rule`
- [ ] Add IPC: `get_active_alerts`
- [ ] Add IPC: `acknowledge_alert`
- [ ] Add IPC: `dismiss_alert`
- [ ] Add IPC: `export_metrics`
- [ ] Add IPC: `clear_metrics_history`
- [ ] Frontend: `src/components/dashboard/DashboardPage.jsx`
- [ ] Frontend: `src/components/dashboard/DashboardGrid.jsx`
- [ ] Frontend: `src/components/dashboard/SystemOverviewPanel.jsx`
- [ ] Frontend: `src/components/dashboard/ProcessPanel.jsx`
- [ ] Frontend: `src/components/dashboard/NetworkPanel.jsx`
- [ ] Frontend: `src/components/dashboard/MemoryPanel.jsx`
- [ ] Frontend: `src/components/dashboard/GPUPanel.jsx`
- [ ] Frontend: `src/components/dashboard/AlertsPanel.jsx`
- [ ] Frontend: `src/components/dashboard/DiskPanel.jsx`
- [ ] Frontend: `src/components/dashboard/MetricGraph.jsx`
- [ ] Frontend: `src/components/dashboard/Sparkline.jsx`
- [ ] Frontend: `src/components/dashboard/DashboardConfig.jsx`

### Phase 6: Platform Hardening

- [ ] Create `src-tauri/src/services/platform/mod.rs`
- [ ] Create `src-tauri/src/services/platform/linux.rs`
- [ ] Create `src-tauri/src/services/platform/macos.rs`
- [ ] Create `src-tauri/src/services/platform/windows.rs`
- [ ] Implement cgroup v2 resource limits (Linux)
- [ ] Implement systemd user service management (Linux)
- [ ] Implement LaunchAgent management (macOS)
- [ ] Implement Windows Job Objects in Rust (migrate from C++ addon)
- [ ] Implement process mitigation policies (Windows ACG, CIG)
- [ ] Create `src-tauri/src/services/process_supervisor/mod.rs`
- [ ] Create `src-tauri/src/services/process_supervisor/resource_limiter.rs`
- [ ] Implement process sandboxing (3 levels)
- [ ] Implement encrypted config at rest
- [ ] Create `src-tauri/src/services/audit/mod.rs`
- [ ] Implement audit logging for all significant actions
- [ ] Implement certificate pinning
- [ ] Implement clipboard security (auto-clear)
- [ ] Add `platform_config` table migration
- [ ] Add `security_config` table migration
- [ ] Add `audit_log` table migration
- [ ] Add IPC: `get_platform_config`
- [ ] Add IPC: `set_platform_config`
- [ ] Add IPC: `get_platform_capabilities`
- [ ] Add IPC: `install_launch_agent`
- [ ] Add IPC: `remove_launch_agent`
- [ ] Add IPC: `install_systemd_service`
- [ ] Add IPC: `remove_systemd_service`
- [ ] Add IPC: `get_cgroup_status`
- [ ] Add IPC: `get_audit_log`
- [ ] Add IPC: `get_security_config`
- [ ] Add IPC: `set_security_config`
- [ ] Add IPC: `get_certificate_pins`
- [ ] Add IPC: `add_certificate_pin`
- [ ] Add IPC: `remove_certificate_pin`
- [ ] Add IPC: `clear_clipboard`
- [ ] Frontend: `src/components/settings/PlatformSection.jsx`
- [ ] Frontend: `src/components/settings/SecuritySection.jsx`
- [ ] Frontend: `src/components/AuditLog.jsx`
- [ ] Frontend: `src/components/CertificatePinning.jsx`
- [ ] Migrate `electron/job/` C++ addon to Rust `windows-sys`

### Phase 7: Build Pipeline

- [ ] Create `.github/workflows/release.yml`
- [ ] Set up build matrix (Linux x64/arm64, macOS x64/arm64, Windows x64)
- [ ] Configure Linux GPG signing
- [ ] Configure macOS codesign + notarization
- [ ] Configure Windows Authenticode signing
- [ ] Add LTO to Cargo.toml release profile
- [ ] Add strip to Cargo.toml release profile
- [ ] Set up UPX compression (optional)
- [ ] Configure Tauri delta updates
- [ ] Create beta channel workflow
- [ ] Set up GitHub Actions caching (cargo, npm)
- [ ] Test full pipeline end-to-end
- [ ] Document release process in README

### Phase 8: Accessibility & Polish

- [ ] Audit all components for keyboard navigation
- [ ] Add `tabIndex` to all interactive elements
- [ ] Add `aria-label` to all buttons and links
- [ ] Add `aria-labelledby` to form controls
- [ ] Add `aria-live` regions for dynamic content
- [ ] Add `role="status"` to status indicators
- [ ] Add `role="alert"` to error messages
- [ ] Add text alternatives for all graphs
- [ ] Implement reduced motion mode (respect `prefers-reduced-motion`)
- [ ] Implement high contrast mode
- [ ] Implement font scaling (12-20px)
- [ ] Implement color blind modes (Deuteranopia, Protanopia, Tritanopia)
- [ ] Implement Command Palette (Ctrl+K / Cmd+K)
- [ ] Implement customizable keyboard shortcuts
- [ ] Implement `src/components/settings/AccessibilitySection.jsx`
- [ ] Implement `src/components/KeyboardShortcuts.jsx`
- [ ] Implement `src/components/CommandPalette.jsx`
- [ ] Implement `src/components/A11yAnnouncer.jsx`
- [ ] Per-project deep config UI — CommandConfig
- [ ] Per-project deep config UI — MonitoringConfig
- [ ] Per-project deep config UI — GitAutomation
- [ ] Per-project deep config UI — ProjectNotifications
- [ ] Per-project deep config UI — ProjectEditorConfig
- [ ] Cross-system integration testing
- [ ] Performance profiling and optimization
- [ ] Update README.md with new features
- [ ] Create user guide documentation
- [ ] Update AGENTS.md with new architecture

---

## Appendix A: New Rust Crate Dependencies

| Crate | Version | Purpose | System |
|-------|---------|---------|--------|
| `zbus` | 4 | D-Bus integration (Linux notifications, tray) | S4, S7, S10 |
| `sysinfo` | 0.30 | Cross-system info (CPU, memory, processes) | S1, S2, S9 |
| `nix` | 0.29 | Already used — extend for cgroups | S2, S10 |
| `windows-sys` | 0.60 | Already used — extend for WMI, Performance Counters | S10, S11 |
| `lettre` | 0.11 | SMTP email sending | S7 |
| `reqwest` | 0.12 | Already used — webhook/HTTP notifications | S7 |
| `tokio-tungstenite` | 0.24 | Already used — WebSocket notifications | S7 |
| `notify-rust` | 4 | Desktop notifications (cross-platform) | S7 |
| `keyring` | 3 | Secure credential storage | S7, S11 |
| `chrono` | 0.4 | Already used — time handling | All |
| `uuid` | 1 | Already used — unique IDs | All |
| `csv` | 1 | CSV export | S9 |

---

## Appendix B: New IPC Command Count

| System | Commands Added |
|--------|---------------|
| S1: Performance | 6 |
| S3: GPU | 4 |
| S4: Display | 4 |
| S2: Memory | 8 |
| S6: Network | 10 |
| S5: UI Customization | 14 |
| S7: Notification | 16 |
| S8: Per-Project Config | 4 |
| S9: Dashboard | 14 |
| S10: Platform | 8 |
| S11: Security | 8 |
| S13: Accessibility | 3 |
| **Total New** | **~99** |

Combined with existing ~95 commands, the app will have ~194 IPC commands at 1.0.

---

## Appendix C: Frontend Component Count

| Area | Components |
|------|-----------|
| Theme engine | 4 (ThemeProvider, ColorPicker, ThemeImportExport, ThemePreview) |
| Dashboard | 12 (DashboardPage, DashboardGrid, 6 panels, MetricGraph, Sparkline, DashboardConfig) |
| Notifications | 6 (NotificationSettings, ChannelConfig, RulesEditor, TemplateEditor, NotificationHistory, NotificationTest, QuietHoursConfig) |
| Settings sections | 7 (Performance, Memory, Network, Notification, Dashboard, Accessibility, Theme, FeatureToggles, Platform, Security) |
| Project settings | 5 (CommandConfig, ResourceLimits, MonitoringConfig, ProjectNotifications, ProjectEditorConfig) |
| Accessibility | 4 (AccessibilitySettings, KeyboardShortcuts, CommandPalette, A11yAnnouncer) |
| System info | 2 (SystemInfoCard, GPUInfo) |
| Alerts | 3 (MemoryAlerts, NetworkAlerts, AuditLog) |
| Network | 2 (NetworkDashboard, ConnectionTable) |
| **Total New** | **~45** |

Combined with existing 21 components, the app will have ~66 components at 1.0.
