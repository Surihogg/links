# Rust 后端 (src-tauri)

## 概述

Tauri 2.x Rust 后端。SQLite 数据持久化 + URL 元数据抓取 + 系统集成（托盘、快捷键、剪贴板）。

## 查找指引

| 需求 | 文件 | 说明 |
|------|------|------|
| 新增/修改 Tauri 命令 | `commands.rs` | `#[tauri::command]` 函数，修改后需在 `lib.rs` 注册 |
| 数据库操作/模型 | `db.rs` | 建表、迁移、CRUD、搜索、导出；`Db(Mutex<Connection>)` |
| URL 元数据抓取 | `fetcher.rs` | reqwest + scraper 解析；Windows 代理检测 |
| 配置管理 | `config.rs` | `Config(Mutex<HashMap>)` 读写 config.json |
| URL 标准化 | `normalize.rs` | 去重用标准化（**未接入**，`#[allow(dead_code)]`） |
| 应用启动/插件/托盘 | `lib.rs` | Tauri Builder 配置、setup 钩子、命令注册表 |
| 依赖/构建配置 | `Cargo.toml` | tauri 2.10.3, rusqlite 0.31 bundled, reqwest rustls |

## 约定

- **命令签名**：`fn cmd_name(db: State<Db>, app: AppHandle, params: Payload) -> Result<T, AppError>`
- **错误处理**：统一返回 `AppError`（Database/IO/Fetch/Json/General），前端收到字符串
- **异步模式**：耗时操作用 `tauri::async_runtime::spawn`，不阻塞主线程
- **DB 访问**：`db.0.lock().unwrap()` 获取连接，短临界区，不跨 await 持锁
- **配置键**：字符串键值对，JSON 值通过 `config.get()` / `config.set()` 读写
- **日志**：`log::info!` / `log::warn!` / `log::error!`，自动写入 `links.log`
- **测试**：内联 `#[cfg(test)] mod tests`，用 `Connection::open_in_memory()` 内存数据库

## 反模式

- **禁止**在 `commands.rs` 中写 SQL 业务逻辑，应委托给 `db.rs` 方法
- **禁止**跨 `.await` 持有 `Mutex<Connection>` 锁，会死锁
- **禁止**同步 HTTP 请求，必须 `reqwest::Client` + `.await`
- **禁止**忽略 `AppError` 转换，用 `?` 传播或 `map_err`

## 备注

- `db.rs` 约 1245 行（含测试），是最大文件；`search_links` 有 FTS5 → 标签 → LIKE 三级降级
- `fetcher.rs` 截取前 2MB HTML 防止 OOM，标题/描述分别截断 500/2000 字符
- `normalize.rs` 实现完整但未使用，去重目前用 `find_by_url` 的尾部斜杠容错
- `is_broken` 列通过 `ALTER TABLE ADD COLUMN` 添加（非正式迁移），migrate 函数中 `.ok()` 忽略重复添加
- Windows 代理：`fetcher.rs` 从注册表读取 `ProxyEnable` / `ProxyServer`
- 抓取失败日志：`commands.rs` 的 `log_fetch_failure` 写入 `fail_links.log`（非 tauri-plugin-log）
