# Links - 本地链接管理工具

全平台桌面应用，用于管理个人书签和资源链接。

## 技术栈

- **Tauri 2.x**（Rust 后端 + WebView 前端，~5MB 体积）
- **Svelte 5** + **TailwindCSS 4**（Vite 构建）
- **SQLite**（rusqlite bundled）+ FTS5 全文搜索

## 命令

```bash
npm run dev              # 启动开发服务器（热重载）
npm run build            # 仅构建前端
npm run tauri dev        # 启动 Tauri 开发模式（Rust + 前端）
npm run tauri build      # 生产构建（.app / .dmg / .exe / .deb）
```

仅检查 Rust（快速，不涉及前端）：
```bash
cd src-tauri && cargo check
```

## 架构

```
src-tauri/src/
  main.rs        # 入口（windows_subsystem）
  lib.rs         # Tauri builder：插件、命令注册、数据库初始化
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
  pages/QuickAdd.svelte  # 快速添加窗口（全局快捷键目标）
```

## 关键模式

- **数据库**：`Db(Mutex<Connection>)` 作为 Tauri state 管理。启动时自动迁移建表。
- **数据位置**：`~/Library/Application Support/com.links.desktop/links.db`（macOS）
- **FTS5 同步**：通过 `links` 表上的触发器自动同步 `links_fts` 虚拟表。
- **元数据抓取**：`links_create` 先立即保存，然后 `tokio::spawn` 异步抓取。只填充空字段（不覆盖用户输入）。
- **动态 SQL 参数**：使用 `rusqlite::params_from_iter(p.iter().map(|v| v.as_ref()))` 处理变长 WHERE 子句——`params![]` 宏需要固定参数数量。
- **Svelte 5**：不支持事件修饰符（`onclick|preventDefault` 无效）。用 `(e) => { e.preventDefault(); ... }` 代替。双向绑定 props 用 `$bindable()`。
- **深色模式**：`app.css` 中的 CSS 变量，通过根 div 的 `.dark` class 切换。持久化在 `localStorage`。
- **导出**：JSON / Markdown / CSV，从 `ExportDialog` 触发。通过 `Blob` + `<a>` 点击下载。

## 注意事项

- Tauri 2.x 中不存在 `tauri::InvokeError`——命令返回 `Result<T, E>`，其中 `E: Serialize`。
- `rusqlite::Connection` 会对 prepared statement 持有借用——必须在释放 `MutexGuard` 之前 drop 掉所有 statement。需要时使用作用域块（`{ let conn = ...; ... }`）。
- Bundle identifier 不能以 `.app` 结尾（与 macOS `.app` 扩展名冲突）。
- 未使用 `@sveltejs/kit`——纯 Svelte + Vite。不要从 `@sveltejs/kit/vite` 导入。
- Svelte 5 的 a11y 警告（label 关联、span 上的 click handler）只是警告，不阻止构建。
