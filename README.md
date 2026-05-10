# Links

本地链接管理工具 — 全平台桌面应用，用于管理个人书签和资源链接。

数据完全存储在本地（SQLite），无需账号、无需联网，你的链接只属于你。

## 设计理念

- **本地优先**：所有数据存储在本地 SQLite 数据库，不依赖任何云服务
- **轻量高效**：Tauri + Rust 后端，安装包约 5MB，内存占用极低
- **快速捕获**：全局快捷键 `Cmd+Shift+L`（macOS）/ `Ctrl+Shift+L`（Windows）随时唤起快速添加窗口
- **全局搜索**：`Cmd+Shift+K`（macOS）/ `Ctrl+Shift+K`（Windows）唤起 Spotlight 搜索浮层，即时搜索并打开链接
- **自动更新**：内置更新检查，新版本发布后一键安装
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
| 书签导入 | 支持 Chrome / Firefox / Safari 书签 HTML + JSON + CSV 导入 |
| 数据导出 | JSON / Markdown / CSV / 浏览器书签 HTML，原生文件保存对话框 |
| 暗色模式 | CSS 变量驱动，持久化到 config.json |
| 全局快捷键 | 快速添加窗口，随时保存链接，支持自定义快捷键 |
| 系统托盘 | 最小化到托盘后台运行，左键点击显示主窗口 |
| 配置管理 | 窗口大小/位置记忆、关闭行为、暗色模式等，持久化到 config.json |
| 跨平台 | macOS / Windows / Linux |
| 全局搜索 | Spotlight 搜索浮层，快捷键唤起，即时搜索，键盘导航，Enter 打开 / Cmd+Enter 定位到主窗口 |
| 浏览器扩展 | Chrome 扩展一键收藏，通过深度链接唤起快速添加窗口 |
| 自动更新 | 启动检查 + 手动检查，Release Notes 弹窗，一键下载安装 |
| 键盘导航 | 主窗口上下键选中链接，回车打开，空格编辑；Spotlight 箭头导航 |
| 导入增强 | 支持 Chrome / Firefox / Safari 书签 HTML，以及 JSON / CSV 文件导入 |
| 导出增强 | JSON / Markdown / CSV / 浏览器书签 HTML 四种格式 |

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
├── .github/workflows/     # CI/CD：macOS + Windows 自动构建
│   └── release.yml
│
├── src-tauri/                       # Rust 后端
│   ├── src/
│   │   ├── main.rs                  # 入口，调用 app_lib::run()
│   │   ├── lib.rs                   # Tauri builder：插件、命令注册、数据库初始化、全局快捷键、系统托盘；deep-link + 单实例处理
│   │   ├── commands.rs              # 39 个 Tauri 命令、快捷键 helper、书签解析、剪贴板、去重/失效检测
│   │   ├── db/                      # 数据库层（R2 拆分自 db.rs）
│   │   │   ├── mod.rs               # Db(Mutex<Connection>) 封装 + open()
│   │   │   ├── error.rs             # AppError 统一错误类型
│   │   │   ├── models.rs            # Link / Category / Tag / *Payload / Stats
│   │   │   ├── migration.rs         # PRAGMA user_version 迁移框架（R5）
│   │   │   ├── row_mapping.rs       # 行映射 + 共享 SQL 工具 + load_tags_for_links 批量加载（R5）
│   │   │   ├── repository.rs        # 链接 / 分组 / 标签 CRUD 与列表
│   │   │   ├── search.rs            # FTS5 + 标签 + 分类 + LIKE 四路 UNION 搜索 + 统计
│   │   │   ├── export.rs            # JSON / Markdown / CSV / Netscape Bookmark HTML
│   │   │   └── tests.rs             # 30+ 个内嵌单元测试
│   │   ├── fetcher.rs               # URL 元数据异步抓取；build_http_client 共享 Windows 系统代理（R1）
│   │   ├── http_server.rs           # 浏览器扩展本地 HTTP 服务（端口 + 令牌鉴权）
│   │   ├── config.rs                # config.json 读写、Config(Mutex<HashMap>) 封装
│   │   └── normalize.rs             # URL 标准化（实现完整但未接入；详见 src-tauri/AGENTS.md）
│   ├── Cargo.toml                   # Rust 依赖：tauri, rusqlite, reqwest, scraper, tiny_http...
│   ├── tauri.conf.json              # 窗口、构建、打包配置（主窗口 + quick-add + spotlight 三窗口）
│   └── capabilities/                # Tauri 权限配置
│
├── src/                             # Svelte 前端
│   ├── main.js                      # 主窗口挂载入口
│   ├── quick-add.js                 # 快速添加窗口入口
│   ├── spotlight.js                 # 全局搜索窗口入口
│   ├── App.svelte                   # 主布局（侧边栏 + 内容区 + FAB + 各类模态装配）
│   ├── app.css                      # CSS 变量设计系统 + 全局公共样式（.btn/.modal/.icon-btn/.spinner/.nav-item）
│   ├── lib/
│   │   ├── api.js                   # Tauri invoke 封装（39 个后端命令的 JS 绑定，命名"动词在前"）
│   │   ├── ready.js                 # 等待后端就绪辅助
│   │   ├── stores/
│   │   │   ├── index.js             # linksStore / categoriesStore / tagsStore / settingsStore
│   │   │   └── themeStore.svelte.js # 主题状态 + 跨窗口同步（R3）
│   │   ├── utils/                   # 共享工具（R3+R4 抽出）
│   │   │   ├── time.js              # formatRelativeTime / formatAbsoluteTime
│   │   │   ├── url.js               # getDomain
│   │   │   ├── linkShare.js         # formatLinkAs（url/markdown/html）
│   │   │   ├── imeGuard.svelte.js   # createImeGuard（IME 输入法守卫）
│   │   │   ├── categoryTree.js      # findCategoryById / flattenCategories / isCategoryDescendant
│   │   │   ├── categoryDragHandler.svelte.js  # createCategoryDrag（Pointer Events 拖拽）
│   │   │   └── cyclePlaceholder.js  # placeholder 提示循环
│   │   └── components/              # UI 组件
│   │       ├── LinkCard.svelte      # 链接卡片
│   │       ├── LinkForm.svelte      # 链接编辑/新增表单（mode="modal" | "standalone"）
│   │       ├── LinkList.svelte      # 链接列表
│   │       ├── SearchBar.svelte     # 搜索栏
│   │       ├── Sidebar.svelte       # 侧边栏装配
│   │       ├── Sidebar/             # 侧边栏子组件（R4+R6）
│   │       │   ├── Brand.svelte     # 品牌头（图标 + 版本 + 更新铃 + 设置齿轮）
│   │       │   ├── CategorySection.svelte  # 分组节
│   │       │   └── TagSection.svelte       # 标签节
│   │       ├── TagInput.svelte      # 标签输入
│   │       ├── CategoryInput.svelte # 分组选择输入
│   │       ├── ExportDialog.svelte  # 导出对话框
│   │       ├── SettingsDialog.svelte # 设置对话框（4 套快捷键用 ShortcutEditor）
│   │       ├── ShortcutEditor.svelte # 单条快捷键展示/录制/保存（R3）
│   │       ├── StatsPanel.svelte    # 统计视图（R4）
│   │       ├── CloseDialog.svelte   # 关闭确认弹窗（R4）
│   │       ├── ReleaseNotesDialog.svelte # 升级后发布说明弹窗（R4）
│   │       ├── UpdateDialog.svelte  # 应用更新下载弹窗
│   │       └── SortSelect.svelte    # 排序下拉选择
│   └── pages/
│       ├── QuickAdd.svelte          # 快速添加（仅 186 行，复用 <LinkForm mode="standalone">）
│       └── Spotlight.svelte         # 全局搜索浮层
│
├── index.html                       # 主窗口 HTML（含启动动画）
├── quick-add.html                   # 快速添加窗口 HTML
├── spotlight.html                   # 全局搜索窗口 HTML
├── browser-extension/               # Chrome 浏览器扩展源码
├── package.json                     # 前端依赖与脚本
├── vite.config.js                   # Vite 配置（三入口：main + quick-add + spotlight）
├── svelte.config.js                 # Svelte 预处理配置
├── tsconfig.json                    # TypeScript 配置（strict + checkJs；详见 src/AGENTS.md）
├── generate-icons.py                # 图标生成脚本
├── REFACTOR_E2E_CHECKLIST.md        # R1-R6 重构端到端测试清单
├── E2E_TEST_CHECKLIST.md            # 全功能端到端测试清单
└── TODO.md                          # 开发进度跟踪 + 重构开发日志
```

### 数据流

```
组件 → store 方法 → api.js → invoke() → Rust command → Db / HTTP
                  ↘ themeStore（跨窗口主题同步事件）
