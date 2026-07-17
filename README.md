<h4 align="right"><strong>English</strong> | <a href="README_CN.md">简体中文</a></h4>
<p align="center">
    <img src=https://gw.alipayobjects.com/zos/k/fa/logo-modified.png width=138/>
</p>
<h1 align="center">Pake Plus</h1>
<p align="center"><strong>Desktop browser app based on Pake —— Adblock · Cache · Clipboard · Settings</strong></p>

<p align="center">Welcome page with URL input  ·  Auto-complete  ·  Desktop shortcut  ·  Multi-module integration</p>

## Features

### 🛡 Adblock

Dual-layer ad filtering powered by EasyList rules, operating at both the network request level and DOM element level:

- **Network Interception**: Registers Tauri resource request interceptors to match every HTTP/HTTPS request against rule sets, blocking blacklisted resources at the source
- **DOM Hiding**: Extracts CSS selectors from rules and injects them as hide styles, with MutationObserver monitoring for dynamically inserted elements
- **Custom Rules**: User-defined filtering rules, one per line, using `||domain^` (block) or `##.selector` (hide) format
- **Statistics**: Real-time block count displayed on the tray icon

### 💾 Offline Cache

HTTP-layer transparent proxy that intercepts `fetch` and `XMLHttpRequest`, caching responses to disk for offline access:

- **Transparent Proxy**: Intercepts all GET requests via `cache_fetch` IPC; queries local index by URL hash and serves cached data directly on hit
- **Auto-Cache & LRU Eviction**: On cache miss, fetches via `reqwest`, writes to disk with `Cache-Control: max-age` TTL; LRU eviction triggers when disk usage exceeds the limit (default 200MB, adjustable 50-1000MB via settings slider)
- **Persistent Index**: `cache-index.json` stores URL→hash→file mapping, surviving app restarts; real-time stats (file count, used space, hit rate, hit/miss counts) displayed in settings panel via `cache_stats_json` IPC
- **Offline Mode**: Monitors `online`/`offline` events; on disconnect shows yellow top banner ("You are offline") and orange OFFLINE badge; serves cached pages normally, displays friendly offline page for uncached URLs; green "⚡ from cache" toast on cache hit

### 📋 Clipboard Management

System-level clipboard monitoring with automatic recording, full-text search, and one-click reuse:

- **System Monitor**: Calls native OS APIs via Rust FFI (Windows `AddClipboardFormatListener`, macOS `NSPasteboard`, Linux X11 selection), running on a dedicated thread without blocking the UI
- **SHA-256 Deduplication**: Computes content hashes on each change, with dual-guard logic to filter duplicate notifications and write-back interference
- **History Panel**: Press `Ctrl+Shift+V` to open a 320×480 floating panel showing records in reverse chronological order, each with a content preview, timestamp, and source application name
- **Full-Text Search**: Fuzzy matching in both Chinese and English, multi-keyword AND logic via space separation, real-time results within ~120ms
- **One-Click Reuse**: Click any record to write its content back to the system clipboard, with a 3-second toast notification; long text (>100 chars) supports expand/collapse
- **Privacy Filtering**: Automatically skips very short text (<2 chars), very long text (>10,000 chars), password-like strings (6-30 chars mixing letters and digits), and credit card numbers (13-19 digits with Luhn validation); supports per-application ignore lists
- **Auto-Cleanup**: Default cap of 2,000 records with 30-day retention; excess records are purged in 500-record batches, with hourly background cleanup
- **Persistence**: SQLite WAL mode storage with unique hash index and UPSERT semantics (re-copying the same text refreshes its timestamp without creating duplicates)

### ⚙ Settings Panel & Navigation

A unified control center managing all module configurations, plus welcome page navigation and desktop deployment:

- **Welcome Page & URL Navigation**: Full-screen dark welcome page on startup with Pake Plus branding, three live module status cards (via `get_module_stats` IPC), URL input with auto-complete (auto-add `https://` prefix and `.com` suffix for bare domains, case normalization), and Start button. Navigation keeps welcome visible during transition (no page flash). `window.name` cross-origin flag prevents re-showing on subsequent loads.
- **🏠 Home / ↻ Refresh / ⚙ Settings Buttons**: Three floating buttons created via pure DOM API — bottom-left Home button returns to welcome page; top-right Refresh reloads the current URL; bottom-right gear opens settings panel
- **Visual Configuration**: Right-side sliding drawer with six tabs (General / Adblock / Cache / Clipboard / Data / About); changes apply immediately and persist to local JSON; opened via gear button, tray menu "Settings", or `Ctrl+Shift+,`
- **Theme & Language**: Dark/light theme toggle; Chinese/English interface switching; Rust enum types ensure compile-time safety
- **Data Export**: One-click export to Downloads as `.pake-data-YYYYMMDD.zip` with `manifest.json`; supports custom save path via native file dialog (`rfd`)
- **Data Import**: Reads ZIP archives with preview confirmation; existing files backed up (`.bak`) before overwrite; supports custom file selection via native file dialog
- **System Diagnostics**: Collects app version, git commit (truncated 8 chars), build time, rustc version, target triple, OS, CPU cores, memory, disk space; one-click copy report to clipboard
- **Version Backup**: Rotates up to 5 historical versions on each save; startup health check auto-restores from latest backup if config is corrupted; Restore button per version
- **Desktop Shortcut**: `pake.exe` can be launched directly via double-click or `.lnk` shortcut placed on desktop; `start.bat` batch file (UTF-8 BOM) available as fallback

