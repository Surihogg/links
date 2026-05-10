//! 链接导出。
//!
//! 支持 json / markdown / csv / html 四种格式：
//! - json/csv/markdown 走过滤参数（category_id / tag / favorite_only）
//! - html（Netscape Bookmark）则导出全部数据，保留分类树

use std::collections::HashMap;

use super::error::AppError;
use super::models::{Category, ExportParams, FlatCategory, JsonExport, Link};
use super::row_mapping::{
    build_category_tree, load_tags_for_links, row_to_category, row_to_link, LINK_COLUMNS,
};
use super::search::compute_stats;
use super::Db;

impl Db {
    pub fn export_links(&self, params: &ExportParams) -> Result<String, AppError> {
        // HTML 书签格式导出所有数据（不支持过滤）
        if params.format == "html" {
            return self.export_bookmark_html();
        }

        let conn = self.0.lock().unwrap();

        let mut where_clauses = Vec::new();
        let mut p: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(cid) = params.category_id {
            where_clauses.push("l.category_id = ?".to_string());
            p.push(Box::new(cid));
        }
        if let Some(ref tag) = params.tag {
            where_clauses.push(
                "EXISTS (SELECT 1 FROM link_tags lt JOIN tags t ON t.id = lt.tag_id WHERE lt.link_id = l.id AND t.name = ?)".to_string(),
            );
            p.push(Box::new(tag.clone()));
        }
        if params.favorite_only.unwrap_or(false) {
            where_clauses.push("l.is_favorite = 1".to_string());
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let sql = format!(
            "SELECT {} FROM links l {} ORDER BY l.updated_at DESC",
            LINK_COLUMNS, where_sql
        );

        let mut stmt = conn.prepare(&sql)?;
        let mut links: Vec<Link> = stmt
            .query_map(
                rusqlite::params_from_iter(p.iter().map(|v| v.as_ref())),
                row_to_link,
            )?
            .collect::<Result<Vec<_>, _>>()?;
        drop(stmt);

        // 批量加载所有过滤后链接的标签
        load_tags_for_links(&conn, &mut links);

        // 计算全局统计（全库维度，不受导出过滤影响）
        let stats = compute_stats(&conn)?;

        match params.format.as_str() {
            "json" => {
                let cat_sql =
                    "SELECT id, name, parent_id FROM categories ORDER BY sort_order, id";
                let cats: Vec<FlatCategory> = conn
                    .prepare(cat_sql)?
                    .query_map([], |row| {
                        Ok(FlatCategory {
                            id: row.get(0)?,
                            name: row.get(1)?,
                            parent_id: row.get(2)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;

                let export = JsonExport {
                    links,
                    categories: cats,
                    stats: Some(stats),
                };
                Ok(serde_json::to_string_pretty(&export)?)
            }
            "markdown" => {
                let mut md = String::from("# Links Export\n\n");
                md.push_str(&format!(
                    "> 收藏总数: {} | 本周新增: {} | 导出时间: {}\n\n",
                    stats.total,
                    stats.this_week,
                    chrono::Local::now().format("%Y-%m-%d %H:%M")
                ));
                if !stats.top.is_empty() {
                    md.push_str("## 最常访问\n\n");
                    for (i, link) in stats.top.iter().enumerate() {
                        md.push_str(&format!(
                            "{}. [{}]({}) — {} 次\n",
                            i + 1,
                            link.title,
                            link.url,
                            link.click_count
                        ));
                    }
                    md.push_str("\n---\n\n");
                }
                md.push_str("## 链接列表\n\n");
                for link in &links {
                    md.push_str(&format!("- [{}]({})", link.title, link.url));
                    if !link.tags.is_empty() {
                        md.push_str(&format!(" `{}`", link.tags.join("` `")));
                    }
                    if !link.description.is_empty() {
                        md.push_str(&format!("\n  {}", link.description));
                    }
                    md.push('\n');
                }
                Ok(md)
            }
            "csv" => {
                let mut csv =
                    String::from("title,url,description,tags,category_id,favorite,created_at\n");
                for link in &links {
                    csv.push_str(&format!(
                        "\"{}\",\"{}\",\"{}\",\"{}\",{},\"{}\",{}\n",
                        link.title.replace('"', "\"\""),
                        link.url.replace('"', "\"\""),
                        link.description.replace('"', "\"\""),
                        link.tags.join(","),
                        link.category_id
                            .map(|i| i.to_string())
                            .unwrap_or_default(),
                        link.is_favorite,
                        link.created_at,
                    ));
                }
                Ok(csv)
            }
            _ => Err(AppError::General(format!(
                "Unsupported export format: {}",
                params.format
            ))),
        }
    }

    fn export_bookmark_html(&self) -> Result<String, AppError> {
        let conn = self.0.lock().unwrap();

        let mut cat_stmt = conn.prepare(
            "SELECT id, name, parent_id, sort_order, created_at, updated_at FROM categories ORDER BY sort_order, updated_at",
        )?;
        let cats: Vec<Category> = cat_stmt
            .query_map([], row_to_category)?
            .collect::<Result<Vec<_>, _>>()?;
        drop(cat_stmt);

        // 复用统一的树构建（与 list_categories 共用）
        let cat_roots = build_category_tree(Vec::new(), cats);

        let sql = format!(
            "SELECT {} FROM links l ORDER BY l.updated_at DESC",
            LINK_COLUMNS
        );
        let mut link_stmt = conn.prepare(&sql)?;
        let links: Vec<Link> = link_stmt
            .query_map([], row_to_link)?
            .collect::<Result<Vec<_>, _>>()?;
        drop(link_stmt);

        let mut links_by_cat: HashMap<Option<i64>, Vec<&Link>> = HashMap::new();
        for link in &links {
            links_by_cat.entry(link.category_id).or_default().push(link);
        }

        Ok(generate_bookmark_html(&cat_roots, &links_by_cat))
    }
}

fn generate_bookmark_html(
    cat_roots: &[Category],
    links_by_cat: &HashMap<Option<i64>, Vec<&Link>>,
) -> String {
    let mut html = String::from(
        "<!DOCTYPE NETSCAPE-Bookmark-file-1>\n\
         <!-- This is an automatically generated file.\n\
              It will be read and overwritten.\n\
              DO NOT EDIT! -->\n\
         <META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n\
         <TITLE>Bookmarks</TITLE>\n\
         <H1>Bookmarks</H1>\n",
    );

    html.push_str("<DL><p>\n");

    if let Some(root_links) = links_by_cat.get(&None) {
        for link in root_links {
            html.push_str(&format_link_html(link, "    "));
        }
    }

    for cat in cat_roots {
        html.push_str(&format_category_html(cat, links_by_cat, "    "));
    }

    html.push_str("</DL><p>\n");
    html
}

fn format_link_html(link: &Link, indent: &str) -> String {
    let timestamp = chrono::NaiveDateTime::parse_from_str(&link.created_at, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.and_utc().timestamp())
        .unwrap_or(0);

    let mut attrs = format!(
        r#"HREF="{}" ADD_DATE="{}""#,
        escape_html_attr(&link.url),
        timestamp
    );
    if !link.favicon_url.is_empty() {
        attrs.push_str(&format!(
            r#" ICON="{}""#,
            escape_html_attr(&link.favicon_url)
        ));
    }

    format!(
        "{}<DT><A {}>{}</A>\n",
        indent,
        attrs,
        escape_html_attr(&link.title)
    )
}

fn format_category_html(
    cat: &Category,
    links_by_cat: &HashMap<Option<i64>, Vec<&Link>>,
    indent: &str,
) -> String {
    let inner_indent = format!("    {}", indent);

    let mut html = format!(
        "{}<DT><H3>{}</H3>\n{}<DL><p>\n",
        indent,
        escape_html_attr(&cat.name),
        indent
    );

    if let Some(links) = links_by_cat.get(&Some(cat.id)) {
        for link in links {
            html.push_str(&format_link_html(link, &inner_indent));
        }
    }

    for child in &cat.children {
        html.push_str(&format_category_html(child, links_by_cat, &inner_indent));
    }

    html.push_str(&format!("{}</DL><p>\n", indent));
    html
}

fn escape_html_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
