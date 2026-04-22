# Links

本地链接管理工具 — 全平台桌面应用，用于管理个人书签和资源链接。

数据完全存储在本地（SQLite），无需账号、无需联网，你的链接只属于你。

## 设计理念

- **本地优先**：所有数据存储在本地 SQLite 数据库，不依赖任何云服务
- **轻量高效**：Tauri + Rust 后端，安装包约 5MB，内存占用极低
- **快速捕获**：全局快捷键 `Cmd+Shift+L`（macOS）/ `Ctrl+Shift+L`（Windows）随时唤起快速添加窗口，粘贴链接即可保存
- **自动抓取**：粘贴 URL 后自动抓取页面标题、描述、favicon 和封面图
- **全文搜索**：基于 SQLite FTS5 的全文搜索引擎，支持中文分词和标签搜索
- **暗色模式**：一键切换明暗主题，跟随系统偏好

## 核心功能

| 功能 | 说明 |
|---|---|
| 链接管理 | 创建、编辑、删除链接，支持 URL / 标题 / 描述 / 备注 |
| 元数据抓取 | 自动抓取页面标题、描述、favicon、og:image（异步，不阻塞 UI） |
| 分组管理 | 树形分组结构，支持嵌套，创建 / 删除 / 收缩展开 |
| 标签系统 | 多标签关联，标签独立管理，支持模糊搜索和手动创建 / 删除 |
| 特别关注 | 一键收藏重要链接，星标筛选 |
| 全文搜索 | FTS5 驱动的实时搜索，搜索结果高亮关键词，同时搜索标签名 |
| 数据导出 | 支持导出为 JSON / Markdown / CSV 格式，原生文件保存对话框 |
| 暗色模式 | CSS 变量驱动的明暗主题，持久化到 localStorage |
| 全局快捷键 | 快速添加窗口，随时随地保存链接 |
| 跨平台 | macOS / Windows / Linux，原生体验 |

## 技术栈

| 层 | 技术 | 说明 |
|---|---|---|
| 后端 | **Tauri 2.x** (Rust) | 原生窗口 + 系统集成 |
| 前端 | **Svelte 5** + **TailwindCSS 4** | 响应式 UI，Vite 构建 |
| 数据库 | **SQLite** (rusqlite bundled) | 本地存储，FTS5 全文搜索 |
| 网络 | **reqwest** (rustls-tls) | 元数据抓取，无 OpenSSL 依赖 |

## 项目架构

```
links/
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── main.rs         # 入口（windows_subsystem）
│   │   ├── lib.rs          # Tauri builder：插件、命令注册、数据库初始化、全局快捷键
│   │   ├── db.rs           # SQLite 建表/迁移、数据模型、Db(Mutex<Connection>) 封装
│   │   ├── fetcher.rs      # URL 元数据异步抓取（标题、描述、favicon、og:image）
│   │   └── commands.rs     # 所有 Tauri 命令（CRUD、搜索、导出）
│   ├── Cargo.toml
│   ├── tauri.conf.json     # Tauri 配置（窗口、构建、打包）
│   └── capabilities/       # 权限配置（全局快捷键、窗口拖拽等）
│
├── src/                    # Svelte 前端
│   ├── main.js             # Svelte 挂载入口
│   ├── App.svelte          # 主布局（侧边栏 + 内容区 + FAB）
│   ├── app.css             # CSS 变量设计系统 + Tailwind + 明暗主题
│   ├── lib/
│   │   ├── api.js          # Tauri invoke 封装（所有后端调用）
│   │   ├── stores/index.js # Svelte writable stores（links / categories / tags）
│   │   └── components/     # UI 组件
│   │       ├── Sidebar.svelte      # 侧边栏（导航、分组管理、标签管理）
│   │       ├── LinkCard.svelte     # 链接卡片（标题、描述、分组、标签、操作）
│   │       ├── LinkList.svelte     # 链接列表（加载态、空态）
│   │       ├── LinkForm.svelte     # 链接编辑表单（创建/编辑、URL 抓取）
│   │       ├── SearchBar.svelte    # 搜索栏（实时搜索、防抖）
│   │       ├── TagInput.svelte     # 标签输入（自动补全）
│   │       └── ExportDialog.svelte # 导出对话框（JSON/MD/CSV）
│   └── pages/
│       └── QuickAdd.svelte # 快速添加页面（全局快捷键唤起）
│
├── AGENTS.md               # AI 开发指南
└── DEBUG.md                # 调试与启动指南
```

