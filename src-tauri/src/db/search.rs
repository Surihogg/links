//! 搜索与统计。
//!
//! `search_links` 实现 FTS5 + 标签 LIKE + 分类 LIKE + 字段 LIKE 的四路 UNION
//! 查询，FTS 失败时退化到 LIKE-only fallback。`get_stats` 提供 sidebar 用的
//! 总览数据（总数、本周新增、点击 Top 3）。

use super::error::AppError;
use super::models::{Link, LinksStats, PaginatedResult, SearchParams, TopLink};
use super::row_mapping::{load_tags_for_links, row_to_link, LINK_COLUMNS};
use super::Db;

impl Db {
    pub fn get_stats(&self) -> Result<LinksStats, AppError> {
        let conn = self.0.lock().unwrap();
        compute_stats(&conn)
    }

    pub fn search_links(
        &self,
        params: &SearchParams,
    ) -> Result<PaginatedResult<Link>, AppError> {
        let conn = self.0.lock().unwrap();
        let query = &params.query;
        let page = params.page.unwrap_or(1).max(1);
        let per_page = params.per_page.unwrap_or(30).min(100);
        let offset = (page - 1) * per_page;

        let like_pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
        let fts_query = format!("{}*", query.replace('"', "\"\""));

        let fts_sql = format!(
            "SELECT {} FROM links l JOIN links_fts fts ON fts.rowid = l.id WHERE links_fts MATCH ?",
            LINK_COLUMNS
        );
        let tag_sql = format!(
            "SELECT DISTINCT {} FROM links l JOIN link_tags lt ON lt.link_id = l.id JOIN tags t ON t.id = lt.tag_id WHERE t.name LIKE ?",
            LINK_COLUMNS
        );
        let cat_sql = format!(
            "SELECT {} FROM links l JOIN categories c ON l.category_id = c.id WHERE c.name LIKE ?",
            LINK_COLUMNS
        );
        let like_sql = format!(
            "SELECT {} FROM links l WHERE l.title LIKE ? OR l.description LIKE ? OR l.notes LIKE ? OR l.url LIKE ?",
            LINK_COLUMNS
        );

        let base_union = format!("{} UNION {} UNION {} UNION {}", fts_sql, tag_sql, cat_sql, like_sql);

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
        let (union_sql, count_sql) = if filter_parts.is_empty() {
            (
                format!("{} ORDER BY {}", base_union, order_by),
                format!("SELECT COUNT(*) FROM ({})", base_union),
            )
        } else {
            let w = filter_parts.join(" AND ");
            (
                format!(
                    "SELECT * FROM ({}) AS sub WHERE {} ORDER BY {}",
                    base_union, w, order_by
                ),
                format!("SELECT COUNT(*) FROM ({}) AS sub WHERE {}", base_union, w),
            )
        };

        let build_search_params = |search_q: &str, like_p: &str| -> Vec<Box<dyn rusqlite::types::ToSql>> {
            let mut p: Vec<Box<dyn rusqlite::types::ToSql>> = vec![
                Box::new(search_q.to_string()),
                Box::new(like_p.to_string()),
                Box::new(like_p.to_string()),
                Box::new(like_p.to_string()),
                Box::new(like_p.to_string()),
                Box::new(like_p.to_string()),
                Box::new(like_p.to_string()),
            ];
            if let Some(Some(cid)) = params.category_id {
                p.push(Box::new(cid));
            }
            if let Some(ref tag) = params.tag {
                p.push(Box::new(tag.clone()));
            }
            p
        };

        let build_like_params = |like_p: &str| -> Vec<Box<dyn rusqlite::types::ToSql>> {
            let mut p: Vec<Box<dyn rusqlite::types::ToSql>> = vec![
                Box::new(like_p.to_string()),
                Box::new(like_p.to_string()),
                Box::new(like_p.to_string()),
                Box::new(like_p.to_string()),
                Box::new(like_p.to_string()),
                Box::new(like_p.to_string()),
            ];
            if let Some(Some(cid)) = params.category_id {
                p.push(Box::new(cid));
            }
            if let Some(ref tag) = params.tag {
                p.push(Box::new(tag.clone()));
            }
            p
        };

        let full_query = format!("{} LIMIT ? OFFSET ?", union_sql);
        let query_p = build_search_params(&fts_query, &like_pattern);
        let count_p = build_search_params(&fts_query, &like_pattern);

        let mut all_p: Vec<Box<dyn rusqlite::types::ToSql>> = query_p;
        all_p.push(Box::new(per_page));
        all_p.push(Box::new(offset));

        let res: Result<Vec<Link>, _> = {
            let mut stmt = conn.prepare(&full_query)?;
            let r = stmt
                .query_map(
                    rusqlite::params_from_iter(all_p.iter().map(|v| v.as_ref())),
                    row_to_link,
                )?
                .collect();
            drop(stmt);
            r
        };

        match res {
            Ok(items) => {
                let total: u32 = conn
                    .query_row(
                        &count_sql,
                        rusqlite::params_from_iter(count_p.iter().map(|v| v.as_ref())),
                        |r| r.get(0),
                    )
                    .unwrap_or(items.len() as u32);

                let mut items = items;
                load_tags_for_links(&conn, &mut items);
                Ok(PaginatedResult {
                    items,
                    total,
                    page,
                    per_page,
                })
            }
            Err(_) => {
                // FTS 解析失败（如查询包含特殊符号），降级到 LIKE-only
                let fallback_union = format!(
                    "{} UNION {} UNION {} ORDER BY {}",
                    tag_sql, cat_sql, like_sql, order_by
                );
                let (fb_query, fb_count) = if filter_parts.is_empty() {
                    (
                        format!("{} LIMIT ? OFFSET ?", fallback_union),
                        format!("SELECT COUNT(*) FROM ({})", fallback_union),
                    )
                } else {
                    let w = filter_parts.join(" AND ");
                    (
                        format!(
                            "SELECT * FROM ({}) AS sub WHERE {} ORDER BY {} LIMIT ? OFFSET ?",
                            fallback_union, w, order_by
                        ),
                        format!(
                            "SELECT COUNT(*) FROM ({}) AS sub WHERE {}",
                            fallback_union, w
                        ),
                    )
                };

                let mut fb_p = build_like_params(&like_pattern);
                let fb_count_p = build_like_params(&like_pattern);
                fb_p.push(Box::new(per_page));
                fb_p.push(Box::new(offset));

                let mut fb_stmt = conn.prepare(&fb_query)?;
                let items: Vec<Link> = fb_stmt
                    .query_map(
                        rusqlite::params_from_iter(fb_p.iter().map(|v| v.as_ref())),
                        row_to_link,
                    )?
                    .collect::<Result<Vec<_>, _>>()?;
                drop(fb_stmt);

                let total: u32 = conn
                    .query_row(
                        &fb_count,
                        rusqlite::params_from_iter(fb_count_p.iter().map(|v| v.as_ref())),
                        |r| r.get(0),
                    )
                    .unwrap_or(items.len() as u32);

                let mut items = items;
                load_tags_for_links(&conn, &mut items);
                Ok(PaginatedResult {
                    items,
                    total,
                    page,
                    per_page,
                })
            }
        }
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
