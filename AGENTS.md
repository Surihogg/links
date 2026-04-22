# Links - 本地链接管理工具

全平台桌面应用，用于管理个人书签和资源链接。

## 技术栈

- **Tauri 2.x**（Rust 后端 + WebView 前端，~5MB 体积）
- **Svelte 5** + **TailwindCSS 4**（Vite 构建）
- **SQLite**（rusqlite bundled）+ FTS5 全文搜索

## 命令

```bash
npm run dev              # 仅前端开发（浏览器预览，后端调用不可用）
npm run build            # 仅构建前端
npm run tauri dev        # 启动 Tauri 开发模式（Rust + 前端，推荐）
npm run tauri build      # 生产构建
```

仅检查 Rust（快速，不涉及前端）：
```bash
cd src-tauri && cargo check
```

## 架构

```
src-tauri/src/
  main.rs        # 入口（windows_subsystem），调用 app_lib::run()
  lib.rs         # Tauri builder：插件、命令注册、数据库初始化、全局快捷键
  db.rs          # SQLite 建表、数据模型（Link/Category/Tag/AppError）、Db 封装
  fetcher.rs     # URL 元数据抓取（标题、描述、favicon、og:image），限制 512KB
  commands.rs    # 所有 Tauri 命令（CRUD、搜索、导出）

src/
  App.svelte          # 主布局（侧边栏 + 链接列表 + 搜索），.dark class 在此 div 上
  main.js             # 挂载入口
  app.css             # CSS 变量设计系统（明暗主题）
  lib/api.js          # Tauri invoke 封装（所有后端调用）
  lib/stores/         # Svelte stores（links, categories, tags）
  lib/components/     # UI 组件
  pages/QuickAdd.svelte  # 快速添加页面（全局快捷键唤起的独立窗口）
```

## 关键模式

- **数据库**：`Db(Mutex<Connection>)` 作为 Tauri state 管理。WAL 模式 + 外键约束。启动时自动迁移建表（`CREATE TABLE IF NOT EXISTS`）。
- **FTS5 同步**：通过 `links` 表上的触发器（INSERT/UPDATE/DELETE）自动同步 `links_fts` 虚拟表。
- **元数据抓取**：`links_create` 先立即保存，然后 `tauri::async_runtime::spawn` 异步抓取。只填充空字段，不覆盖用户输入。响应体限制 512KB，标题 500 字符，描述 2000 字符。
- **分类筛选**：`ListLinksParams.category_id` 类型是 `Option<Option<i64>>`——`None` 不筛选，`Some(None)` 筛选未分组，`Some(Some(id))` 筛选指定分组。
- **清除分类**：`links_update` 中 `category_id = -1` 表示清除分类（设为 NULL）。`links_create` 中 `category_id` 经 `.filter(|&id| id > 0)` 处理，`-1` 和 `0` 都变为 NULL。
- **搜索降级**：`links_search` 先尝试 FTS5 MATCH，失败时降级为 LIKE。结果合并 FTS + 标签名匹配 + LIKE，去重返回。
- **导出**：通过 `rfd` 原生文件对话框保存，不是 Blob 下载。
- **全局快捷键**：`Cmd+Shift+L` 唤起快速添加。在 `lib.rs` 中用 Rust 代码注册，**不是** `tauri.conf.json`。快捷键 handler 动态创建 `quick-add` WebviewWindow。
- **深色模式**：CSS 变量 + `.dark` class 切换，持久化在 localStorage。所有模态框必须在 `.dark` div 内部，否则暗色变量不生效。
- **前端状态**：stores 用 Svelte 4 的 `writable`（适合跨组件），组件内用 Svelte 5 的 `$state`/`$derived` runes。混用是故意的。
- **侧边栏收缩**：`$state(new Set())` 对 `Set.add/delete` 不触发响应式，需要创建新 Set 赋值。

## 注意事项

- Tauri 2.x 中不存在 `tauri::InvokeError`——命令返回 `Result<T, E>`，`E: Serialize`。
- `rusqlite::Connection` 会对 prepared statement 持有借用——必须在释放 `MutexGuard` 之前 drop 掉所有 statement。
- Bundle identifier 不能以 `.app` 结尾（与 macOS `.app` 冲突）。
- 未使用 `@sveltejs/kit`——纯 Svelte + Vite。不要从 `@sveltejs/kit/vite` 导入。
- Svelte 5 不支持事件修饰符（`onclick|preventDefault` 无效），用 `(e) => { e.preventDefault(); ... }` 代替。
- Svelte 5 的 `$state(expr)` 只捕获初始值，不随变量变化自动更新（表单初始化模式，不是 bug）。
- Svelte 5 的 a11y 警告不阻止构建。
- `tauri-plugin-global-shortcut` 2.x **不接受** `tauri.conf.json` 的 `plugins` 配置——写入会崩溃。快捷键必须用 Rust 注册。
- `capabilities/default.json` 必须包含 `global-shortcut:allow-register` 和 `core:window:allow-start-dragging`，`windows` 数组必须包含 `"quick-add"`。
- Rust lib crate 名是 `app_lib`（`main.rs` 调用 `app_lib::run()`），不是 `links`。
- Vite 开发端口严格为 `1420`（`strictPort: true`）。
- `reqwest` 使用 `rustls-tls`（不依赖系统 OpenSSL），`rusqlite` 使用 `bundled`（自带 SQLite）。
- 主窗口 `titleBarStyle: "Overlay"` + `transparent: true`，拖拽通过 `data-tauri-drag-region`。
- HTML `<select>` 的 `<option value={null}>` 实际传字符串 `"null"` 而非 JS `null`，用 `value=""` 代替。
- 目前没有测试基础设施。

## 跨平台协作开发

本项目在 macOS 和 Windows 上同时开发，通过 GitHub 同步代码。

### 已配置

- **`.gitattributes`**：`text=auto eol=lf`，统一换行符为 LF，消除跨平台 diff
- **`.gitignore`**：覆盖 `.DS_Store`、`Thumbs.db`、`.idea/`、`.vscode/` 等平台和 IDE 文件
- **依赖**：`rusqlite bundled` + `reqwest rustls-tls`，无需系统安装 SQLite/OpenSSL，两端开箱即用

### 注意事项

- 提交前运行 `cd src-tauri && cargo check` 确保 Rust 编译通过
- 提交信息统一使用**中文**
- Windows 端如遇到 `rusqlite` 编译失败，确认已安装 Visual Studio C++ Build Tools
- macOS 端如遇到端口占用：`lsof -i :1420` 查找并 kill 进程
