//! 数据库 schema 与迁移。
//!
//! 当前用 `CREATE TABLE IF NOT EXISTS` + `ALTER TABLE ADD COLUMN .ok()` 的轻量
//! 方案，未来若引入显式版本表（PRAGMA user_version）应在此处统一。

use rusqlite::Result as SqlResult;

use super::Db;

impl Db {
    /// 创建或升级 schema。所有 ALTER TABLE 都用 `.ok()` 兜底重复添加；
    /// 若需要严格的迁移失败可观测性，应迁移到 user_version 框架。
    pub fn migrate(&self) -> SqlResult<()> {
        let conn = self.0.lock().unwrap();
        conn.execute_batch(
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

        // —— 兼容旧数据库的列追加（重复添加被静默忽略） ——
        conn.execute_batch("ALTER TABLE links ADD COLUMN is_broken INTEGER NOT NULL DEFAULT 0").ok();
        conn.execute_batch("ALTER TABLE tags ADD COLUMN updated_at TEXT NOT NULL DEFAULT ''").ok();
        conn.execute_batch("UPDATE tags SET updated_at = datetime('now','localtime') WHERE updated_at = ''").ok();
        // 统计与行为追踪
        conn.execute_batch("ALTER TABLE links ADD COLUMN click_count INTEGER NOT NULL DEFAULT 0").ok();
        conn.execute_batch("ALTER TABLE links ADD COLUMN last_opened_at INTEGER DEFAULT NULL").ok();
        Ok(())
    }
}
