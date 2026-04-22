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
src-tauri/src/        # Rust 后端。详见 src-tauri/src/AGENTS.md
src/
  App.svelte          # 主布局（侧边栏 + 链接列表 + 搜索），.dark class 在此 div 上
  main.js             # Svelte 5 mount 挂载入口
  app.css             # CSS 变量设计系统（明暗主题）
  lib/api.js          # Tauri invoke 封装（所有后端调用）
  lib/stores/         # Svelte stores（links, categories, tags）
  lib/components/     # UI 组件。详见 src/lib/components/AGENTS.md
  pages/QuickAdd.svelte  # 快速添加页面（全局快捷键唤起的独立 WebviewWindow）
```

## 关键模式

- **数据流**：组件 → store 方法（`stores/index.js`）→ `api.js` → `invoke("cmd", { params })` → Rust 命令。
- **数据位置**（跨平台）：
  - macOS: `~/Library/Application Support/com.links.desktop/links.db`
  - Windows: `%APPDATA%\com.links.desktop\links.db`
  - Linux: `~/.local/share/com.links.desktop/links.db`
- **日志位置**（`tauri-plugin-log` 配置了 `LogDir` + `Stdout` + `Webview`）：
  - Windows: `%LOCALAPPDATA%\com.links.desktop\logs\`（注意是 LocalAppData，不是 Roaming）
  - macOS: `~/Library/Logs/com.links.desktop/`
  - Linux: `~/.local/share/com.links.desktop/logs/`
  - Webview target 会把 Rust 日志同步到 DevTools Console
- **全局快捷键**：`Cmd+Shift+L`（macOS）/ `Ctrl+Shift+L`（Windows）唤起快速添加。Rust 代码注册，**不是** `tauri.conf.json`。Handler 动态创建 `quick-add` WebviewWindow。
- **深色模式**：CSS 变量 + `.dark` class 切换，持久化在 localStorage（key：`links-dark-mode`）。**所有模态框必须在 `.dark` div 内部**，否则暗色变量不生效。
- **前端状态混用**：stores 用 Svelte 4 `writable`（跨组件共享），组件内用 Svelte 5 `$state`/`$derived` runes。**是故意的，不要统一**。
- **跨平台标题栏**：`titleBarStyle: "Overlay"` 在 macOS 隐藏原生标题栏（需要 36px padding + `data-tauri-drag-region`），在 Windows 保留原生标题栏。`App.svelte` 用 `is_macos`（从 `navigator.platform` 嗅探）条件渲染。
- **侧边栏 Set 响应式**：`$state(new Set())` 对 `Set.add/delete` **不触发响应式**，必须创建新 Set 赋值（`collapsed = new Set(collapsed)`）。见 `Sidebar.svelte` 的 `toggle_section`。

## 注意事项

- Tauri 2.x 中不存在 `tauri::InvokeError`——命令返回 `Result<T, E>`，`E: Serialize`。
- Bundle identifier 不能以 `.app` 结尾（与 macOS `.app` 冲突）。
- 未使用 `@sveltejs/kit`——纯 Svelte + Vite。不要从 `@sveltejs/kit/vite` 导入。
- Svelte 5 不支持事件修饰符（`onclick|preventDefault` 无效），用 `(e) => { e.preventDefault(); ... }` 代替。
- Svelte 5 的 `$state(expr)` 只捕获初始值，不随变量变化自动更新（表单初始化模式，不是 bug）。
- Svelte 5 的 a11y 警告不阻止构建（label 没 for、div 有 onclick 等，忽略即可）。
- `tauri-plugin-global-shortcut` 2.x **不接受** `tauri.conf.json` 的 `plugins` 配置——写入会崩溃。快捷键必须用 Rust 注册。
- `capabilities/default.json` 必须包含 `global-shortcut:allow-register` 和 `core:window:allow-start-dragging`，`windows` 数组必须包含 `"quick-add"`。
- Rust lib crate 名是 `app_lib`（`main.rs` 调用 `app_lib::run()`），不是 `links`。
- Vite 开发端口严格为 `1420`（`strictPort: true`）。
- `reqwest` 使用 `native-tls`（macOS: Security.framework, Windows: SChannel）——**不要改回 `rustls-tls`**，Windows 下会因证书问题抓取失败。
- `rusqlite` 使用 `bundled`（自带 SQLite，无需系统安装）。
- `@tauri-apps/api` v2 **没有** `os` 模块，平台检测用 `navigator.platform` 即可。
- 主窗口 `titleBarStyle: "Overlay"` + `transparent: true`，拖拽通过 `data-tauri-drag-region`。
- HTML `<select>` 的 `<option value={null}>` 实际传字符串 `"null"` 而非 JS `null`，用 `value=""` 代替。
- 目前没有测试基础设施。

## 跨平台协作开发

本项目在 macOS 和 Windows 上同时开发，通过 GitHub 同步代码。

### 已配置

- **`.gitattributes`**：`text=auto eol=lf`，统一换行符为 LF，消除跨平台 diff
- **`.gitignore`**：覆盖 `.DS_Store`、`Thumbs.db`、`.idea/`、`.vscode/` 等平台和 IDE 文件
- **依赖**：`rusqlite bundled` + `reqwest native-tls`，无需系统安装 SQLite，TLS 走系统原生栈

### 注意事项

- 提交前运行 `cd src-tauri && cargo check` 确保 Rust 编译通过
- 提交信息统一使用**中文**
- Windows 端如遇到 `rusqlite` 编译失败，确认已安装 Visual Studio C++ Build Tools
- macOS 端如遇到端口占用：`lsof -i :1420` 查找并 kill 进程
