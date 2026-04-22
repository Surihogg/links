# Rust 后端

Tauri 命令处理 + SQLite + 元数据抓取。所有跨进程边界的逻辑都在这里。

## 文件职责

| 文件 | 内容 | 注意 |
|---|---|---|
| `main.rs` | `fn main()` 入口，6 行 | `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` 禁止 release 模式显示控制台 |
| `lib.rs` | Tauri Builder：插件、命令注册、DB 初始化、全局快捷键 | **所有命令必须在 `invoke_handler![...]` 里列出**，否则前端 invoke 会失败 |
| `db.rs` | 建表 SQL + 数据结构（`Link`/`Category`/`Tag`/`AppError`/`PaginatedResult`/参数结构）+ `Db(Mutex<Connection>)` | 建表用 `CREATE TABLE IF NOT EXISTS`，可重复运行不破坏数据 |
| `fetcher.rs` | `fetch_metadata(url)` — HTTP GET + `scraper` 解析 HTML | 超时 8s，响应体截断 512KB，title ≤500 字符，description ≤2000 字符 |
| `commands.rs` | 17 个 `#[tauri::command]` 函数 | 最大文件（553 行），CRUD + 搜索 + 导出都在这 |

## 命令签名约定

```rust
#[tauri::command]
pub fn foo(db: State<'_, Db>, payload: FooPayload) -> Result<Foo, AppError>
```

- **Payload 结构**：多字段参数用 `#[derive(Deserialize)] struct FooPayload`，不用多个裸参数。
- **State**：需要数据库用 `State<'_, Db>`，需要 AppHandle 用 `AppHandle`（不是 `&AppHandle`）。
- **错误**：一律返回 `Result<T, AppError>`。`AppError` 手动 `impl Serialize`（序列化为字符串），`tauri::InvokeError` 在 Tauri 2.x 不存在。
- **异步**：抓取这类 I/O 用 `async fn` + `.await`；纯 DB 操作用同步 `fn`（Mutex 是 `std::sync::Mutex`，不是 tokio 的）。

## Mutex 生命周期陷阱

```rust
let conn = db.0.lock().unwrap();
let mut stmt = conn.prepare("...")?;  // stmt 借用 conn
// ... 用 stmt
drop(stmt);  // 必须显式 drop 或用作用域块
// 现在才能用 conn 做其他事
```

- `rusqlite::Statement` 持有 `&Connection` 借用，**在释放 `MutexGuard` 之前必须 drop 所有 statement**。
- 长函数里用作用域块隔离：`let items = { let mut stmt = conn.prepare(...)?; ... };`。
- 见 `commands.rs` 中 `load_tags_for_link`、`links_search` 的显式 `drop(stmt)`。

## 动态 SQL 参数

变长 WHERE 子句不能用 `params![]`（要求固定参数数量）：

```rust
let mut p: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
if let Some(cid) = params.category_id {
    sql_parts.push("l.category_id = ?".into());
    p.push(Box::new(cid));
}
// ...
let rows = stmt.query_map(
    rusqlite::params_from_iter(p.iter().map(|v| v.as_ref())),
    row_to_link,
)?;
```

用 `params_from_iter` + `Box<dyn ToSql>`。`links_list`、`links_update`、`export_links` 都是这个模式。

## 元数据抓取异步派发

`links_create` 先同步保存链接，然后 `tauri::async_runtime::spawn`（**不是** `tokio::spawn`）异步抓取：

```rust
tauri::async_runtime::spawn(async move {
    match crate::fetcher::fetch_metadata(&url_for_fetch).await {
        Ok(meta) => { /* UPDATE 只填充空字段 */ }
        Err(e) => log::warn!("metadata fetch failed for {}: {}", url_for_fetch, e),
    }
});
```

- 只填充空字段：`SET title = CASE WHEN title = '' THEN ?1 ELSE title END, ...`。不覆盖用户手动填的标题。
- 失败必须 `log::warn!`（历史上因 `if let Ok(...)` 静默吞错误导致调试困难）。
- `app.state::<Db>()` 在 spawn 内重新获取（不要把 `State` clone 进闭包，无效）。

## `category_id` 的三值语义

- **`links_list` 筛选**：`ListLinksParams.category_id: Option<Option<i64>>`
  - `None` → 不筛选
  - `Some(None)` → 只返回未分组（`category_id IS NULL`）
  - `Some(Some(id))` → 返回指定分组
- **`links_create`**：`.filter(|&id| id > 0)`，把 `-1` / `0` 归为 NULL。
- **`links_update`**：显式用 `-1` 表示 "清除分类"（生成 `SET category_id = NULL`），正值为新分类 ID，`None` 表示不修改字段。

## 搜索降级

`links_search`：FTS5 MATCH 失败（比如查询包含特殊字符）→ 降级为 `tag LIKE ? UNION ... l.title LIKE ? OR l.description LIKE ? OR l.notes LIKE ?`。LIKE 模式转义 `%` 和 `_`。

## 反模式

- **不要**用 `tokio::spawn` 派发——用 `tauri::async_runtime::spawn`，保证在 Tauri runtime 上下文。
- **不要**静默 `.ok()` 掉 `fetch_metadata` 的错误——至少 `log::warn!` 输出原因。
- **不要**在 Tauri 2.x 里用 `tauri::InvokeError`——不存在。
- **不要**在 `links_create` / `links_update` 里覆盖已有元数据（用户编辑过的标题/描述）——走 `CASE WHEN '' THEN ? ELSE ... END`。
- **不要**把 `category_id` 当 `Option<i64>` 简单处理——`Option<Option<i64>>` 的三值语义是故意的。
