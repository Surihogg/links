# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概览

Links 是本地优先的跨平台桌面链接管理器：Tauri 2.x / Rust 后端、Svelte 5 / Vite 前端、SQLite（rusqlite bundled）存储。单应用、单 Cargo 项目，不是 monorepo。详细文件索引见根目录及 `src/`、`src-tauri/` 下的 `AGENTS.md`；修改对应区域前应先阅读这些文档。

## 常用命令

```bash
npm install
npm run tauri dev          # 完整应用：启动 Vite、编译 Rust，支持热重载
npm run dev                # 仅浏览器预览前端，Tauri API / 后端不可用
npm run build              # 构建三个前端入口到 dist/
npm run tauri build        # 当前平台的完整安装包

npx oxlint src             # 使用项目已安装的 oxlint；没有 npm run lint 脚本
npx playwright install chromium  # 首次运行 E2E 前安装浏览器
npm run test:e2e           # 8 个真实浏览器测试，使用隔离的 Tauri IPC mock

(cd src-tauri && cargo check)
(cd src-tauri && cargo test)
(cd src-tauri && cargo test test_create_link_minimal)  # 按名称过滤单个测试
(cd src-tauri && cargo test db::tests::test_create_link_minimal -- --exact --nocapture)
```

Vite 端口固定为 `1420` 且 `strictPort: true`，占用时直接失败。Playwright 测试位于 `tests/e2e/`，通过 `tauriMock.js` 模拟 IPC，不读写真实数据库或配置。窗口、快捷键、托盘、导入导出等原生行为仍需按 `E2E_TEST_CHECKLIST.md` 手工验证，R1-R6 重构回归另见 `REFACTOR_E2E_CHECKLIST.md`。Rust DB 测试使用内存 SQLite，不依赖应用数据目录。

## 整体架构

### 一个 Rust 进程，三个独立 Webview

窗口在 `src-tauri/tauri.conf.json` 声明，并对应 `vite.config.js` 的三个 Rollup 入口：

- `main`：`index.html` → `src/main.js` → `App.svelte`，主列表、侧边栏和模态框。
- `quick-add`：`quick-add.html` → `pages/QuickAdd.svelte`，复用 `LinkForm mode="standalone"`。
- `spotlight`：`spotlight.html` → `pages/Spotlight.svelte`，全局搜索浮层。

三个窗口有独立 JS 上下文，不能依赖共享模块变量同步状态；跨窗口通信走 Tauri 事件。`themeStore.svelte.js` 负责主题初始化、持久化和 `theme-changed` 广播；`lib/lifecycle/eventListeners.js` 处理主窗口的数据刷新和异步元数据回推。主窗口 `onMount` 的启动流程拆在 `lib/lifecycle/`，新增监听器应注册到 `createCleanupManager()`。

`src-tauri/src/lib.rs` 是后端组合根：注册插件和命令，初始化 DB / Config，管理托盘、全局快捷键、单实例、deep-link、更新器及本地 HTTP 服务。Chrome MV3 扩展通过 `links://add` 将页面交给 QuickAdd；Bookmarklet 使用另一条独立的 `http_server.rs` 本地回环 HTTP + 随机令牌通道。

### 前后端调用链

```text
Svelte 组件 → lib/stores → lib/api.js → invoke()
            → commands.rs → db/* / fetcher / 系统 API
```

新增后端命令通常需要同时修改：`commands.rs` 的 `#[tauri::command]`、`lib.rs` 的 `invoke_handler!` 注册、`src/lib/api.js` 的 async 封装，以及需要该状态的 store。命令名使用动词在前（如 `create_link`）；JS invoke 参数键必须与 Rust 的 snake_case 参数一致。组件不得直接调用 `invoke()`。

三个 JS 入口都用 `ErrorBoundary.svelte` 包装页面；`lib/monitoring/errorMonitor.svelte.js` 捕获未处理异常并去重上报 `log_frontend_event`，`performance.js` 记录主窗口 / Spotlight 搜索及 LinkList 渲染耗时。监控数据仅写入本地 Performance API 与 `links.log`，不上传外部服务，也不记录搜索词内容。

