//! 搜索与统计。
//!
//! `search_links` 按 Unicode 空白拆分关键词，关键词之间使用 AND；每个关键词
//! 仍可命中 FTS5、标签、分类或基础字段 LIKE 中的任意一路。FTS 解析失败时，
//! 整组关键词统一降级到 LIKE-only，保证多条件语义不变。`get_stats` 提供
//! sidebar 用的总览数据（总数、本周新增、点击 Top 3）。

use super::error::AppError;
use super::models::{Link, LinksStats, PaginatedResult, SearchParams, TopLink};
use super::row_mapping::{load_tags_for_links, row_to_link, LINK_COLUMNS};
use super::Db;

/// LIKE 模式显式使用反斜杠转义，避免用户输入的 % / _ 被当成通配符。
fn like_pattern(term: &str) -> String {
    format!("%{}%", term.replace('%', "\\%").replace('_', "\\_"))
}

/// 单个关键词在非 FTS 字段中的匹配分支，只返回链接 id 供外层 AND 组合。
fn non_fts_id_union() -> String {
    "SELECT l.id FROM links l JOIN link_tags lt ON lt.link_id = l.id JOIN tags t ON t.id = lt.tag_id WHERE t.name LIKE ? ESCAPE '\\' \
     UNION SELECT l.id FROM links l JOIN categories c ON l.category_id = c.id WHERE c.name LIKE ? ESCAPE '\\' \
     UNION SELECT l.id FROM links l WHERE l.title LIKE ? ESCAPE '\\' OR l.description LIKE ? ESCAPE '\\' OR l.notes LIKE ? ESCAPE '\\' OR l.url LIKE ? ESCAPE '\\'"
        .into()
}

/// 构造关键词间 AND、单关键词多字段 OR 的链接查询。
fn search_source(terms: &[String], include_fts: bool) -> String {
    let non_fts = non_fts_id_union();
    let predicates = terms
        .iter()
        .map(|_| {
            if include_fts {
                format!(
                    "l.id IN (SELECT l.id FROM links l JOIN links_fts fts ON fts.rowid = l.id WHERE links_fts MATCH ? UNION {})",
                    non_fts
                )
            } else {
                format!("l.id IN ({})", non_fts)
            }
        })
        .collect::<Vec<_>>()
        .join(" AND ");
    format!("SELECT {} FROM links l WHERE {}", LINK_COLUMNS, predicates)
}

impl Db {
    pub fn get_stats(&self) -> Result<LinksStats, AppError> {
        let conn = self.0.lock().unwrap();
        compute_stats(&conn)
    }

