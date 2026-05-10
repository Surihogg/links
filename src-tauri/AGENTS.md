# Rust 后端 (src-tauri)

## 概述

Tauri 2.x Rust 后端。SQLite 数据持久化 + URL 元数据抓取 + 系统集成（托盘、快捷键、剪贴板）。
R2 重构后 `db.rs` 1686 行已拆为 `db/` 子模块；R5 引入 `PRAGMA user_version` 显式迁移框架。

## 查找指引

| 需求 | 文件 | 说明 |
|------|------|------|
| 新增/修改 Tauri 命令 | `commands.rs` | `#[tauri::command]` 函数；新增后须在 `lib.rs` 的 `invoke_handler!` 中注册；命名采用「动词在前」 |
| 数据模型与 DTO | `db/models.rs` | `Link` / `Category` / `Tag` / `*Payload` / `LinksStats` / `SearchParams` 等 |
| 数据库 schema 与迁移 | `db/migration.rs` | `Migration { version, apply }` + `MIGRATIONS` 数组；`PRAGMA user_version` 驱动 |
| 链接 / 分组 / 标签 CRUD | `db/repository.rs` | `impl Db` 含 `find_by_url` / `create_link` / `list_links` / 分组与标签全套 |
| 搜索与统计 | `db/search.rs` | `search_links` 四路 UNION + LIKE 回退；`get_stats` Top 3 |
| 导出 | `db/export.rs` | json / markdown / csv / Netscape Bookmark HTML |
| 行映射 / 公共 SQL 工具 | `db/row_mapping.rs` | `row_to_link` / `row_to_category` / `row_to_tag` / `LINK_COLUMNS` / `load_tags_for_links`（批量）/ `build_category_tree` / `ensure_tags` |
| 错误类型 | `db/error.rs` | `AppError` 枚举（Database/Io/Fetch/Json/General）+ Serialize impl |
| 内嵌测试 | `db/tests.rs` | 30+ 个 `#[test]`，用 `Connection::open_in_memory()` |
| URL 元数据抓取 | `fetcher.rs` | reqwest + scraper；R1 抽出的 `build_http_client` 共用 Windows 系统代理 |
| Windows 代理读取 | `fetcher.rs` | 注册表读 `ProxyEnable` / `ProxyServer` / `ProxyOverride` |
| 配置管理 | `config.rs` | `Config(Mutex<HashMap>)` 读写 config.json |
| URL 标准化 | `normalize.rs` | 去重用标准化（**仍未接入**，`#[allow(dead_code)]`，详见 §备注） |
| 浏览器扩展本地 HTTP 服务 | `http_server.rs` | tiny_http；端口 + 令牌鉴权 |
| 应用启动/插件/托盘 | `lib.rs` | Tauri Builder 配置、setup 钩子、命令注册表、deep-link 回调 |
| Rust 入口 | `main.rs` | 调用 `app_lib::run()`，`#![windows_subsystem = "windows"]` |
| 依赖/构建 | `Cargo.toml` | tauri 2.10.x、rusqlite 0.31 bundled、reqwest rustls、tiny_http、scraper |

## 约定

