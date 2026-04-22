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
  fetcher.rs     # URL 元数据抓取（标题、描述、favicon、og:image）
  commands.rs    # 所有 Tauri 命令（CRUD、搜索、导出）

src/
  App.svelte          # 主布局（侧边栏 + 链接列表 + 搜索）
  main.js             # 挂载入口
  app.css             # Tailwind + CSS 变量（明暗主题）
  lib/api.js          # Tauri invoke 封装（所有后端调用）
  lib/stores/         # Svelte stores（links, categories, tags）
  lib/components/     # UI 组件
  pages/QuickAdd.svelte  # 快速添加页面（全局快捷键唤起的独立窗口）
```

## 关键模式

- **数据库**：`Db(Mutex<Connection>)` 作为 Tauri state 管理。WAL 模式 + 外键约束。启动时自动迁移建表（`CREATE TABLE IF NOT EXISTS`）。
- **数据位置**：
  - macOS: `~/Library/Application Support/com.links.desktop/links.db`
  - Windows: `%APPDATA%\com.links.desktop\links.db`
  - Linux: `~/.local/share/com.links.desktop/links.db`
- **FTS5 同步**：通过 `links` 表上的触发器（INSERT/UPDATE/DELETE）自动同步 `links_fts` 虚拟表。
- **元数据抓取**：`links_create` 先立即保存，然后 `tauri::async_runtime::spawn` 异步抓取。只填充空字段（`CASE WHEN title = '' THEN ? ELSE title END`），不覆盖用户输入。
- **动态 SQL 参数**：使用 `rusqlite::params_from_iter(p.iter().map(|v| v.as_ref()))` 处理变长 WHERE 子句——`params![]` 宏需要固定参数数量。
- **全局快捷键**：`Cmd+Shift+L` 唤起快速添加窗口。快捷键在 `lib.rs` 中用 Rust 代码注册（`GlobalShortcutExt::register`），**不是**在 `tauri.conf.json` 中配置。快捷键 handler 动态创建 `quick-add` WebviewWindow。
- **分类筛选**：`ListLinksParams.category_id` 类型是 `Option<Option<i64>>`——`None` 表示不筛选，`Some(None)` 筛选未分类，`Some(Some(id))` 筛选指定分类。
- **清除分类**：`links_update` 中 `category_id = -1` 表示清除分类（设为 NULL）。不是用 `None`，而是用特殊值 `-1`。
- **搜索降级**：`links_search` 先尝试 FTS5 MATCH，失败时降级为 LIKE 查询。结果合并 FTS + 标签名匹配 + LIKE，去重后返回。
- **导出**：JSON / Markdown / CSV。前端调用 `export_links` 获取格式化内容，再调用 `save_file`（Rust 侧通过 `rfd` 原生文件对话框保存）。不是 Blob 下载。
- **Svelte 5**：不支持事件修饰符（`onclick|preventDefault` 无效）。用 `(e) => { e.preventDefault(); ... }` 代替。双向绑定 props 用 `$bindable()`。
- **深色模式**：`app.css` 中的 CSS 变量，通过根 div 的 `.dark` class 切换。持久化在 `localStorage`。
- **前端状态**：stores 使用 Svelte 4 风格的 `writable`（非 Svelte 5 runes），组件内用 `$state`/`$derived` runes。混用是故意的——stores 的 `writable` API 更适合跨组件共享。

## 注意事项

- Tauri 2.x 中不存在 `tauri::InvokeError`——命令返回 `Result<T, E>`，其中 `E: Serialize`。`AppError` 实现了 `Serialize` 即可。
- `rusqlite::Connection` 会对 prepared statement 持有借用——必须在释放 `MutexGuard` 之前 drop 掉所有 statement。需要时使用作用域块（`{ let conn = ...; ... }`）或显式 `drop(stmt)`。
- Bundle identifier 不能以 `.app` 结尾（与 macOS `.app` 扩展名冲突）。
- 未使用 `@sveltejs/kit`——纯 Svelte + Vite。不要从 `@sveltejs/kit/vite` 导入。
- Svelte 5 的 a11y 警告（label 关联、span 上的 click handler）只是警告，不阻止构建。
- Svelte 5 的 `$state(expr)` 只捕获初始值——`$state(link?.url ?? "")` 不会随 `link` 变化自动更新。这是故意的（表单初始化模式），不是 bug。
- `tauri-plugin-global-shortcut` 2.x **不接受** `tauri.conf.json` 的 `plugins` 配置——写入会导致启动崩溃（`"invalid type: map, expected unit"`）。快捷键必须用 Rust 代码注册。
- `capabilities/default.json` 必须包含 `global-shortcut:allow-register` 等权限，且 `windows` 数组必须包含 `"quick-add"`，否则快捷键创建的窗口权限不足。
- Rust lib crate 名是 `app_lib`（Cargo.toml `[lib] name`），不是 `links`。`main.rs` 调用 `app_lib::run()`。
- Vite 开发服务器端口严格为 `1420`（`strictPort: true`），这是 Tauri dev 模式期望的端口。
- `reqwest` 使用 `native-tls`（macOS: Security.framework, Windows: SChannel），`rusqlite` 使用 `bundled` 特性编译 SQLite——都能正确读取系统证书存储，且 SQLite 无需系统安装。
- 主窗口配置了 `titleBarStyle: "Overlay"` + `transparent: true`——自定义标题栏，拖拽区域通过 `core:window:allow-start-dragging` 权限实现。
- 目前没有测试基础设施。
