# Links 调试与启动指南

## 环境要求

| 工具 | 版本 | 检查命令 |
|---|---|---|
| Rust | ≥ 1.77.2 | `rustc --version` |
| Node.js | ≥ 18 | `node --version` |
| npm | ≥ 9 | `npm --version` |
| macOS | Xcode Command Line Tools | `xcode-select -p` |

首次安装 Rust：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## 启动方式

### 开发模式（推荐）

```bash
npm run tauri dev
```

这会同时启动：
1. Vite 开发服务器（`http://localhost:1420`）—— 前端热重载
2. Rust 编译 + Tauri 窗口

首次启动较慢（Rust 编译），后续增量编译很快。修改前端代码会自动热更新，修改 Rust 代码会触发重编译。

### 仅前端开发

```bash
npm run dev
```

只启动 Vite，在浏览器 `http://localhost:1420` 预览。注意：**后端调用会失败**（没有 Tauri 运行时），适合纯 UI 调整。

### 仅检查 Rust

```bash
cd src-tauri && cargo check
```

快速语法检查，不生成二进制文件。比完整编译快很多。

### 生产构建

```bash
npm run tauri build
```

产出：
- `src-tauri/target/release/bundle/macos/Links.app`
- `src-tauri/target/release/bundle/dmg/Links_0.1.0_aarch64.dmg`

---

## 常见启动问题

### 1. 端口 1420 被占用

**现象**：`npm run tauri dev` 报错 `Port 1420 is already in use`

**原因**：Vite 配置了 `strictPort: true`（Tauri 要求固定端口），上一个进程没有正确退出。

**解决**：
```bash
# 找到占用端口的进程
lsof -i :1420
# 杀掉进程
kill -9 <PID>
```

### 2. Rust 编译失败

**现象**：`cargo` 报错，通常在 `src-tauri/` 目录下

**排查步骤**：
```bash
cd src-tauri
cargo check 2>&1
```

常见原因：
- **依赖下载失败**：网络问题，重试 `cargo build`
- **rusqlite bundled 编译失败**：需要 C 编译器（macOS 自带 clang）
- **版本不兼容**：删除 `Cargo.lock`，重新 `cargo build`

### 3. 全局快捷键无效（Cmd+Shift+L）

**现象**：快捷键不响应

**排查**：
1. 检查 `capabilities/default.json` 是否包含 `global-shortcut:allow-register`
2. 检查 `lib.rs` 中 `GlobalShortcutExt::register` 是否被调用
3. macOS 系统设置中是否冲突：系统偏好设置 → 键盘 → 快捷键
4. ⚠️ **不要**在 `tauri.conf.json` 的 `plugins` 中添加全局快捷键配置——`tauri-plugin-global-shortcut` 2.x 不接受 JSON 配置，写入会导致启动崩溃

### 4. 应用启动后白屏

**现象**：窗口显示空白

**排查**：
1. 确认 Vite 开发服务器在 `localhost:1420` 运行
2. 打开 DevTools：macOS 上 `Cmd+Option+I`
3. 检查 Console 是否有错误
4. 检查 `tauri.conf.json` 的 `build.devUrl` 是否为 `http://localhost:1420`

### 5. 数据库初始化失败

**现象**：启动崩溃或链接数据不加载

**排查**：
1. 数据库位置：`~/Library/Application Support/com.links.desktop/links.db`
2. 检查目录权限：`ls -la ~/Library/Application\ Support/com.links.desktop/`
3. 如果数据库损坏，可以删除重建：
   ```bash
   rm ~/Library/Application\ Support/com.links.desktop/links.db
   rm ~/Library/Application\ Support/com.links.desktop/links.db-wal
   rm ~/Library/Application\ Support/com.links.desktop/links.db-shm
   ```
4. 重启应用，会自动迁移建表

---

## 调试技巧

### 打开 DevTools

在 Tauri 窗口中：
- **macOS**：`Cmd+Option+I`
- **Windows/Linux**：`Ctrl+Shift+I`

### 前端调试

在 DevTools Console 中可以直接调用后端命令：

```javascript
// 列出所有链接
const { invoke } = window.__TAURI__.core;
const result = await invoke("links_list", { params: {} });
console.log(result);

// 创建链接
await invoke("links_create", {
  payload: { url: "https://example.com" }
});

// 搜索
const results = await invoke("links_search", { query: "关键词" });

// 查看分类
const cats = await invoke("categories_list");
console.log(cats);
```

### Rust 日志