`App.svelte` 负责筛选条件和视图编排，`linksStore` 负责分页、搜索及局部更新。复制、主题、时间、URL、分类树、IME 等共享逻辑已在 `lib/utils/` 或专用 store 中，修改前先查找并复用。

### 数据、搜索与后台任务

`Db` 是 `Mutex<rusqlite::Connection>`，开启 WAL 和外键；`impl Db` 分散在 `db/repository.rs`、`search.rs`、`export.rs` 等模块，模型与错误由 `db/mod.rs` 重导出。SQL 业务逻辑属于 `db/*`，不应写入 `commands.rs`。

- Schema 由 `PRAGMA user_version` 和 `migration.rs` 的 `MIGRATIONS` 驱动；只能在数组末尾追加新版本，已发布迁移不可修改。
- `links_fts` 是由 links 表触发器同步的 FTS5 external-content 表。搜索组合 FTS、标签、分类和 LIKE，FTS 失败时回退 LIKE。
- list / search / export 返回链接集合时必须用 `load_tags_for_links` 批量加载标签；单条 `load_tags_for_link` 只用于单条创建或更新结果。
- 元数据抓取与可达性检查在创建 / 更新后通过 `tauri::async_runtime::spawn` 后台执行，再用 Tauri 事件回推前端；不要在创建命令中同步抓取。
- `Config(Mutex<HashMap>)` 将设置持久化到 `config.json`，与 SQLite 数据分开；前端配置不得改用 `localStorage`。
- `normalize.rs` 的 URL 标准化目前有意未接入去重流程，避免擅自改变用户原始 URL。

## 关键约束

- **锁与 IO**：不得跨 `.await` 持有 `Mutex<Connection>`；不得持锁做文件 IO、HTTP 或长计算。导入应先解析文件，再进入短事务。
- **共享后端逻辑**：HTTP 客户端复用 `fetcher::build_http_client`；创建 / 更新的可达性检查复用 `check_and_update_link_reachability`，不要复制代理或检查逻辑。
- **前端样式**：颜色、遮罩、阴影使用 `app.css` 的 CSS 变量；复用全局 `.btn`、`.modal*`、`.icon-btn`、`.spinner*`、`.nav-item`，不要在组件中硬编码颜色或重复定义。
- **Svelte 5**：模块顶层使用 `$state` / `$derived` 时文件必须是 `.svelte.js` / `.svelte.ts`。表单 Enter 处理保留 `createImeGuard()`，避免输入法组词误提交。
- **类型纪律**：不使用 `as any` 或 `@ts-ignore` 抑制错误。当前 `tsconfig.json` 的 `include` 不覆盖普通 `.js`，不能把它当作全量前端类型检查。
- **平台行为**：不要移除 `src-tauri/src/main.rs` 的 `#![windows_subsystem = "windows"]`，否则 Windows 会出现额外控制台窗口。

## 提交、日志与发布

- 代码注释和提交信息使用中文；提交说明应交代改动原因及影响范围。
- 不直接提交到远程仓库。先在本地独立分支开发并测试；合并 / 提交前需通过测试并获得人工测试确认。
- 任务若来自 TODO 工作区，完成后按 `YYYY-MM-DD HH:MM:SS` 时间戳追加到 `TODO.md` 的 `## 开发日志`。
- 打 `vX.Y.Z` tag 发布时，同步刷新 `AGENTS.md`、`README.md`、`USER_GUIDE.md`。版本号按 `AGENTS.md` 在 `tauri.conf.json`、`Cargo.toml` / `Cargo.lock`、`package.json` 间保持一致，并检查 `package-lock.json`。
- 大规模重构合并前刷新根目录、`src-tauri/`、`src/` 三份 `AGENTS.md`；README / USER_GUIDE 仅在有对外可见变化时刷新。
- GitHub Actions 在 `v*` tag、main 分支或手动触发时构建 macOS（双架构）和 Windows 包，Release 默认为 Draft，需人工发布。
