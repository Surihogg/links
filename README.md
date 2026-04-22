# Links

本地链接管理工具 — 全平台桌面应用，用于管理个人书签和资源链接。

数据完全存储在本地（SQLite），无需账号、无需联网，你的链接只属于你。

## 设计理念

- **本地优先**：所有数据存储在本地 SQLite 数据库，不依赖任何云服务
- **轻量高效**：Tauri + Rust 后端，安装包约 5MB，内存占用极低
- **快速捕获**：全局快捷键 `Cmd+Shift+L`（macOS）/ `Ctrl+Shift+L`（Windows）随时唤起快速添加窗口
- **自动抓取**：粘贴 URL 后自动抓取页面标题、描述、favicon 和封面图
- **全文搜索**：基于 SQLite FTS5，支持中文和标签搜索，搜索结果高亮

## 核心功能

| 功能 | 说明 |
|---|---|
| 链接管理 | 创建、编辑、删除，支持 URL / 标题 / 描述 / 备注 |
| 元数据抓取 | 自动抓取标题、描述、favicon、og:image（异步，不阻塞 UI） |
| 分组管理 | 树形分组，支持嵌套、创建 / 删除 / 收缩展开 |
| 标签系统 | 多标签关联，独立管理，模糊搜索，手动创建 / 删除 |
| 特别关注 | 一键收藏，星标筛选 |
| 全文搜索 | FTS5 实时搜索，结果高亮，同时搜索标签名 |
| 数据导出 | JSON / Markdown / CSV，原生文件保存对话框 |
| 暗色模式 | CSS 变量驱动，持久化到 localStorage |
| 全局快捷键 | 快速添加窗口，随时保存链接 |
| 跨平台 | macOS / Windows / Linux |

## 技术栈

| 层 | 技术 | 说明 |
|---|---|---|
| 后端 | **Tauri 2.x** (Rust) | 原生窗口 + 系统集成 |
| 前端 | **Svelte 5** + **TailwindCSS 4** | 响应式 UI，Vite 构建 |
| 数据库 | **SQLite** (rusqlite bundled) | 本地存储，FTS5 全文搜索 |
| 网络 | **reqwest** (rustls-tls) | 元数据抓取，无系统依赖 |

## 项目架构

```
links/
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── main.rs         # 入口，调用 app_lib::run()
│   │   ├── lib.rs          # Tauri builder：插件、命令注册、数据库初始化、全局快捷键
│   │   ├── db.rs           # SQLite 建表/迁移、数据模型、Db(Mutex<Connection>) 封装
│   │   ├── fetcher.rs      # URL 元数据异步抓取
│   │   └── commands.rs     # 所有 Tauri 命令
│   ├── Cargo.toml
│   ├── tauri.conf.json     # 窗口、构建、打包配置
│   └── capabilities/       # 权限配置
│
├── src/                    # Svelte 前端
│   ├── main.js             # 挂载入口
│   ├── App.svelte          # 主布局（侧边栏 + 内容区 + FAB）
│   ├── app.css             # CSS 变量设计系统 + 明暗主题
│   ├── lib/
│   │   ├── api.js          # Tauri invoke 封装
│   │   ├── stores/index.js # Svelte stores（links / categories / tags）
│   │   └── components/     # UI 组件
│   └── pages/
│       └── QuickAdd.svelte # 快速添加页面
│
├── AGENTS.md               # AI 开发指南
└── README.md
```

### 数据流

```
组件 → store 方法 → api.js → invoke() → Rust command → Db / HTTP
```

### 数据库

| 表 | 说明 |
|---|---|
| `links` | URL、标题、描述、备注、分组、收藏 |
| `categories` | 分组，支持树形嵌套（parent_id） |
| `tags` | 标签，独立管理，不随链接删除 |
| `link_tags` | 多对多关联表 |
| `links_fts` | FTS5 虚拟表（触发器自动同步） |

### 后端命令

| 命令 | 用途 |
|---|---|
| `links_list` | 分页查询（支持分组/标签/收藏筛选） |
| `links_create` | 创建链接 + 异步抓取元数据 |
| `links_update` | 部分更新（`category_id: -1` 清除分组） |
| `links_delete` | 删除链接 |
| `links_search` | FTS5 全文搜索 + 标签名匹配 + LIKE 降级 |
| `categories_list` | 树形结构 |
| `categories_create/update/delete` | 分组 CRUD |
| `tags_list/create/delete` | 标签 CRUD |
| `tags_autocomplete` | 标签模糊搜索 |
| `export_links` | 导出为 JSON / Markdown / CSV |
| `open_url` | 在默认浏览器中打开 |
| `save_file` | 原生文件保存对话框 |
| `fetch_metadata` | 手动触发元数据抓取 |

## 快速开始

### 环境要求

| 工具 | 版本 | 安装 |
|---|---|---|
| Rust | ≥ 1.77.2 | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Node.js | ≥ 18 | [nodejs.org](https://nodejs.org) |
| npm | ≥ 9 | 随 Node.js 安装 |

macOS 需要安装 `xcode-select --install`，Windows 需要 [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)。

### 安装与启动

```bash
git clone https://github.com/Surihogg/links.git && cd links
npm install
npm run tauri dev
```

### 常用命令

```bash
npm run tauri dev          # 开发模式（Rust + 前端热重载）
npm run dev                # 仅前端（浏览器预览，后端不可用）
npm run build              # 仅构建前端
cd src-tauri && cargo check  # 快速检查 Rust 语法
```

## 打包

```bash
npm run tauri build
```

| 平台 | 产出 |
|---|---|
| macOS | `bundle/macos/Links.app`、`bundle/dmg/Links_*.dmg` |
| Windows | `bundle/msi/Links_*.msi`、`bundle/nsis/Links_*-setup.exe` |
| Linux | `bundle/debian/*.deb`、`bundle/appimage/*.AppImage` |

`rusqlite` 使用 bundled 自带 SQLite，`reqwest` 使用 rustls-tls 无需系统 OpenSSL。只能在当前平台构建当前平台的包。

## 数据存储

| 平台 | 路径 |
|---|---|
| macOS | `~/Library/Application Support/com.links.desktop/links.db` |
| Windows | `%APPDATA%\com.links.desktop\links.db` |
| Linux | `~/.local/share/com.links.desktop/links.db` |

WAL 模式，删除后重启自动重建。

## 调试

### DevTools

macOS: `Cmd+Option+I`，Windows/Linux: `Ctrl+Shift+I`

### Console 直接调用后端

```javascript
const { invoke } = window.__TAURI__.core;
await invoke("links_list", { params: {} });
await invoke("links_search", { query: "关键词" });
await invoke("categories_list");
```

### Rust 日志

| 平台 | 位置 |
|---|---|
| macOS | `~/Library/Logs/com.links.desktop/` |
| Windows | `%APPDATA%\com.links.desktop\logs\` |
| Linux | `~/.local/share/com.links.desktop/logs/` |

### 常见问题

**端口 1420 被占用**：`lsof -i :1420` 找到进程后 `kill -9 <PID>`

**Rust 编译失败**：`cd src-tauri && cargo check 2>&1`，依赖下载失败重试，版本冲突删除 `Cargo.lock`

**快捷键无效**：检查 `capabilities/default.json` 是否包含 `global-shortcut:allow-register`

**白屏**：确认 Vite 在 `localhost:1420` 运行，DevTools 查看报错

**数据库损坏**：删除 `links.db`（含 `-wal` `-shm`）后重启自动重建

## License

MIT
