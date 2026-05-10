//! 数据库 schema 与迁移框架。
//!
//! 历史方案是 `CREATE TABLE IF NOT EXISTS` + `ALTER TABLE ADD COLUMN .ok()`，
//! 真错误（如 SQL 语法错）会被 `.ok()` 静默吞掉，缺乏可观测性。
//!
//! 现在改用 `PRAGMA user_version` 显式版本管理：
//! - 每个迁移是 `Migration { version, sql }` 元组
//! - 启动时读取 user_version，依次跑所有 version > current 的迁移
//! - 每个迁移用事务包裹，失败回滚整个迁移并向上抛 AppError
//!
//! 兼容老数据库（user_version 默认 0）的关键：v1 基线必须保持幂等
//! （`CREATE TABLE IF NOT EXISTS` / `ALTER TABLE` 在已有列时会失败但被显式
//! 忽略），这样老库升级到 v1 时不会因为表/列已存在而崩溃。

use rusqlite::Result as SqlResult;

use super::Db;

/// 单个迁移版本。
struct Migration {
    /// 目标版本号（成功执行后写入 user_version）
    version: i32,
    /// 执行函数：成功返回 Ok，失败由调用方回滚整个事务
    apply: fn(&rusqlite::Transaction) -> SqlResult<()>,
}

const MIGRATIONS: &[Migration] = &[
    Migration { version: 1, apply: m1_baseline },
    // 未来新增迁移按版本号递增追加；不要修改已发布的迁移函数
];

/// v1 基线：完整建表 + 触发器 + 历史 ALTER。
/// 等价于历史 migrate() 的全部内容；幂等，可在已有库上重复执行。
fn m1_baseline(tx: &rusqlite::Transaction) -> SqlResult<()> {
    tx.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            parent_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );

        CREATE TABLE IF NOT EXISTS links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL,
            title TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            favicon_url TEXT NOT NULL DEFAULT '',
            og_image_url TEXT NOT NULL DEFAULT '',
            category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
            is_favorite INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );

        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS link_tags (
            link_id INTEGER NOT NULL REFERENCES links(id) ON DELETE CASCADE,
            tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
            PRIMARY KEY (link_id, tag_id)
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS links_fts USING fts5(
            title, description, notes, url,
            content=links, content_rowid=id
        );

        CREATE TRIGGER IF NOT EXISTS links_ai AFTER INSERT ON links BEGIN
            INSERT INTO links_fts(rowid, title, description, notes, url)
            VALUES (new.id, new.title, new.description, new.notes, new.url);
        END;

        CREATE TRIGGER IF NOT EXISTS links_ad AFTER DELETE ON links BEGIN
            INSERT INTO links_fts(links_fts, rowid, title, description, notes, url)
            VALUES ('delete', old.id, old.title, old.description, old.notes, old.url);
        END;

        CREATE TRIGGER IF NOT EXISTS links_au AFTER UPDATE ON links BEGIN
            INSERT INTO links_fts(links_fts, rowid, title, description, notes, url)
            VALUES ('delete', old.id, old.title, old.description, old.notes, old.url);
            INSERT INTO links_fts(rowid, title, description, notes, url)
            VALUES (new.id, new.title, new.description, new.notes, new.url);
        END;
        ",
    )?;

    // 历史 ADD COLUMN：老库升 v1 时这些列可能不存在；新库 CREATE TABLE 已含则
    // 报 "duplicate column"，用 try_alter 容忍这一类预期内的重复添加。
    try_alter(tx, "ALTER TABLE links ADD COLUMN is_broken INTEGER NOT NULL DEFAULT 0")?;
    try_alter(tx, "ALTER TABLE tags ADD COLUMN updated_at TEXT NOT NULL DEFAULT ''")?;
    tx.execute_batch(
        "UPDATE tags SET updated_at = datetime('now','localtime') WHERE updated_at = ''",
    )?;
    try_alter(tx, "ALTER TABLE links ADD COLUMN click_count INTEGER NOT NULL DEFAULT 0")?;
    try_alter(tx, "ALTER TABLE links ADD COLUMN last_opened_at INTEGER DEFAULT NULL")?;

    Ok(())
}

