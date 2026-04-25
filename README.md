# Links

本地链接管理工具 — 全平台桌面应用，用于管理个人书签和资源链接。

数据完全存储在本地（SQLite），无需账号、无需联网，你的链接只属于你。

## 设计理念

- **本地优先**：所有数据存储在本地 SQLite 数据库，不依赖任何云服务
- **轻量高效**：Tauri + Rust 后端，安装包约 5MB，内存占用极低
- **快速捕获**：全局快捷键 `Cmd+Shift+L`（macOS）/ `Ctrl+Shift+L`（Windows）随时唤起快速添加窗口
- **自动抓取**：粘贴 URL 后自动抓取页面标题、描述、favicon 和封面图
- **全文搜索**：基于 SQLite FTS5，支持中文和标签搜索，搜索结果高亮
- **安全可靠**：链接去重检测、失效检测，数据损坏自动重建

## 核心功能

| 功能 | 说明 |
|---|---|
| 链接管理 | 创建、编辑、删除，支持 URL / 标题 / 描述 / 备注 |
| 元数据抓取 | 自动抓取标题、描述、favicon、og:image（异步，不阻塞 UI），支持手动刷新 |
| 分组管理 | 树形分组，支持嵌套、创建 / 删除 / 收缩展开 |
| 标签系统 | 多标签关联，独立管理，模糊搜索，智能推荐，手动创建 / 删除 |
| 书签收藏 | 一键收藏，书签样式图标，星标筛选 |
| 全文搜索 | FTS5 实时搜索，结果高亮，同时搜索标签名，LIKE 降级兜底 |
| 链接分享 | 复制到剪贴板，支持 URL / Markdown / HTML 三种格式 |
| 去重检测 | URL 重复时自动提示，编辑时排除自身 |
| 失效检测 | 添加时自动检测可达性，卡片显示警告图标 |
| 书签导入 | 支持 Chrome / Firefox / Safari 书签 HTML 导入 |
| 数据导出 | JSON / Markdown / CSV，原生文件保存对话框 |
| 暗色模式 | CSS 变量驱动，持久化到 config.json |
| 全局快捷键 | 快速添加窗口，随时保存链接，支持自定义快捷键 |
| 系统托盘 | 最小化到托盘后台运行，左键点击显示主窗口 |
| 配置管理 | 窗口大小/位置记忆、关闭行为、暗色模式等，持久化到 config.json |
| 跨平台 | macOS / Windows / Linux |

## 技术栈

| 层 | 技术 | 说明 |
|---|---|---|
| 后端 | **Tauri 2.x** (Rust) | 原生窗口 + 系统集成 |
| 前端 | **Svelte 5** + **TailwindCSS 4** | 响应式 UI，Vite 构建 |
| 数据库 | **SQLite** (rusqlite bundled) | 本地存储，FTS5 全文搜索 |
| 网络 | **reqwest** (rustls-tls) | 元数据抓取，无系统依赖 |
| 构建 | **Vite 6** + **Cargo** | 前端热重载 + Rust 编译 |

## 项目架构

```
links/
├── .github/workflows/     # CI/CD：自动构建 Windows 安装包
│   └── release.yml
│
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── main.rs         # 入口，调用 app_lib::run()
│   │   ├── lib.rs          # Tauri builder：插件、命令注册、数据库初始化、全局快捷键、系统托盘
│   │   ├── db.rs           # SQLite 建表/迁移、数据模型、Db(Mutex<Connection>) 封装、CRUD、搜索、导出
│   │   ├── fetcher.rs      # URL 元数据异步抓取、Windows 系统代理
│   │   ├── commands.rs     # 所有 Tauri 命令、快捷键管理、剪贴板、去重/失效检测
│   │   ├── config.rs       # config.json 读写、Config(Mutex<HashMap>) 封装
│   │   └── normalize.rs    # URL 标准化（去重比较用）
│   ├── Cargo.toml          # Rust 依赖：tauri, rusqlite, reqwest, scraper, tokio...
│   ├── tauri.conf.json     # 窗口、构建、打包配置（主窗口 + quick-add 窗口）
│   └── capabilities/       # Tauri 权限配置
│
├── src/                    # Svelte 前端
│   ├── main.js             # 主窗口挂载入口
│   ├── quick-add.js        # 快速添加窗口入口
│   ├── App.svelte          # 主布局（侧边栏 + 内容区 + FAB）
│   ├── app.css             # CSS 变量设计系统 + 明暗主题
│   ├── lib/
│   │   ├── api.js          # Tauri invoke 封装（全部后端命令的 JS 绑定）
│   │   ├── stores/index.js # Svelte stores（links / categories / tags）
│   │   └── components/     # UI 组件
│   │       ├── LinkCard.svelte      # 链接卡片
│   │       ├── LinkForm.svelte      # 链接编辑/新增表单
│   │       ├── LinkList.svelte      # 链接列表
│   │       ├── SearchBar.svelte     # 搜索栏
│   │       ├── Sidebar.svelte       # 侧边栏（分组 + 标签）
│   │       ├── TagInput.svelte      # 标签输入
│   │       ├── ExportDialog.svelte  # 导出对话框
│   │       └── SettingsDialog.svelte # 设置对话框
│   └── pages/
│       └── QuickAdd.svelte # 快速添加页面
│
├── index.html              # 主窗口 HTML（含启动动画）
├── quick-add.html          # 快速添加窗口 HTML
├── package.json            # 前端依赖与脚本
├── vite.config.js          # Vite 配置（双入口：main + quick-add）
├── svelte.config.js        # Svelte 预处理配置
├── tsconfig.json           # TypeScript 严格模式配置
├── generate-icons.py       # 图标生成脚本
└── TODO.md                 # 开发进度跟踪
```