- **命令签名**：`fn cmd_name(db: State<Db>, app: AppHandle, params: Payload) -> Result<T, AppError>`
- **命令命名**：「动词在前」（`list_links` / `create_link` / `search_links` / `get_links_stats` / `autocomplete_tags`）；R2 后已统一，新增命令必须遵循
- **错误处理**：统一返回 `AppError`，前端通过 Serialize impl 收到字符串；用 `?` 传播或 `map_err`，不静默吞错
- **异步模式**：耗时操作用 `tauri::async_runtime::spawn`，不阻塞主线程
- **DB 访问**：`db.0.lock().unwrap()` 获取连接；**短临界区**；不跨 await 持锁；不在锁内做文件 IO
- **数据库迁移**：在 `db/migration.rs` 的 `MIGRATIONS` 数组末尾追加新 `Migration { version, apply }`；**不允许**修改已发布的迁移函数；ALTER TABLE 用 `try_alter` 容忍 duplicate column 但其它错误正常上抛
- **N+1 防御**：list/search/export 路径加载链接列表后**必须**用 `load_tags_for_links(&conn, &mut items)` 批量填充标签；`load_tags_for_link`（单数）仅用于 create_link / update_link 单条返回
- **配置键**：字符串键值对，JSON 字符串值通过 `config.get()` / `config.set()` 读写；保存须 `config.save(&app_data_dir(app)?)`
- **路径解析**：用 `commands::app_data_dir(app)` helper 获取数据目录，避免散落的 `expect()`
- **快捷键管理**：4 套快捷键（quick-add/main/spotlight/hide）通过 `update_shortcut(app, config, &KIND, value)` 统一处理
- **HTTP 客户端**：`fetch_metadata` 与 `check_link_status` 共用 `fetcher::build_http_client(url, timeout, ua)`；不再各写一份代理装配
- **日志**：`log::info!` / `log::warn!` / `log::error!`，自动写入 `links.log`（tauri-plugin-log）；元数据抓取失败额外写 `fail_links.log`
- **测试**：内嵌 `#[cfg(test)] mod tests` 或独立 `tests.rs`；用 `Connection::open_in_memory()` 隔离

## 反模式

- **禁止**在 `commands.rs` 中写 SQL 业务逻辑，应委托给 `db/*.rs` 的 `impl Db` 方法
- **禁止**跨 `.await` 持有 `Mutex<Connection>` 锁，会死锁
- **禁止**在持锁期间做文件 IO 或长时间计算（R1 修复了 `import_bookmarks` 的此类阻塞）
- **禁止**同步 HTTP 请求，必须 `reqwest::Client` + `.await`
- **禁止**忽略 `AppError` 转换，用 `?` 传播或 `map_err`
- **禁止**修改已发布过的 `Migration` 函数（迁移历史不可变）；只能追加新版本
- **禁止**用 `.ok()` 静默吞 SQL 错误（除明确预期的 duplicate column）
- **禁止**在 list/search/export 路径用 `for link in items { load_tags_for_link(...) }` 写法（N+1）

## 备注

- `db/` 子模块（R2 拆分后）：`mod.rs` 53 / `error.rs` 25 / `models.rs` 180 / `migration.rs` 256 / `row_mapping.rs` 192 / `repository.rs` 455 / `search.rs` 257 / `export.rs` 247 / `tests.rs` 822（测试占比 26%）
- 生产代码最大文件：`commands.rs` 1094 行（导入解析 + 39 个命令 + 共享 helper）
- `fetcher.rs` 566 行（含 Windows 代理逻辑与 13 个测试）；截取前 2MB HTML 防 OOM；标题/描述截断 500/2000 字符
- `lib.rs` 518 行：插件、托盘、deep-link、HTTP 服务装配；`invoke_handler!` 注册全部 39 个命令
- `normalize.rs` 仍是死代码（127 行 + 14 个测试）：R5 评估后决定**不接入**——库内已存的 URL 是用户粘贴的原文，强制归一化会让"按 URL 查询/对比"在用户眼里突然变样；现有 `find_by_url` 的尾斜杠容错已覆盖最常见场景
- `is_broken` / `click_count` / `last_opened_at` / `tags.updated_at` 都是历史 ALTER TABLE 加的列；R5 后由 v1 基线迁移用 `try_alter` 容忍重复添加
- 抓取失败日志：`commands::log_fetch_failure` 写入 `fail_links.log`（独立于 tauri-plugin-log，便于事后排查）
- `cargo test` 共 95 个：90 个 db CRUD + 3 迁移框架 + 2 批量加载（R5 新增）
- 启动期 `data_dir(app)` 用 expect 直接 panic（应用都启动不了）；运行期请用 `commands::app_data_dir` 传播 `AppError`
- HTTP 服务监听 127.0.0.1，端口动态分配（启动时找空闲），token 32 字符随机生成；前端通过 `get_local_server_info` 拉取动态参数构造 Bookmarklet
- `main.rs` 含 `#![windows_subsystem = "windows"]` 指令（**禁止移除**，否则 Windows 会弹出 console 窗口）