## Tech Stack

- **Backend**: Rust (Tauri v2 IPC framework), 67 unit tests all passing
- **Frontend**: TypeScript (CLI) + vanilla JavaScript (WebView injection), 297 tests all passing
- **Shell**: System WebView (Windows: WebView2, macOS: WKWebView, Linux: WebKitGTK)
- **Storage**: JSON config files + SQLite WAL (clipboard history)
- **Key Crates**: tauri 2.10, tauri-plugin-http, clipboard-rs, rusqlite, sha2, regex, url, rfd, zip, sysinfo, arboard, chrono, built, serde, tokio

## Quick Start

```bash
# Install pnpm
npm install -g pnpm

# Install dependencies
pnpm install

# Build CLI
pnpm run cli:build

# Dev mode (run directly)
cd src-tauri
cargo run --no-default-features --features "custom-protocol,clipboard"

# Or double-click desktop shortcut
# Desktop -> Pake Plus.lnk

# CLI packaging with all features
node dist/cli.js https://github.com --name MyApp \
  --block-ads --adblock-rules ./my-rules.txt \
  --cache --cache-size 500 \
  --clipboard --clipboard-max 5000 \
  --show-system-tray
```

## CLI Options

| Option                     | Description                      | Default |
| -------------------------- | -------------------------------- | ------- |
| `--name <string>`          | App name                         | -       |
| `--icon <path>`            | App icon                         | -       |
| `--width <number>`         | Window width                     | 1200    |
| `--height <number>`        | Window height                    | 780     |
| `--show-system-tray`       | Show system tray                 | false   |
| `--block-ads`              | Enable ad/tracker blocking       | false   |
| `--adblock-rules <path>`   | Custom adblock rules file        | -       |
| `--cache`                  | Enable offline cache proxy       | false   |
| `--cache-size <number>`    | Cache size limit MB (50-1000)    | 200     |
| `--clipboard`              | Enable clipboard management      | false   |
| `--clipboard-max <number>` | Max clipboard records (500-5000) | 2000    |
| `--debug`                  | Debug build                      | false   |

## Project Structure

```
src-tauri/src/
├── adblock/              # Adblock engine
│   ├── mod.rs            # Module entry + AdblockState
│   ├── engine.rs         # URL matching engine
│   └── rules.rs          # EasyList rule parser
├── cache/                  # Offline cache engine
│   ├── mod.rs              # Module entry + CacheState
│   ├── engine.rs           # LRU eviction + disk storage
│   └── commands.rs         # IPC commands (cache_fetch, etc.)
├── app/
│   ├── clipboard/          # Clipboard management
│   │   ├── monitor.rs      # System clipboard monitor (FFI)
│   │   ├── store.rs        # SQLite storage & retrieval
│   │   ├── filter.rs       # Privacy filtering
│   │   ├── panel.rs        # History panel window
│   │   ├── commands.rs     # IPC commands
│   │   ├── settings.rs     # Clipboard-specific settings
│   │   ├── cleanup.rs      # Auto-cleanup
│   │   └── source.rs       # Source app identification
│   ├── settings/           # Settings & data management
│   │   ├── types.rs        # Data structures & enums
│   │   ├── traits.rs       # ModuleSettings trait
│   │   ├── io.rs           # JSON R/W & version backup
│   │   ├── health.rs       # Startup health check
│   │   ├── diagnostics.rs  # Diagnostics collection
│   │   └── commands.rs     # IPC commands
│   ├── setup.rs            # Tray menu & global shortcuts
│   └── window.rs           # Window creation & JS injection
├── inject/
│   ├── custom.js           # Settings panel, welcome page, 3 buttons
│   ├── adblock.js          # fetch/XHR interception + DOM hiding
│   ├── cache.js            # fetch/XHR proxy, offline detection, hit toast
│   └── event.js            # Keyboard shortcuts
└── lib.rs                  # App entry, 30+ IPC command registration
testcode/
├── test_adblock.ps1        # Adblock module test
├── test_cache.ps1          # Cache module test
├── test_settings.ps1       # Settings module test
├── test_all.ps1            # Full integration test (85 items)
└── README.md               # Screenshot checklist
start.bat                   # Desktop launcher (UTF-8 BOM)
```

## Test Results

| Type             | Count | Passed | Result     |
| ---------------- | ----- | ------ | ---------- |
| Rust tests       | 67    | 67     | All passed |
| JavaScript tests | 297   | 297    | All passed |
| cargo check      | -     | -      | 0 errors   |
| Integration test | 85    | 85     | All passed |

## Credits

This project is based on [Pake](https://github.com/tw93/Pake) (MIT License). See `latex/main.pdf` for the full experiment report.

## License

- Pake Plus additions: MIT License
- Original Pake code: MIT License, Copyright (c) 2023 Tw93
