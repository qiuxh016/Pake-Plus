<h4 align="right"><a href="README.md">English</a> | <strong>简体中文</strong></h4>
<p align="center">
    <img src=https://gw.alipayobjects.com/zos/k/fa/logo-modified.png width=138/>
</p>
<h1 align="center">Pake Plus</h1>
<p align="center"><strong>基于 Pake 的网页桌面应用增强工具 —— 广告拦截 · 剪贴板管理 · 可视化设置</strong></p>

## 功能

### 🛡 广告拦截

基于 EasyList 规则引擎，在**网络请求层**和 **DOM 元素层**实现双重广告过滤：

- **网络拦截**：注册 Tauri 资源请求拦截器，对每个 HTTP/HTTPS 请求进行规则匹配，命中黑名单的直接拒绝，从源头阻止广告资源加载
- **DOM 隐藏**：从规则集中提取 CSS 选择器，注入页面作为隐藏样式，使用 MutationObserver 监听动态插入元素，实时隐藏广告内容
- **自定义规则**：支持用户添加自定义过滤规则，每行一条，格式为 `||域名^`（阻止）或 `##.选择器`（隐藏）
- **拦截统计**：托盘图标实时显示拦截计数

### 💾 离线缓存

基于 HTTP 层透明代理实现页面资源的本地缓存，让已访问页面在断网时仍然可浏览：

- **透明代理**：拦截所有 GET 请求，命中本地缓存且未过期的直接返回缓存数据，无需网络请求
- **LRU 淘汰**：缓存总量超过上限时自动清理最少使用的文件，默认上限 200MB，可在 50-1000MB 之间调节
- **缓存索引**：维护 URL 哈希到文件路径的映射，支持快速查找和统计命中率
- **离线模式**：断网时自动展示可访问的已缓存页面列表

### 📋 剪贴板管理

系统级剪贴板监听，自动记录文本变化，支持全文搜索和一键复用：

- **系统监听**：通过 Rust FFI 调用操作系统原生 API（Windows `AddClipboardFormatListener`、macOS `NSPasteboard`、Linux X11 selection），独立线程运行不阻塞主界面
- **SHA-256 去重**：检测到文本变化后计算哈希值去重，结合双重判断消除重复通知和回写干扰
- **历史面板**：`Ctrl+Shift+V` 打开 320×480 悬浮面板，按时间倒序展示历史记录，每条记录显示内容预览、时间和来源应用
- **全文搜索**：支持中英文模糊匹配，多关键词空格分隔 AND 逻辑，120ms 内实时返回结果
- **一键复用**：点击历史记录即可写回系统剪贴板，显示 3 秒提示，支持展开/收起长文本（超过 100 字符）
- **隐私过滤**：自动跳过短文本（<2 字符）、超长文本（>10,000 字符）、疑似密码（6-30 位字母数字组合）和银行卡号（13-19 位 Luhn 校验），支持按来源应用忽略
- **自动清理**：默认保留 2000 条记录和 30 天，超出后按 500 条批次删除最早记录，每小时后台清理一次
- **数据持久化**：SQLite WAL 模式存储，唯一哈希索引实现 UPSERT 更新（相同内容只刷新时间不重复写入）

### ⚙ 设置面板

统一的控制中心，管理所有模块的配置：

- **可视化配置**：右侧滑出抽屉面板，六个标签页（一般 / 广告拦截 / 离线缓存 / 剪贴板 / 数据管理 / 关于），修改即时生效并持久化到本地 JSON
- **主题与语言**：支持浅色/深色/跟随系统三种主题，中英文界面切换，Theme 和 Language 使用 Rust 枚举类型实现编译期类型安全
- **数据导出**：一键扫描数据目录打包为 `.pake-data.zip`，包含 manifest.json 文件清单，支持跨设备迁移
- **数据导入**：读取 ZIP 文件，预览内容后确认导入，现有文件先备份（`.bak`）后覆盖，支持回滚
- **系统诊断**：采集应用版本、Git 提交、编译时间、rustc 版本、目标平台、OS 信息、CPU 核心数、内存使用、磁盘空间等信息，一键复制诊断报告到剪贴板
- **版本备份**：保存设置前自动轮转保留最近 5 个历史版本，启动时健康检查自动从备份修复损坏文件