### 数据流

```
组件 → store 方法 → api.js → invoke() → Rust command → Db / HTTP
```

前端通过 `api.js` 统一封装所有 `invoke()` 调用，store 层管理状态和缓存，组件只与 store 交互。

### 数据库

| 表 | 说明 |
|---|---|
| `links` | URL、标题、描述、备注、分组、收藏、失效标记 |
| `categories` | 分组，支持树形嵌套（parent_id） |
| `tags` | 标签，独立管理，不随链接删除 |
| `link_tags` | 多对多关联表 |
| `links_fts` | FTS5 虚拟表（触发器自动同步） |

`links` 表的 `is_broken` 字段标记链接失效状态，由后台异步检测更新。

### 后端命令

| 命令 | 用途 |
|---|---|
| `links_list` | 分页查询（支持分组/标签/收藏/未标签筛选） |
| `links_create` | 创建链接 + 后台异步抓取元数据 + 可达性检测 |
| `links_update` | 部分更新（`category_id: -1` 清除分组） |
| `links_delete` | 删除链接 |
| `links_search` | FTS5 全文搜索 + 标签名匹配 + LIKE 降级 |
| `copy_to_clipboard` | 复制内容到系统剪贴板 |
| `check_duplicate` | URL 去重检测（支持排除自身） |
| `check_link_status` | 链接可达性检测（HEAD → GET 降级） |
| `categories_list` | 树形结构 |
| `categories_create/update/delete` | 分组 CRUD |
| `tags_list/create/delete` | 标签 CRUD |
| `tags_autocomplete` | 标签模糊搜索 |
| `export_links` | 导出为 JSON / Markdown / CSV |
| `import_bookmarks` | 导入浏览器书签 HTML |
| `open_url` | 在默认浏览器中打开 |
| `save_file` | 原生文件保存对话框 |
| `fetch_metadata` | 手动触发元数据抓取 |
| `get_setting / set_setting` | 读写配置项 |
| `get_shortcut / set_shortcut` | 快捷键管理 |
| `exit_app` | 安全退出应用 |

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
cd src-tauri && cargo test   # 运行 Rust 单元测试
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

### CI/CD

项目配置了 GitHub Actions 自动构建（`.github/workflows/release.yml`）：

- 触发条件：推送到 `main` 分支或手动触发
- 构建平台：Windows（当前仅 Windows）
- 产出：自动创建 GitHub Release Draft，附带安装包附件
- 发布流程：Draft 需手动确认后发布

如需支持 macOS / Linux 自动构建，需添加对应的 CI Runner。

## 数据存储

所有数据（数据库、配置、日志）存放在同一目录：

| 平台 | 路径 |
|---|---|
| macOS | `~/Library/Application Support/com.links.desktop/` |
| Windows | `%APPDATA%\com.links.desktop\` |
| Linux | `~/.local/share/com.links.desktop/` |

| 文件 | 说明 |
|---|---|
| `links.db` | SQLite 数据库（WAL 模式，损坏后删除重启自动重建） |
| `config.json` | 配置文件（窗口大小/位置、暗色模式、关闭行为、快捷键等） |
| `links.log` | 应用日志（自动轮转，单文件上限 10MB） |
| `fail_links.log` | 元数据抓取失败日志（含 URL、错误信息、代理状态） |

## 调试

### DevTools

macOS: `Cmd+Option+I`，Windows/Linux: `Ctrl+Shift+I`

### Console 直接调用后端

```javascript
const { invoke } = window.__TAURI__.core;
await invoke("links_list", { params: {} });
await invoke("links_search", { params: { query: "关键词" } });
await invoke("categories_list");
await invoke("get_setting", { key: "close-behavior" });
```

### Rust 日志

日志文件位于数据存储目录（见上方），文件名为 `links.log`。

### 常见问题

**端口 1420 被占用**：`lsof -i :1420` 找到进程后 `kill -9 <PID>`

**Rust 编译失败**：`cd src-tauri && cargo check 2>&1`，依赖下载失败重试，版本冲突删除 `Cargo.lock`

**快捷键无效**：检查 `capabilities/default.json` 是否包含 `global-shortcut:allow-register`

**白屏**：确认 Vite 在 `localhost:1420` 运行，DevTools 查看报错

**数据库损坏**：删除 `links.db`（含 `-wal` `-shm`）后重启自动重建

**元数据抓取失败**：检查 `fail_links.log`，Windows 用户确认系统代理设置

## License

MIT
