# 任务 D — 设置面板 + 数据迁移 + 诊断 + 视频 实施计划

---

## 一、总体概览

D 同学负责四个独立模块，都不依赖 A/B/C 的代码：

| 模块          | 核心工作                                   | 新增文件数                       |
| ------------- | ------------------------------------------ | -------------------------------- |
| 设置面板      | HTML 页面 + Rust 读写 JSON + 托盘入口      | 2（settings.html + settings.rs） |
| 数据导出/导入 | ZIP 打包/解压 + 文件遍历                   | 并入 settings.rs                 |
| 系统诊断      | sysinfo 采集 + built 编译信息 + 剪贴板报告 | 并入 settings.rs                 |
| 视频制作      | 录制 + 剪辑（最后阶段）                    | 0（纯操作）                      |

**新增 Rust 依赖**（追加到 `Cargo.toml`）：

```toml
zip = "2.2"            # ZIP 打包/解压
sysinfo = "0.33"       # 系统信息采集
arboard = "3.4"        # 写入剪贴板（诊断报告）
chrono = "0.4"         # 时间格式化
```

**新增 build 依赖**（追加到 `[build-dependencies]`）：

```toml
built = "0.7"          # 编译期注入版本/git信息
```

---

## 二、实施步骤（按顺序执行）

### 第 1 步：加 Rust 依赖 + 改 build.rs

**说明**：先跑通编译，确保新 crate 能正常下载和构建。这是所有后续工作的基础。

**操作**：

1. 在 `Cargo.toml` 的 `[dependencies]` 末尾追加 `zip`、`sysinfo`、`arboard`、`chrono`
2. 在 `[build-dependencies]` 追加 `built`
3. 修改 `build.rs`，在 tauri_build 之后调用 `built::write_built_file()` —— 这会生成一个 `built.rs` 文件到 `OUT_DIR`，Rust 代码可以通过 `include!` 引入，获取版本号、git commit、编译时间等信息
4. `cargo check` 确保通过

**预期耗时**：30 分钟

---

### 第 2 步：写 `settings.rs` —— 纯数据层（JSON 读写）

**说明**：这是其他所有功能的基础。先把配置的读和写搞定，不涉及 UI。

**操作**：

1. 新建 `src-tauri/src/app/settings.rs`
2. 定义 `AppSettings` 结构体，聚合四个分类的设置项：

```
AppSettings
├── AdblockSettings   (enabled, custom_rules: Vec<String>)
├── CacheSettings     (enabled, max_size_mb, hit_rate_1h)
├── ClipboardSettings (enabled, max_records, retention_days, ignore_short)
└── GeneralSettings   (theme, language)
```

3. 每个子结构体实现 `Default` trait
4. 写两个函数：
   - `get_settings_path(app) -> PathBuf` —— 返回 `$APP_DATA_DIR/pake-settings.json`
   - `load_settings(path) -> AppSettings` —— 读 JSON，不存在则返回并写入默认值
   - `save_settings(path, settings)` —— 序列化写入
5. 在 `mod.rs` 中加 `pub mod settings;`

**为什么从这一步开始**：不涉及 Tauri command 注册，不涉及 UI，纯逻辑，最快跑通。

**预期耗时**：1 小时

---

### 第 3 步：注册 IPC 命令，让 JS 能读写设置

**说明**：把上一步的函数包装为 Tauri command，注册到 `lib.rs`，让前端能调用。

**操作**：

1. 在 `settings.rs` 中定义两个 `#[tauri::command]`：
   - `fn get_settings(app: AppHandle) -> AppSettings`
   - `fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String>`
2. 在 `lib.rs` 的导入区域，新增：
   ```rust
   use app::settings::{get_settings, save_settings};
   ```
3. 在 `lib.rs` 的 `generate_handler![]` 宏中，追加 `get_settings, save_settings`
4. `cargo build` 确认编译通过

**预期耗时**：30 分钟

---

### 第 4 步：写设置面板 HTML 页面

**说明**：这是一个纯前端工作。新建一个独立 HTML 文件，内嵌 CSS 和 JS，不依赖任何框架。

**操作**：

1. 新建 `src-tauri/assets/settings.html`
2. 布局：左侧导航栏 + 右侧内容区

```
┌──────────────────────────────────────────────┐
│  Pake Plus 设置                              │
├────────┬─────────────────────────────────────┤
│ 📡 广告 │  [广告拦截设置内容]               │
│ 💾 缓存 │                                    │
│ 📋 剪贴板│                                   │
│ 📦 数据 │                                    │
│ ℹ️ 关于 │                                    │
├────────┴─────────────────────────────────────┤
│                    [保存]  [取消]             │
└──────────────────────────────────────────────┘
```

3. 页面加载时调用 `invoke('get_settings')` 填充表单
4. 点击保存时调用 `invoke('save_settings', { settings })` 写入
5. 样式干净简洁，深色顶栏 + 白色内容区，无外部依赖

**可以先用硬编码假数据开发**，不依赖设置的实际字段是否完整。

**预期耗时**：2 小时

---