/// 容忍 "duplicate column" 错误的 ALTER 包装。
/// 其它错误（如语法错、表不存在）正常向上传播，不再静默吞掉。
fn try_alter(tx: &rusqlite::Transaction, sql: &str) -> SqlResult<()> {
    match tx.execute_batch(sql) {
        Ok(()) => Ok(()),
        Err(rusqlite::Error::SqliteFailure(_, Some(ref msg)))
            if msg.contains("duplicate column name") =>
        {
            Ok(())
        }
        Err(e) => Err(e),
    }
}

impl Db {
    /// 应用所有未执行的迁移。每个迁移用事务包裹，
    /// 成功后通过 `PRAGMA user_version = N` 推进版本号。
    pub fn migrate(&self) -> SqlResult<()> {
        let mut conn = self.0.lock().unwrap();
        let current: i32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        log::info!("[migrate] current user_version = {}", current);

        for m in MIGRATIONS {
            if m.version <= current {
                continue;
            }
            log::info!("[migrate] applying v{}", m.version);
            let tx = conn.transaction()?;
            (m.apply)(&tx)?;
            // PRAGMA 不支持参数绑定，必须字面量拼接；version 是 i32 常量，无注入风险
            tx.execute_batch(&format!("PRAGMA user_version = {}", m.version))?;
            tx.commit()?;
            log::info!("[migrate] v{} done", m.version);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;
    use rusqlite::Connection;
    use std::sync::Mutex;

    fn open_in_memory() -> Db {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        Db(Mutex::new(conn))
    }

    #[test]
    fn migrate_fresh_db_to_latest() {
        let db = open_in_memory();
        db.migrate().unwrap();

        let conn = db.0.lock().unwrap();
        let v: i32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
        assert_eq!(v, MIGRATIONS.last().unwrap().version);

        // 关键表与列都存在
        let table_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN \
                 ('categories','links','tags','link_tags')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(table_count, 4);

        let click_count_exists: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('links') WHERE name='click_count'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(click_count_exists, 1);
    }

    #[test]
    fn migrate_is_idempotent() {
        let db = open_in_memory();
        db.migrate().unwrap();
        // 重复 migrate 不应报错（user_version 已是最新，循环全部跳过）
        db.migrate().unwrap();
    }

    #[test]
    fn migrate_handles_legacy_schema_without_user_version() {
        // 模拟 R5 之前的老库：所有表已存在但 user_version=0
        let db = open_in_memory();
        {
            let conn = db.0.lock().unwrap();
            conn.execute_batch(
                "
                CREATE TABLE categories (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    parent_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
                    sort_order INTEGER NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
                );
                CREATE TABLE links (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    url TEXT NOT NULL,
                    title TEXT NOT NULL DEFAULT '',
                    description TEXT NOT NULL DEFAULT '',
                    notes TEXT NOT NULL DEFAULT '',
                    favicon_url TEXT NOT NULL DEFAULT '',
                    og_image_url TEXT NOT NULL DEFAULT '',
                    category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
                    is_favorite INTEGER NOT NULL DEFAULT 0,
                    is_broken INTEGER NOT NULL DEFAULT 0,
                    click_count INTEGER NOT NULL DEFAULT 0,
                    last_opened_at INTEGER DEFAULT NULL,
                    created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
                );
                CREATE TABLE tags (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    updated_at TEXT NOT NULL DEFAULT ''
                );
                ",
            )
            .unwrap();
        }

        // 应当幂等地补齐 link_tags / fts / 触发器，并把版本号推到 1
        db.migrate().unwrap();

        let conn = db.0.lock().unwrap();
        let v: i32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
        assert_eq!(v, 1);

        let link_tags_exists: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='link_tags'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(link_tags_exists, 1);
    }
}