应用使用了 `tauri-plugin-log`，日志位置：
- **macOS**：`~/Library/Logs/com.links.desktop/`
- **Windows**：`%APPDATA%\com.links.desktop\logs\`
- **Linux**：`~/.local/share/com.links.desktop/logs/`

### 数据库直接查询

```bash
sqlite3 ~/Library/Application\ Support/com.links.desktop/links.db

# 常用查询
SELECT COUNT(*) FROM links;
SELECT * FROM links ORDER BY updated_at DESC LIMIT 5;
SELECT * FROM categories;
SELECT * FROM tags;
SELECT * FROM link_tags;

# FTS5 搜索测试
SELECT * FROM links_fts WHERE links_fts MATCH '关键词*';

# 退出
.quit
```

---

## 项目结构速查

```
src-tauri/src/
  main.rs              # 入口，调用 app_lib::run()
  lib.rs               # Tauri builder：插件注册、命令注册、数据库初始化、全局快捷键
  db.rs                # SQLite 建表/迁移、数据模型、Db(Mutex<Connection>) 封装
  fetcher.rs           # URL 元数据异步抓取
  commands.rs          # 12 个 Tauri 命令（前端通过 invoke 调用）

src/
  main.js              # Svelte 挂载入口
  App.svelte           # 主布局（侧边栏 + 链接列表 + 搜索 + FAB）
  app.css              # CSS 变量设计系统 + Tailwind + 明暗主题
  lib/
    api.js             # invoke() 封装，所有后端调用的入口
    stores/index.js    # Svelte writable stores（links/categories/tags）
    components/        # UI 组件
```

### 前端 → 后端调用链

```
组件 (Svelte)
  → store 方法 (stores/index.js)
    → api.js 函数
      → invoke("command_name", { params })
        → Rust #[tauri::command] (commands.rs)
          → Db 操作 (db.rs)
```

### 后端命令列表

| 命令 | 用途 |
|---|---|
| `links_list` | 分页查询链接（支持分类/标签/收藏筛选） |
| `links_create` | 创建链接 + 异步抓取元数据 |
| `links_update` | 更新链接字段（部分更新） |
| `links_delete` | 删除链接 |
| `links_search` | FTS5 全文搜索 |
| `categories_list` | 列出分类（树形结构） |
| `categories_create` | 创建分类 |
| `categories_update` | 更新分类 |
| `categories_delete` | 删除分类 |
| `tags_list` | 列出所有标签 |
| `tags_autocomplete` | 标签前缀搜索 |
| `export_links` | 导出为 JSON/Markdown/CSV |

---

## 已知坑点

### Svelte 5 特殊行为

- **不支持事件修饰符**：`onclick|preventDefault` 无效，改为 `(e) => { e.preventDefault(); ... }`
- **$state 初始化捕获**：`$state(link?.url ?? "")` 只捕获初始值，link 变化不会自动更新——这是故意的（表单初始化模式）
- **a11y 警告可以忽略**：label 关联、span click handler 等警告不阻止构建
- **stores 混用**：跨组件共享状态用 Svelte 4 的 `writable`，组件内部用 Svelte 5 的 `$state`/`$derived` runes

### Rust/Tauri 特殊行为

- **Db Mutex 生命周期**：`MutexGuard` 持有时，`Connection` 被锁定。prepared statement 借用 `Connection`，必须在 drop `MutexGuard` 之前结束。用作用域块 `{ let conn = db.0.lock().unwrap(); ... }` 控制
- **crate 名**：`Cargo.toml` 中 `[lib] name = "app_lib"`，`main.rs` 调用 `app_lib::run()`
- **Bundle identifier**：不能以 `.app` 结尾（与 macOS `.app` 冲突）
- **reqwest**：使用 `rustls-tls`（不依赖系统 OpenSSL），`rusqlite` 使用 `bundled`（自带 SQLite）
- **动态 SQL 参数**：变长 WHERE 子句用 `params_from_iter()`，不用 `params![]` 宏

### 全局快捷键

- 快捷键在 Rust 代码中注册（`lib.rs`），**不是** `tauri.conf.json`
- `capabilities/default.json` 必须包含 `global-shortcut:allow-register` 等权限
- `tauri-plugin-global-shortcut` 2.x 不接受 JSON 配置——写入会崩溃

---

## 快速诊断清单

应用不工作时，按顺序检查：

1. `npm run build` 是否成功？ → 前端编译问题
2. `cd src-tauri && cargo check` 是否通过？ → Rust 编译问题
3. `lsof -i :1420` 端口是否被占用？ → 端口冲突
4. DevTools Console 有无报错？ → 前端运行时错误
5. `sqlite3 .../links.db "SELECT COUNT(*) FROM links"` → 数据库是否正常
6. 日志目录有无错误日志？ → Rust 侧错误