## 技术栈

- **后端**：Rust（Tauri v2 IPC 框架），67 个单元测试全部通过
- **前端**：TypeScript（CLI）+ 原生 JavaScript（WebView 注入），297 个测试全部通过
- **桌面壳**：系统 WebView（Windows: WebView2，macOS: WKWebView，Linux: WebKitGTK）
- **存储**：JSON 配置文件 + SQLite WAL（剪贴板历史）
- **关键 Crates**：clipboard-rs、rusqlite、sha2、regex、url、rfd、zip、sysinfo、arboard、chrono、built

## 快速开始

```bash
# 安装 pnpm
npm install -g pnpm

# 安装依赖
pnpm install

# 构建 CLI
pnpm run cli:build

# 打包应用（启用剪贴板管理）
node dist/cli.js https://github.com --name MyApp --clipboard --clipboard-max 2000

# 打包应用（启用广告拦截 + 剪贴板）
node dist/cli.js https://example.com --name MyApp \
  --block-ads --adblock-rules ./my-rules.txt \
  --clipboard --clipboard-max 2000 \
  --show-system-tray
```

## CLI 选项

| 选项                       | 说明                        | 默认值 |
| -------------------------- | --------------------------- | ------ |
| `--name <string>`          | 应用名称                    | -      |
| `--icon <path>`            | 应用图标                    | -      |
| `--width <number>`         | 窗口宽度                    | 1200   |
| `--height <number>`        | 窗口高度                    | 780    |
| `--show-system-tray`       | 显示系统托盘                | false  |
| `--block-ads`              | 启用广告/跟踪拦截           | false  |
| `--adblock-rules <path>`   | 自定义广告规则文件          | -      |
| `--clipboard`              | 启用剪贴板管理              | false  |
| `--clipboard-max <number>` | 剪贴板最大记录数 (500-5000) | 2000   |
| `--debug`                  | 调试构建                    | false  |

## 项目结构

```
src-tauri/src/
├── adblock/              # 广告拦截引擎
│   ├── mod.rs            # 模块入口 + AdblockState
│   ├── engine.rs         # URL 匹配引擎
│   └── rules.rs          # EasyList 规则解析器
├── app/
│   ├── clipboard/          # 剪贴板管理
│   │   ├── monitor.rs      # 系统剪贴板监听（FFI）
│   │   ├── store.rs        # SQLite 存储与检索
│   │   ├── filter.rs       # 隐私过滤
│   │   ├── panel.rs        # 历史面板窗口
│   │   ├── commands.rs     # IPC 命令
│   │   ├── settings.rs     # 剪贴板专用设置
│   │   ├── cleanup.rs      # 自动清理
│   │   └── source.rs       # 来源应用识别
│   ├── settings/           # 设置面板与数据管理
│   │   ├── types.rs        # 数据结构与枚举
│   │   ├── traits.rs       # ModuleSettings trait
│   │   ├── io.rs           # JSON 读写与版本备份
│   │   ├── health.rs       # 启动健康检查
│   │   ├── diagnostics.rs  # 诊断信息采集
│   │   └── commands.rs     # IPC 命令
│   ├── setup.rs            # 托盘菜单与全局快捷键
│   └── window.rs           # 窗口创建与 JS 注入
├── inject/
│   ├── custom.js           # 设置面板侧边栏
│   ├── adblock.js          # fetch/XHR 拦截 + DOM 隐藏
│   ├── settings.js         # 剪贴板测试面板
│   └── event.js            # 键盘快捷键处理
└── lib.rs                  # 应用入口与命令注册
```

## 测试结果

| 测试类型        | 数量 | 通过 | 结果       |
| --------------- | ---- | ---- | ---------- |
| Rust 后端测试   | 67   | 67   | 全部通过   |
| JavaScript 测试 | 297  | 297  | 全部通过   |
| cargo check     | -    | -    | 0 warnings |

## 开源说明

本项目基于 [Pake](https://github.com/tw93/Pake)（MIT License）进行二次开发，实验报告见 `latex/main.pdf`。

## 许可证

- Pake Plus 新增代码：MIT License
- 原 Pake 代码：MIT License，Copyright (c) 2023 Tw93