### 数据流

```
组件 (Svelte 5)
  → store 方法 (stores/index.js)
    → api.js 函数
      → invoke("command_name", { params })
        → Rust #[tauri::command] (commands.rs)
          → Db 操作 / HTTP 抓取
```

### 数据库结构

```
links          ← 主表：URL、标题、描述、备注、分组、收藏
categories     ← 分组：支持树形嵌套（parent_id）
tags           ← 标签：独立管理，不随链接删除
link_tags      ← 关联表：多对多
links_fts      ← FTS5 虚拟表：全文搜索（触发器自动同步）
```

### 后端命令

| 命令 | 用途 |
|---|---|
| `links_list` | 分页查询链接（支持分组/标签/收藏筛选） |
| `links_create` | 创建链接 + 异步抓取元数据 |
| `links_update` | 更新链接字段（部分更新） |
| `links_delete` | 删除链接 |
| `links_search` | FTS5 全文搜索（同时搜索标签名） |
| `categories_list` | 列出分组（树形结构） |
| `categories_create` | 创建分组 |
| `categories_update` | 更新分组 |
| `categories_delete` | 删除分组 |
| `tags_list` | 列出所有标签 |
| `tags_create` | 创建标签 |
| `tags_delete` | 删除标签 |
| `tags_autocomplete` | 标签模糊搜索 |
| `export_links` | 导出为 JSON / Markdown / CSV |
| `open_url` | 在默认浏览器中打开链接 |
| `save_file` | 原生文件保存对话框 |
| `fetch_metadata` | 手动触发 URL 元数据抓取 |

## 快速开始

### 环境要求

| 工具 | 版本 | 安装 |
|---|---|---|
| Rust | ≥ 1.77.2 | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Node.js | ≥ 18 | [nodejs.org](https://nodejs.org) |
| npm | ≥ 9 | 随 Node.js 安装 |

**macOS 额外要求**：Xcode Command Line Tools（`xcode-select --install`）

**Windows 额外要求**：[Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

### 安装与启动

```bash
# 克隆项目
git clone <repo-url> && cd links

# 安装前端依赖
npm install

# 开发模式（推荐）
npm run tauri dev
```

首次启动较慢（Rust 编译），后续增量编译很快。前端修改自动热更新，Rust 修改触发重编译。

### 其他命令

```bash
npm run dev          # 仅前端开发（浏览器预览，后端调用不可用）
npm run build        # 仅构建前端
cd src-tauri && cargo check  # 快速检查 Rust 语法
```

## 打包发布

### macOS

```bash
npm run tauri build
```

产出文件：
- `src-tauri/target/release/bundle/macos/Links.app` — macOS 应用
- `src-tauri/target/release/bundle/dmg/Links_0.1.0_aarch64.dmg` — DMG 安装包（Apple Silicon）
- `src-tauri/target/release/bundle/dmg/Links_0.1.0_x64.dmg` — DMG 安装包（Intel）

### Windows

```bash
npm run tauri build
```

产出文件：
- `src-tauri/target/release/bundle/msi/Links_0.1.0_x64_en-US.msi` — MSI 安装包
- `src-tauri/target/release/bundle/nsis/Links_0.1.0_x64-setup.exe` — NSIS 安装包

### Linux

```bash
npm run tauri build
```

产出文件：
- `src-tauri/target/release/bundle/debian/*.deb`
- `src-tauri/target/release/bundle/appimage/*.AppImage`

### 跨平台构建注意事项

- `rusqlite` 使用 `bundled` 特性 — 自带 SQLite，无需系统安装
- `reqwest` 使用 `rustls-tls` — 无需系统 OpenSSL
- 在 macOS 上只能构建 macOS 安装包，Windows 同理。跨平台需要对应平台的 CI 或交叉编译环境

## 数据存储

| 平台 | 数据库路径 |
|---|---|
| macOS | `~/Library/Application Support/com.links.desktop/links.db` |
| Windows | `%APPDATA%\com.links.desktop\links.db` |
| Linux | `~/.local/share/com.links.desktop/links.db` |

数据库使用 WAL 模式，支持并发读取。删除数据库文件后重启应用会自动重建。

## License

MIT