### 第 5 步：托盘菜单增加"设置"入口

**说明**：让用户能打开设置窗口。

**操作**：

1. 在 `settings.rs` 中写一个函数 `pub fn open_settings_window(app: &AppHandle)`：
   - 用 `WebviewWindowBuilder` 创建新窗口
   - url 指向 `assets/settings.html`
   - 窗口标题 "Pake Plus 设置"，大小 520×620，居中，不可缩放
2. 在 `setup.rs` 的托盘菜单中添加 `"settings"` 菜单项
3. 在 `setup.rs` 的 `on_menu_event` 中增加 `"settings"` 分支，调用 `open_settings_window`
4. `cargo build && cargo run` 验证：右键托盘 → 设置 → 窗口弹出

**预期耗时**：1 小时

---

### 第 6 步：实现数据导出功能

**说明**：这是 Rust 工作量较大的部分。遍历数据目录，打包为 ZIP。

**操作**：

1. 在 `settings.rs` 中新增 `#[command] fn export_data(app: AppHandle) -> Result<String, String>`
2. 函数逻辑：
   - 创建临时目录 `$TEMP/pake-export-temp/`
   - 收集每个模块的数据文件：
     - 设置文件：直接复制 `pake-settings.json`
     - 缓存目录：复制索引文件 `cache-index.json` + 最近的 50MB 缓存文件
     - 剪贴板数据库：复制 `clipboard-history.json`
   - 生成 `manifest.json`，记录导出时间、版本号、各模块文件清单
   - 用 `zip` crate 将临时目录打包为 `.pake-data.zip`
   - 清理临时目录
   - 弹出系统文件保存对话框，让用户选择保存路径
   - 返回保存路径
3. 注册命令到 `generate_handler![]`
4. 在设置面板 HTML 中，给"导出"按钮绑定点击事件，调用 `invoke('export_data')`

**注意**：A/B/C 的模块还没写，但你可以**只处理已存在的通用数据**（设置 JSON），后续等他们的存储格式确定后再补充路径。你的代码结构是模块化的，加路径只需加一行。

**预期耗时**：1.5 小时

---

### 第 7 步：实现数据导入功能

**说明**：解压 ZIP，校验，恢复文件。

**操作**：

1. 在 `settings.rs` 中新增 `#[command] fn import_data(app: AppHandle) -> Result<String, String>`
2. 函数逻辑：
   - 弹出系统文件选择对话框，限定 `.zip` 文件
   - 读取 ZIP，先找 `manifest.json`，解析检查格式和版本兼容性
   - 创建临时目录解压
   - 将文件按 manifest 中的路径映射恢复到数据目录
   - 现有同名文件先备份（加 `.bak` 后缀），恢复失败可回滚
   - 清理临时目录
   - 返回导入摘要字符串（如 "已恢复 3 项配置、50MB 缓存、892 条剪贴板历史"）
3. 注册命令
4. 在设置面板 HTML 中绑定"导入"按钮

**预期耗时**：1.5 小时

---

### 第 8 步：实现系统诊断信息采集

**说明**：用 `sysinfo` + `built` 采集系统和编译信息。

**操作**：

1. 在 `settings.rs` 中新增一个函数 `fn collect_diagnostics() -> Diagnostics`
2. 定义 `Diagnostics` 结构体，字段：
   ```rust
   struct Diagnostics {
       app_version: String,      // 来自 built 或 env!("CARGO_PKG_VERSION")
       git_commit: String,       // 来自 built
       build_time: String,       // 来自 built
       rustc_version: String,    // 来自 built
       target_triple: String,    // 来自 built
       os_name: String,          // 来自 sysinfo (System::name())
       os_version: String,       // 来自 sysinfo (System::os_version())
       cpu_cores: usize,         // 来自 sysinfo (System::cpu_count())
       total_ram_mb: u64,        // 来自 sysinfo
       used_ram_mb: u64,         // 来自 sysinfo
       disk_total_mb: u64,       // 来自 sysinfo (disk usage of /)
       disk_free_mb: u64,        // 来自 sysinfo
       pid: u32,                 // 来自 std::process::id()
       enabled_features: Vec<String>, // 如 ["adblock", "cache", "clipboard"]
   }
   ```
3. 实现时注意：`sysinfo::System` 需要先 `refresh_all()` 再读数据
4. 写 `#[command] fn get_diagnostics(app: AppHandle) -> Diagnostics`
5. 注册命令
6. 在设置面板"关于"标签页中调用并展示

**预期耗时**：1 小时

---

### 第 9 步：实现"复制诊断报告"

**说明**：将诊断信息格式化为纯文本，写入剪贴板。

**操作**：

1. 在 `settings.rs` 中新增 `#[command] fn copy_diagnostics_report(app: AppHandle) -> Result<(), String>`
2. 函数逻辑：
   - 调用 `collect_diagnostics()` 获取数据
   - 格式化为多行纯文本（见下方格式）
   - 用 `arboard::Clipboard::new()` 写入系统剪贴板
