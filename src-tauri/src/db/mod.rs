//! 数据库层。
//!
//! 历史上 `db.rs` 单文件 1686 行，承担了模型 / 迁移 / CRUD / 搜索 / 导出
//! 全部职责。R2 重构后按主题拆分到子模块：
//!
//! - [`error`]：统一错误类型 [`AppError`]
//! - [`models`]：所有结构体 / DTO 定义
//! - [`migration`]：schema 与迁移（[`Db::migrate`]）
//! - [`row_mapping`]：SQL 行→Rust 结构的映射、`ensure_tags`、分类树构建
//! - [`repository`]：链接 / 分组 / 标签的 CRUD 与列表
//! - [`search`]：FTS5 搜索与统计
//! - [`export`]：链接导出（json / md / csv / html）
//!
//! 对外仍以 `crate::db::Xxx` 形式暴露公共项，保持 commands.rs 端调用不变。

use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;
use std::sync::Mutex;

mod error;
mod export;
mod migration;
mod models;
pub(crate) mod row_mapping;
mod repository;
mod search;

#[cfg(test)]
mod tests;

// —— 公开重导出 ——
pub use error::AppError;
pub use models::*;
// row_mapping 内部工具：仅在 commands::import_json_entries 等场景跨模块调用 ensure_tags
pub(crate) use row_mapping::ensure_tags;

/// SQLite 连接的 Mutex 封装。
///
/// 通过 `Mutex<Connection>` 串行化访问，临界区应尽量短，**禁止跨 await
/// 持锁**（参见 src-tauri/AGENTS.md 的反模式约定）。
pub struct Db(pub Mutex<Connection>);

impl Db {
    /// 打开（或新建）SQLite 数据库文件。启用 WAL 与外键。
    pub fn open(path: &Path) -> SqlResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Db(Mutex::new(conn)))
    }
}