    pub fn search_links(&self, params: &SearchParams) -> Result<PaginatedResult<Link>, AppError> {
        let conn = self.0.lock().unwrap();
        let page = params.page.unwrap_or(1).max(1);
        let per_page = params.per_page.unwrap_or(30).min(100);
        let offset = (page - 1) * per_page;

        let mut terms: Vec<String> = params.query.split_whitespace().map(str::to_owned).collect();
        // 保留历史空查询行为：走 LIKE %% 返回全部，FTS 失败不会影响结果。
        if terms.is_empty() {
            terms.push(String::new());
        }

        let mut filter_parts: Vec<String> = Vec::new();
        if let Some(ref cid_opt) = params.category_id {
            match cid_opt {
                Some(_) => filter_parts.push("category_id = ?".into()),
                None => filter_parts.push("category_id IS NULL".into()),
            }
        }
        if params.tag.is_some() {
            filter_parts.push("EXISTS (SELECT 1 FROM link_tags lt2 JOIN tags t2 ON t2.id = lt2.tag_id WHERE lt2.link_id = sub.id AND t2.name = ?)".into());
        }
        if params.untagged_only.unwrap_or(false) {
            filter_parts
                .push("NOT EXISTS (SELECT 1 FROM link_tags lt WHERE lt.link_id = sub.id)".into());
        }
        if params.favorite_only.unwrap_or(false) {
            filter_parts.push("is_favorite = 1".into());
        }

        let order_by = match params.sort_by.as_deref() {
            Some("click_count") => "click_count DESC, updated_at DESC",
            Some("last_opened_at") => "last_opened_at DESC, updated_at DESC",
            _ => "updated_at DESC",
        };

        let build_params = |include_fts: bool| -> Vec<Box<dyn rusqlite::types::ToSql>> {
            let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            for term in &terms {
                if include_fts {
                    values.push(Box::new(format!("{}*", term.replace('"', "\"\""))));
                }
                let pattern = like_pattern(term);
                // 标签、分类、标题、描述、备注、URL，共 6 个 LIKE 参数。
                for _ in 0..6 {
                    values.push(Box::new(pattern.clone()));
                }
            }
            if let Some(Some(cid)) = params.category_id {
                values.push(Box::new(cid));
            }
            if let Some(ref tag) = params.tag {
                values.push(Box::new(tag.clone()));
            }
            values
        };

        let execute = |include_fts: bool| -> Result<(Vec<Link>, u32), AppError> {
            let source = search_source(&terms, include_fts);
            let filtered = if filter_parts.is_empty() {
                format!("SELECT * FROM ({}) AS sub", source)
            } else {
                format!(
                    "SELECT * FROM ({}) AS sub WHERE {}",
                    source,
                    filter_parts.join(" AND ")
                )
            };
            let list_sql = format!("{} ORDER BY {} LIMIT ? OFFSET ?", filtered, order_by);
            let count_sql = format!("SELECT COUNT(*) FROM ({}) AS counted", filtered);

            let mut list_params = build_params(include_fts);
            let count_params = build_params(include_fts);
            list_params.push(Box::new(per_page));
            list_params.push(Box::new(offset));

            let items = {
                let mut stmt = conn.prepare(&list_sql)?;
                let rows = stmt.query_map(
                    rusqlite::params_from_iter(list_params.iter().map(|v| v.as_ref())),
                    row_to_link,
                )?;
                rows.collect::<Result<Vec<_>, _>>()?
            };
            let total = conn.query_row(
                &count_sql,
                rusqlite::params_from_iter(count_params.iter().map(|v| v.as_ref())),
                |row| row.get(0),
            )?;
            Ok((items, total))
        };

        // 任一关键词含 FTS 特殊语法时，整组查询统一回退，避免不同关键词
        // 分别走不同策略而出现难以理解的 AND 结果。
        let (mut items, total) = match execute(true) {
            Ok(result) => result,
            Err(_) => execute(false)?,
        };
        load_tags_for_links(&conn, &mut items);
        Ok(PaginatedResult {
            items,
            total,
            page,
            per_page,
        })
    }
}

/// 从已有连接计算统计数据（供 get_stats / export 共用，避免重复持锁）。
pub fn compute_stats(conn: &rusqlite::Connection) -> Result<LinksStats, AppError> {
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM links", [], |r| r.get(0))?;
    let this_week: i64 = conn.query_row(
        "SELECT COUNT(*) FROM links WHERE created_at >= datetime('now','localtime','-7 days')",
        [],
        |r| r.get(0),
    )?;
    let mut raw_links: Vec<Link> = {
        let mut stmt = conn.prepare(&format!(
            "SELECT {} FROM links l WHERE l.click_count > 0 ORDER BY l.click_count DESC LIMIT 3",
            LINK_COLUMNS
        ))?;
        let rows = stmt.query_map([], row_to_link)?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    load_tags_for_links(conn, &mut raw_links);
    let top: Vec<TopLink> = raw_links
        .into_iter()
        .map(|link| TopLink {
            id: link.id,
            title: if link.title.is_empty() {
                link.url.clone()
            } else {
                link.title.clone()
            },
            url: link.url,
            click_count: link.click_count,
            tags: link.tags,
        })
        .collect();
    Ok(LinksStats {
        total,
        this_week,
        top,
    })
}