3. 报告格式：
   ```
   Pake Plus v1.0.0 (commit: a1b2c3d)
   built: 2026-07-16 14:30:00, rustc 1.93.0
   target: x86_64-pc-windows-msvc
   OS: Windows 11 Home China 24H2
   CPU: 12 cores, RAM: 8214/16384 MB
   Disk C: 46208/256000 MB free
   Enabled: adblock, cache, clipboard, data-export
   PID: 12345
   ```
4. 注册命令
5. 在设置面板"关于"标签页添加"复制诊断信息"按钮

**预期耗时**：30 分钟

---

### 第 10 步：联调 + 完善设置面板 UI

**说明**：此时所有 Rust 功能已就绪。把 HTML 的交互补齐，联调一遍。

**操作**：

1. 确保设置面板所有标签页正确显示
2. 逐项测试：修改 → 保存 → 重启应用 → 验证配置持久化
3. 测试导出/导入流程完整闭环
4. 测试诊断信息展示和复制
5. 处理异常情况（JSON 损坏、磁盘空间不足、ZIP 文件损坏）

**预期耗时**：1.5 小时

---

### 第 11 步：视频制作（等 A/B/C 功能完成后）

**说明**：3 分钟介绍视频。建议最后两天做，熟练后一次录完。

**操作**：

1. 准备好演示素材页面（一个有广告的页面、一个含图片的页面用于测试缓存、一个随手复制的场景）
2. 用 OBS 或系统自带录屏工具录制
3. 按文档中的分镜表顺序
4. 剪辑工具：剪映或 DaVinci Resolve（免费）
5. 导出 720p，控制文件在 50MB 以内

**视频中你的部分（30 秒）**：右键托盘 → 设置面板 → 浏览各标签页 → 导出数据 → 切换到关于页 → 复制诊断报告

**预期耗时**：录制 30 分钟 + 剪辑 1 小时

---

## 三、文件变更清单

### 新建文件

| 文件                             | 内容                                        |
| -------------------------------- | ------------------------------------------- |
| `src-tauri/src/app/settings.rs`  | 设置读写、数据导出/导入、诊断采集、IPC 命令 |
| `src-tauri/assets/settings.html` | 设置面板页面（内嵌 CSS + JS）               |

### 修改文件

| 文件                         | 修改内容                                  |
| ---------------------------- | ----------------------------------------- |
| `src-tauri/Cargo.toml`       | 追加 zip、sysinfo、arboard、chrono、built |
| `src-tauri/build.rs`         | 追加 built 调用                           |
| `src-tauri/src/app/mod.rs`   | 加 `pub mod settings;`                    |
| `src-tauri/src/lib.rs`       | use 导入 + `generate_handler![]` 追加命令 |
| `src-tauri/src/app/setup.rs` | 托盘菜单加 "Settings" 项 + 事件处理       |

**不会冲突**：这些文件是 A/B/C 也需要改的（`mod.rs`、`lib.rs`、`setup.rs`、`Cargo.toml`），但改的位置不重叠——各自加各自的 use、各自的 handler、各自的菜单项。合并时只需按行拼接。

---

## 四、可以独立完成的证据

| 步骤               | 是否需要等 A/B/C | 说明                                                                                             |
| ------------------ | ---------------- | ------------------------------------------------------------------------------------------------ |
| 1-3（环境+Rust层） | 否               | 纯 D 的代码                                                                                      |
| 4（HTML 页面）     | 否               | 可用假数据开发                                                                                   |
| 5（托盘入口）      | 否               | 不依赖 A/B/C 代码                                                                                |
| 6-7（导出导入）    | **部分**         | 目前只处理通用文件（settings.json），A/B/C 的缓存/剪贴板数据路径现在用占位符，后续加一行路径即可 |
| 8-9（诊断）        | 否               | 纯系统信息，不依赖任何模块                                                                       |
| 10（联调）         | 否               | 设置面板本身功能闭环                                                                             |
| 11（视频）         | 是               | 需要 A/B/C 功能完成后录他们的演示                                                                |

---

## 五、代码结构（settings.rs 大纲）

```
pub struct AppSettings { adblock, cache, clipboard, general }
pub struct AdblockSettings { ... }
pub struct CacheSettings { ... }
pub struct ClipboardSettings { ... }
pub struct GeneralSettings { ... }
pub struct Diagnostics { ... }

fn get_settings_path(app) -> PathBuf        // 工具函数
pub fn load_settings(app) -> AppSettings    // 工具函数
pub fn save_settings_file(app, settings)    // 工具函数

#[command] fn get_settings(app) -> AppSettings
#[command] fn save_settings(app, settings) -> Result<(), String>
#[command] fn export_data(app) -> Result<String, String>
#[command] fn import_data(app) -> Result<String, String>
#[command] fn get_diagnostics(app) -> Diagnostics
#[command] fn copy_diagnostics_report(app) -> Result<(), String>

pub fn open_settings_window(app)            // 创建设置窗口
pub fn collect_diagnostics() -> Diagnostics // 纯数据采集
```

**总 Rust 代码量预估**：300-400 行（参考：invoke.rs 约 209 行，setup.rs 约 177 行）