```

前端通过 `api.js` 统一封装所有 `invoke()` 调用，store 层管理状态和缓存，组件只与 store 交互。
主题切换通过 `themeStore.setMode(...)` 触发，自动通过 Tauri `theme-changed` 事件同步到三个窗口。

### 数据库

| 表 | 说明 |
|---|---|
| `links` | URL、标题、描述、备注、分组、收藏、失效标记、点击次数、最后打开时间 |
| `categories` | 分组，支持树形嵌套（parent_id） |
| `tags` | 标签，独立管理，不随链接删除 |
| `link_tags` | 多对多关联表 |
| `links_fts` | FTS5 虚拟表（触发器自动同步） |

`links` 表的 `is_broken` 字段标记链接失效状态，由后台异步检测更新；
`click_count` / `last_opened_at` 用于"最多访问"和"最近打开"排序与统计 Top 3。

数据库 schema 通过 `PRAGMA user_version` 显式版本管理（见 `src-tauri/src/db/migration.rs`），
每个迁移用事务包裹、失败回滚；老库（无 user_version）首次启动自动幂等升级到 v1。

### 后端命令（39 个）

R2 重构后所有命令统一为「动词在前」命名风格。

| 命令 | 用途 |
|---|---|
| `list_links` | 分页查询（支持分组/标签/收藏/未标签/未分组筛选与排序） |
| `create_link` | 创建链接 + 后台异步抓取元数据 + 可达性检测 |
| `update_link` | 部分更新（`category_id: -1` 清除分组） |
| `delete_link` | 删除链接 |
| `search_links` | FTS5 + 标签名 + 分类名 + LIKE 四路 UNION，FTS 失败时回退 LIKE-only |
| `get_links_stats` | 统计：总数 + 本周新增 + Top 3 链接 |
| `check_duplicate` | URL 去重检测（支持排除自身） |
| `check_link_status` | 链接可达性检测（HEAD → GET 降级） |
| `list_categories` | 分组树形结构 |
| `create_category` / `update_category` / `delete_category` | 分组 CRUD（update 支持改名 + 改父级 + 清父级） |
| `list_tags` / `create_tag` / `update_tag` / `delete_tag` | 标签 CRUD |
| `autocomplete_tags` | 标签模糊搜索（按引用次数排序） |
| `fetch_metadata` | 手动触发元数据抓取 |
| `open_url` | 在默认浏览器中打开 |
| `open_data_dir` | 在文件管理器中打开数据目录 |
| `save_file` | 原生文件保存对话框 |
| `copy_to_clipboard` | 复制内容到系统剪贴板 |
| `export_links` | 导出为 JSON / Markdown / CSV / Netscape Bookmark HTML |
| `import_bookmarks` | 导入浏览器书签 HTML / JSON（解析与事务分离，避免持锁阻塞） |
| `get_setting` / `set_setting` | 读写配置项（持久化到 config.json） |
| `get_shortcut` / `set_shortcut` | 快速添加快捷键 |
| `get_main_shortcut` / `set_main_shortcut` | 主窗口快捷键 |
| `get_spotlight_shortcut` / `set_spotlight_shortcut` | Spotlight 快捷键 |
| `get_hide_shortcut` / `set_hide_shortcut` | 隐藏窗口快捷键 |
| `get_system_proxy` | 获取系统代理设置（Windows） |
| `pop_pending_deep_link` | 取出并清除待处理的深度链接数据 |
| `check_startup_deep_link` | 检查启动是否由深度链接触发 |
| `get_local_server_info` | 获取本地 HTTP 服务端口和令牌（供 Bookmarklet 用） |
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
cd src-tauri && cargo test   # 运行 Rust 单元测试（共 95 个）
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

- 触发条件：推送 `v*` tag 或手动触发
- 构建平台：macOS（aarch64 + x86_64）+ Windows
- 产出：自动创建 GitHub Release Draft，附带安装包附件
- Windows 构建附带浏览器扩展 zip 附件
- Release 为 Draft 模式，需手动确认发布

如需支持更多平台自动构建，需添加对应的 CI Runner。

## 数据存储

所有数据（数据库、配置、日志）存放在同一目录：

| 平台 | 路径                                                  |
|---|-----------------------------------------------------|
| macOS | `~/Library/Application\ Support/com.links.desktop/` |
| Windows | `%APPDATA%\com.links.desktop\`                      |
| Linux | `~/.local/share/com.links.desktop/`                 |

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
await invoke("list_links", { params: {} });
await invoke("search_links", { params: { query: "关键词" } });
await invoke("list_categories");
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

## 版本历史

### 未发布（v1.3.0 之上的 R1-R6 重构，2026-05-10）

仅代码内部结构与性能改进，**无对外可见行为变化**：

- ♻️ R1：修两处后端 bug — `fetcher.rs` Windows 代理解析 `&&`/`||` 优先级缺括号；`import_bookmarks` 持锁做文件 IO 阻塞其它 DB 命令
- ♻️ R1：抽 `update_shortcut` / `app_data_dir` / `build_http_client` / 单一 `ensure_tags`/`get_or_create_category`，消除 ~118 行后端样板
- ♻️ R2：拆 `db.rs` 1686 行 → `db/{models,migration,error,row_mapping,repository,search,export,tests}.rs` 9 个文件
- ♻️ R2：39 个 Tauri 命令统一为「动词在前」命名风格（`links_list` → `list_links` 等）
- ♻️ R3：抽前端共享 `lib/utils/`（time / url / linkShare / imeGuard）+ `lib/stores/themeStore`；消除 App / QuickAdd / Spotlight 三处 `apply_theme` 副本
- ♻️ R3：QuickAdd 由 518 行降至 186 行，复用 `<LinkForm mode="standalone">`
- ♻️ R3：抽 `ShortcutEditor.svelte`，SettingsDialog 中 4 套快捷键样板（~250 行重复）合并为元数据驱动
- ♻️ R3：8 处硬编码颜色替换为 CSS 变量（`--scrim-bg` / `--shadow-drag`）；`.icon-btn` / `.spinner` / `.spinner-sm` / `@keyframes spin` 提到 `app.css`
- ♻️ R4：App.svelte 抽 `StatsPanel` / `CloseDialog` / `ReleaseNotesDialog`（marked 依赖跟随转移）；Sidebar 抽 `Brand`
- ♻️ R4：抽 `categoryTree` / `categoryDragHandler` / `cyclePlaceholder` 工具
- ♻️ R5：DB schema 用 `PRAGMA user_version` 显式版本管理，老库幂等升级
- ♻️ R5：消除 list/search/export 路径的 N+1 标签查询，单页满载时 DB 查询次数从 31 降至 2
- ♻️ R6：Sidebar 拆 `CategorySection` / `TagSection`，主组件从 948 行降至 266 行
- ♻️ R6：`.nav-item` 提到 `app.css`，3 处共享
- 📊 测试：`cargo test` 90 → 95（迁移框架 +3、批量加载 +2）；前端构建产物 main bundle 142.5KB → 129.3KB（-9%）

### v1.2.0 (2026-05-04)
- ✨ 全局搜索（Spotlight）：`Cmd+Shift+K` 唤起搜索浮层，即时搜索，键盘导航
- ✨ 浏览器扩展：Chrome 扩展一键收藏链接
- ✨ 自动更新：启动检查 + 手动检查，Release Notes 弹窗
- ✨ 主窗口键盘导航：上下键选中，回车打开，空格编辑
- ✨ 导入增强：支持 JSON 文件导入，自动创建分组和标签
- ✨ 导出增强：支持导出为浏览器书签 HTML 格式
- ✨ 分组拖拽移动：支持拖拽调整分组顺序和层级
- ✨ 分组展开阶梯缩进 + 绿色指示条
- 🔧 搜索增强：支持按分组名搜索，筛选条件芯片
- 🔧 暗色模式下搜索高亮颜色适配
- 🔧 输入法回车适配（创建分组/标签时）
- 🔧 CI/CD 支持 macOS + Windows 双平台构建和签名

## License

MIT
