<h4 align="right"><strong>English</strong> | <a href="README_CN.md">简体中文</a></h4>
<p align="center">
    <img src=https://gw.alipayobjects.com/zos/k/fa/logo-modified.png width=138/>
</p>
<h1 align="center">Pake Plus</h1>
<p align="center"><strong>Enhanced desktop app builder based on Pake —— Adblock · Cache · Clipboard · Settings</strong></p>

## Features

### 🛡 Adblock

Dual-layer ad filtering powered by EasyList rules, operating at both the network request level and DOM element level:

- **Network Interception**: Registers Tauri resource request interceptors to match every HTTP/HTTPS request against rule sets, blocking blacklisted resources at the source
- **DOM Hiding**: Extracts CSS selectors from rules and injects them as hide styles, with MutationObserver monitoring for dynamically inserted elements
- **Custom Rules**: User-defined filtering rules, one per line, using `||domain^` (block) or `##.selector` (hide) format
- **Statistics**: Real-time block count displayed on the tray icon

### 💾 Offline Cache

HTTP-layer transparent proxy that caches page resources locally, enabling offline browsing of previously visited pages:

- **Transparent Proxy**: Intercepts all GET requests; cached and valid responses are served directly without network access
- **LRU Eviction**: Automatically purges least-recently-used files when cache exceeds the limit (default 200MB, adjustable 50-1000MB)
- **Cache Index**: Maintains URL-hash-to-file-path mapping for fast lookup and hit-rate statistics
- **Offline Mode**: Automatically displays a list of accessible cached pages when disconnected

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

### ⚙ Settings Panel

A unified control center managing all module configurations:

- **Visual Configuration**: Right-side sliding drawer with six tabs (General / Adblock / Cache / Clipboard / Data / About); changes apply immediately and persist to local JSON
- **Theme & Language**: Light, dark, and system-follow themes; Chinese/English interface switching; Rust enum types ensure compile-time safety for Theme and Language values
- **Data Export**: One-click directory scan and packaging into `.pake-data.zip` with a `manifest.json` file listing, enabling cross-device migration
- **Data Import**: Reads ZIP archives, previews contents before confirming import; existing files are backed up (`.bak`) before being overwritten, with rollback support
- **System Diagnostics**: Collects app version, git commit, build time, rustc version, target platform, OS info, CPU cores, memory usage, and disk space; one-click copy of the full diagnostics report to clipboard
- **Version Backup**: Automatically rotates up to 5 historical versions before each save; startup health checks can restore from backups if the config file is corrupted

## Tech Stack

- **Backend**: Rust (Tauri v2 IPC framework), 67 unit tests all passing
- **Frontend**: TypeScript (CLI) + vanilla JavaScript (WebView injection), 297 tests all passing
- **Shell**: System WebView (Windows: WebView2, macOS: WKWebView, Linux: WebKitGTK)
- **Storage**: JSON config files + SQLite WAL (clipboard history)
- **Key Crates**: clipboard-rs, rusqlite, sha2, regex, zip, sysinfo, arboard, chrono, built

## Quick Start

```bash
# Install pnpm
npm install -g pnpm

# Install dependencies
pnpm install

# Build CLI
pnpm run cli:build

# Package an app (with clipboard management)
node dist/cli.js https://github.com --name MyApp --clipboard --clipboard-max 2000

# Package with all features
node dist/cli.js https://example.com --name MyApp \
  --block-ads \
  --cache --cache-size 500 \
  --clipboard --clipboard-max 2000 \
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
| `--clipboard`              | Enable clipboard management      | false   |
| `--clipboard-max <number>` | Max clipboard records (500-5000) | 2000    |
| `--debug`                  | Debug build                      | false   |

## Project Structure

```
src-tauri/src/
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
│   ├── custom.js           # Settings panel sidebar
│   ├── settings.js         # Clipboard test panel
│   └── event.js            # Keyboard shortcuts
└── lib.rs                  # App entry & command registration
```

## Test Results

| Type             | Count | Passed | Result     |
| ---------------- | ----- | ------ | ---------- |
| Rust tests       | 67    | 67     | All passed |
| JavaScript tests | 297   | 297    | All passed |
| cargo check      | -     | -      | 0 warnings |

## Credits

This project is based on [Pake](https://github.com/tw93/Pake) (MIT License). See `latex/main.pdf` for the full experiment report.

## License

- Pake Plus additions: MIT License
- Original Pake code: MIT License, Copyright (c) 2023 Tw93
