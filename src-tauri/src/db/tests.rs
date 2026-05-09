//! db 模块的内部单元测试。
//!
//! 用 `Connection::open_in_memory()` 在内存中跑全套迁移与 CRUD，
//! 与生产代码完全隔离。

use std::sync::Mutex;

use rusqlite::Connection;

use super::models::*;
use super::row_mapping::ensure_tags;
use super::Db;

fn test_db() -> Db {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    let db = Db(Mutex::new(conn));
    db.migrate().unwrap();
    db
}

fn make_link(url: &str) -> CreateLinkPayload {
    CreateLinkPayload {
        url: url.to_string(),
        title: None,
        description: None,
        notes: None,
        category_id: None,
        tags: None,
        favicon_url: None,
        og_image_url: None,
    }
}

fn make_link_full(url: &str, title: &str, desc: &str, tags: Vec<&str>) -> CreateLinkPayload {
    CreateLinkPayload {
        url: url.to_string(),
        title: Some(title.to_string()),
        description: Some(desc.to_string()),
        notes: None,
        category_id: None,
        tags: Some(tags.iter().map(|s| s.to_string()).collect()),
        favicon_url: None,
        og_image_url: None,
    }
}

fn make_link_full_cat(url: &str, title: &str, cat_id: Option<i64>) -> CreateLinkPayload {
    CreateLinkPayload {
        url: url.to_string(),
        title: Some(title.to_string()),
        description: None,
        notes: None,
        category_id: cat_id,
        tags: None,
        favicon_url: None,
        og_image_url: None,
    }
}

#[test]
fn test_create_link_minimal() {
    let db = test_db();
    let link = db.create_link(&make_link("https://example.com")).unwrap();
    assert_eq!(link.url, "https://example.com");
    assert_eq!(link.title, "");
    assert!(link.category_id.is_none());
    assert!(link.tags.is_empty());
}

#[test]
fn test_create_link_full() {
    let db = test_db();
    let cat = db
        .create_category(&CreateCategoryPayload {
            name: "Rust".into(),
            parent_id: None,
        })
        .unwrap();
    let payload = CreateLinkPayload {
        url: "https://rust-lang.org".into(),
        title: Some("Rust Language".into()),
        description: Some("Systems programming language".into()),
        notes: Some("My notes".into()),
        category_id: Some(cat.id),
        tags: Some(vec!["rust".into(), "programming".into()]),
        favicon_url: Some("https://rust-lang.org/favicon.ico".into()),
        og_image_url: None,
    };
    let link = db.create_link(&payload).unwrap();
    assert_eq!(link.title, "Rust Language");
    assert_eq!(link.description, "Systems programming language");
    assert_eq!(link.category_id, Some(cat.id));
    let mut tag_names = link.tags.clone();
    tag_names.sort();
    assert_eq!(tag_names, vec!["programming", "rust"]);
}

#[test]
fn test_category_id_filter_negative() {
    let db = test_db();
    let payload = CreateLinkPayload {
        category_id: Some(-1),
        ..make_link("https://example.com")
    };
    let link = db.create_link(&payload).unwrap();
    assert!(link.category_id.is_none());
}

#[test]
fn test_category_id_filter_zero() {
    let db = test_db();
    let payload = CreateLinkPayload {
        category_id: Some(0),
        ..make_link("https://example.com")
    };
    let link = db.create_link(&payload).unwrap();
    assert!(link.category_id.is_none());
}

#[test]
fn test_list_links_pagination() {
    let db = test_db();
    for i in 0..5 {
        db.create_link(&make_link(&format!("https://example.com/{}", i)))
            .unwrap();
    }
    let result = db
        .list_links(&ListLinksParams {
            page: Some(1),
            per_page: Some(3),
            category_id: None,
            tag: None,
            query: None,
            favorite_only: None,
            untagged_only: None,
            uncategorized_only: None,
            sort_by: None,
        })
        .unwrap();
    assert_eq!(result.items.len(), 3);
    assert_eq!(result.total, 5);
    assert_eq!(result.page, 1);
}

#[test]
fn test_list_links_filter_by_category() {
    let db = test_db();
    let cat = db
        .create_category(&CreateCategoryPayload {
            name: "News".into(),
            parent_id: None,
        })
        .unwrap();
    db.create_link(&CreateLinkPayload {
        category_id: Some(cat.id),
        ..make_link("https://news.com")
    })
    .unwrap();
    db.create_link(&make_link("https://other.com")).unwrap();

    let result = db
        .list_links(&ListLinksParams {
            category_id: Some(Some(cat.id)),
            page: None,
            per_page: None,
            tag: None,
            query: None,
            favorite_only: None,
            untagged_only: None,
            uncategorized_only: None,
            sort_by: None,
        })
        .unwrap();
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].url, "https://news.com");
}

#[test]
fn test_list_links_filter_by_tag() {
    let db = test_db();
    db.create_link(&make_link_full(
        "https://rust-lang.org",
        "Rust",
        "Lang",
        vec!["rust"],
    ))
    .unwrap();
    db.create_link(&make_link("https://other.com")).unwrap();

    let result = db
        .list_links(&ListLinksParams {
            tag: Some("rust".into()),
            page: None,
            per_page: None,
            category_id: None,
            query: None,
            favorite_only: None,
            untagged_only: None,
            uncategorized_only: None,
            sort_by: None,
        })
        .unwrap();
    assert_eq!(result.items.len(), 1);
}

#[test]
fn test_list_links_filter_favorites() {
    let db = test_db();
    let link = db.create_link(&make_link("https://example.com")).unwrap();
    db.update_link(&UpdateLinkPayload {
        id: link.id,
        is_favorite: Some(true),
        ..Default::default()
    })
    .unwrap();
    db.create_link(&make_link("https://other.com")).unwrap();

    let result = db
        .list_links(&ListLinksParams {
            favorite_only: Some(true),
            page: None,
            per_page: None,
            category_id: None,
            tag: None,
            query: None,
            untagged_only: None,
            uncategorized_only: None,
            sort_by: None,
        })
        .unwrap();
    assert_eq!(result.items.len(), 1);
    assert!(result.items[0].is_favorite);
}

#[test]
fn test_update_link_title() {
    let db = test_db();
    let link = db.create_link(&make_link("https://example.com")).unwrap();
    let updated = db
        .update_link(&UpdateLinkPayload {
            id: link.id,
            title: Some("New Title".into()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(updated.title, "New Title");
}

#[test]
fn test_update_link_clear_category() {
    let db = test_db();
    let cat = db
        .create_category(&CreateCategoryPayload {
            name: "Test".into(),
            parent_id: None,
        })
        .unwrap();
    let link = db
        .create_link(&CreateLinkPayload {
            category_id: Some(cat.id),
            ..make_link("https://example.com")
        })
        .unwrap();
    assert_eq!(link.category_id, Some(cat.id));

    let updated = db
        .update_link(&UpdateLinkPayload {
            id: link.id,
            category_id: Some(-1),
            ..Default::default()
        })
        .unwrap();
    assert!(updated.category_id.is_none());
}

#[test]
fn test_update_link_replace_tags() {
    let db = test_db();
    let link = db
        .create_link(&make_link_full(
            "https://example.com",
            "T",
            "D",
            vec!["old1", "old2"],
        ))
        .unwrap();
    assert_eq!(link.tags, vec!["old1", "old2"]);

    let updated = db
        .update_link(&UpdateLinkPayload {
            id: link.id,
            tags: Some(vec!["new1".into()]),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(updated.tags, vec!["new1"]);
}

#[test]
fn test_delete_link_removes_tag_associations() {
    let db = test_db();
    let link = db
        .create_link(&make_link_full(
            "https://example.com",
            "T",
            "D",
            vec!["tag1"],
        ))
        .unwrap();
    db.delete_link(link.id).unwrap();

    let tags = db.list_tags().unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "tag1");
}

#[test]
fn test_toggle_favorite() {
    let db = test_db();
    let link = db.create_link(&make_link("https://example.com")).unwrap();
    assert!(!link.is_favorite);

    let updated = db
        .update_link(&UpdateLinkPayload {
            id: link.id,
            is_favorite: Some(true),
            ..Default::default()
        })
        .unwrap();
    assert!(updated.is_favorite);
}

#[test]
fn test_category_tree() {
    let db = test_db();
    let parent = db
        .create_category(&CreateCategoryPayload {
            name: "Parent".into(),
            parent_id: None,
        })
        .unwrap();
    let child = db
        .create_category(&CreateCategoryPayload {
            name: "Child".into(),
            parent_id: Some(parent.id),
        })
        .unwrap();

    let tree = db.list_categories().unwrap();
    assert_eq!(tree.len(), 1);
    assert_eq!(tree[0].id, parent.id);
    assert_eq!(tree[0].children.len(), 1);
    assert_eq!(tree[0].children[0].id, child.id);
}

#[test]
fn test_category_tree_deterministic() {
    let db = test_db();
    let a = db
        .create_category(&CreateCategoryPayload {
            name: "A_Root".into(),
            parent_id: None,
        })
        .unwrap();
    let b = db
        .create_category(&CreateCategoryPayload {
            name: "B_Root".into(),
            parent_id: None,
        })
        .unwrap();
    let c = db
        .create_category(&CreateCategoryPayload {
            name: "C_Child".into(),
            parent_id: Some(a.id),
        })
        .unwrap();

    let tree = db.list_categories().unwrap();
    assert_eq!(tree.len(), 2);
    let a_cat = tree.iter().find(|t| t.id == a.id).unwrap();
    assert_eq!(a_cat.children.len(), 1);
    assert_eq!(a_cat.children[0].id, c.id);
    let b_cat = tree.iter().find(|t| t.id == b.id).unwrap();
    assert!(b_cat.children.is_empty());
}

#[test]
fn test_delete_parent_category_nullifies_child_ref() {
    let db = test_db();
    let parent = db
        .create_category(&CreateCategoryPayload {
            name: "Parent".into(),
            parent_id: None,
        })
        .unwrap();
    let _child = db
        .create_category(&CreateCategoryPayload {
            name: "Child".into(),
            parent_id: Some(parent.id),
        })
        .unwrap();
    db.create_link(&CreateLinkPayload {
        category_id: Some(parent.id),
        ..make_link("https://example.com")
    })
    .unwrap();

    db.delete_category(parent.id).unwrap();

    let link_after = db
        .list_links(&ListLinksParams {
            page: None,
            per_page: None,
            category_id: None,
            tag: None,
            query: None,
            favorite_only: None,
            untagged_only: None,
            uncategorized_only: None,
            sort_by: None,
        })
        .unwrap();
    assert!(link_after.items[0].category_id.is_none());

    let cats = db.list_categories().unwrap();
    assert_eq!(cats.len(), 1);
    assert!(cats[0].parent_id.is_none());
}

#[test]
fn test_tag_crud() {
    let db = test_db();
    let tag = db.create_tag("rust").unwrap();
    assert_eq!(tag.name, "rust");

    let dup = db.create_tag("rust").unwrap();
    assert_eq!(dup.id, tag.id);

    let tags = db.list_tags().unwrap();
    assert_eq!(tags.len(), 1);

    db.delete_tag(tag.id).unwrap();
    assert!(db.list_tags().unwrap().is_empty());
}

#[test]
fn test_autocomplete_tags() {
    let db = test_db();
    db.create_tag("rust").unwrap();
    db.create_tag("ruby").unwrap();
    db.create_tag("javascript").unwrap();

    let results = db.autocomplete_tags("ru").unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|t| t.name.contains("ru")));
}

#[test]
fn test_ensure_tags_trims_and_ignores_empty() {
    let db = test_db();
    let conn = db.0.lock().unwrap();
    let ids = ensure_tags(
        &conn,
        &["  hello  ".into(), "".into(), "world".into()],
    );
    assert_eq!(ids.len(), 2);
}

#[test]
fn test_search_fts() {
    let db = test_db();
    db.create_link(&make_link_full(
        "https://example.com",
        "Rust Programming Language",
        "A systems language",
        vec![],
    ))
    .unwrap();

    let results = db
        .search_links(&SearchParams {
            query: "Rust".into(),
            page: None,
            per_page: None,
            category_id: None,
            tag: None,
            favorite_only: None,
            untagged_only: None,
            sort_by: None,
        })
        .unwrap();
    assert!(!results.items.is_empty());
    assert!(results.items[0].title.contains("Rust"));
}

#[test]
fn test_search_chinese() {
    let db = test_db();
    db.create_link(&make_link_full(
        "https://example.com",
        "Rust 编程语言指南",
        "学习 Rust",
        vec![],
    ))
    .unwrap();

    let results = db
        .search_links(&SearchParams {
            query: "编程".into(),
            page: None,
            per_page: None,
            category_id: None,
            tag: None,
            favorite_only: None,
            untagged_only: None,
            sort_by: None,
        })
        .unwrap();
    assert!(!results.items.is_empty());
}

#[test]
fn test_search_by_tag_name() {
    let db = test_db();
    db.create_link(&make_link_full(
        "https://example.com",
        "Some Title",
        "desc",
        vec!["webassembly"],
    ))
    .unwrap();

    let results = db
        .search_links(&SearchParams {
            query: "webassembly".into(),
            page: None,
            per_page: None,
            category_id: None,
            tag: None,
            favorite_only: None,
            untagged_only: None,
            sort_by: None,
        })
        .unwrap();
    assert!(!results.items.is_empty());
}

#[test]
fn test_search_like_fallback() {
    let db = test_db();
    db.create_link(&make_link_full(
        "https://example.com",
        "Hello World",
        "desc",
        vec![],
    ))
    .unwrap();

    let results = db
        .search_links(&SearchParams {
            query: "ello".into(),
            page: None,
            per_page: None,
            category_id: None,
            tag: None,
            favorite_only: None,
            untagged_only: None,
            sort_by: None,
        })
        .unwrap();
    assert!(!results.items.is_empty());
}

#[test]
fn test_search_by_category_name() {
    let db = test_db();
    let cat = db
        .create_category(&CreateCategoryPayload {
            name: "开发工具".into(),
            parent_id: None,
        })
        .unwrap();
    db.create_link(&make_link_full_cat(
        "https://tauri.app",
        "Tauri App",
        Some(cat.id),
    ))
    .unwrap();

    let results = db
        .search_links(&SearchParams {
            query: "开发工具".into(),
            page: None,
            per_page: None,
            category_id: None,
            tag: None,
            favorite_only: None,
            untagged_only: None,
            sort_by: None,
        })
        .unwrap();
    assert_eq!(results.items.len(), 1, "应通过分组名搜到该分组下的链接");
    assert_eq!(results.items[0].url, "https://tauri.app");
}

#[test]
fn test_export_json() {
    let db = test_db();
    db.create_link(&make_link_full(
        "https://example.com",
        "Test",
        "Desc",
        vec!["tag1"],
    ))
    .unwrap();

    let json = db
        .export_links(&ExportParams {
            format: "json".into(),
            category_id: None,
            tag: None,
            favorite_only: None,
        })
        .unwrap();
    assert!(json.contains("Test"));
    assert!(json.contains("tag1"));
}

#[test]
fn test_export_markdown() {
    let db = test_db();
    db.create_link(&make_link_full(
        "https://example.com",
        "Test",
        "Desc",
        vec![],
    ))
    .unwrap();

    let md = db
        .export_links(&ExportParams {
            format: "markdown".into(),
            category_id: None,
            tag: None,
            favorite_only: None,
        })
        .unwrap();
    assert!(md.starts_with("# Links Export"));
    assert!(md.contains("[Test](https://example.com)"));
}

#[test]
fn test_export_csv() {
    let db = test_db();
    db.create_link(&make_link_full(
        "https://example.com",
        "Test",
        "Desc",
        vec![],
    ))
    .unwrap();

    let csv = db
        .export_links(&ExportParams {
            format: "csv".into(),
            category_id: None,
            tag: None,
            favorite_only: None,
        })
        .unwrap();
    assert!(csv.starts_with("title,url,description"));
    assert!(csv.contains("Test"));
}

#[test]
fn test_export_html_basic() {
    let db = test_db();
    db.create_link(&make_link_full(
        "https://example.com",
        "Root Link",
        "desc",
        vec![],
    ))
    .unwrap();

    let html = db
        .export_links(&ExportParams {
            format: "html".into(),
            category_id: None,
            tag: None,
            favorite_only: None,
        })
        .unwrap();
    assert!(html.starts_with("<!DOCTYPE NETSCAPE-Bookmark-file-1>"));
    assert!(html.contains(r#"<DT><A HREF="https://example.com""#));
    assert!(html.contains("Root Link"));
    assert!(html.contains("<DL><p>"));
    assert!(html.contains("</DL><p>"));
}

#[test]
fn test_export_html_with_categories() {
    let db = test_db();
    let parent = db
        .create_category(&CreateCategoryPayload {
            name: "开发工具".into(),
            parent_id: None,
        })
        .unwrap();
    let child = db
        .create_category(&CreateCategoryPayload {
            name: "Rust".into(),
            parent_id: Some(parent.id),
        })
        .unwrap();
    db.create_link(&make_link_full_cat(
        "https://tauri.app",
        "Tauri",
        Some(child.id),
    ))
    .unwrap();
    db.create_link(&make_link_full_cat(
        "https://example.com",
        "Root",
        None,
    ))
    .unwrap();

    let html = db
        .export_links(&ExportParams {
            format: "html".into(),
            category_id: None,
            tag: None,
            favorite_only: None,
        })
        .unwrap();
    assert!(html.contains("开发工具"));
    assert!(html.contains("Rust"));
    assert!(html.contains("Tauri"));
    assert!(html.contains("Root"));

    // 验证嵌套结构：开发工具 DL 在 Rust DL 之前关闭
    let dev_start = html.find("开发工具").unwrap();
    let rust_start = html.find("Rust</H3>").unwrap();
    assert!(rust_start > dev_start, "Rust should be nested inside 开发工具");
}

#[test]
fn test_export_html_escaping() {
    let db = test_db();
    db.create_link(&make_link_full(
        "https://example.com?a=1&b=2",
        "<Script>Alert</Script>",
        "desc",
        vec![],
    ))
    .unwrap();

    let html = db
        .export_links(&ExportParams {
            format: "html".into(),
            category_id: None,
            tag: None,
            favorite_only: None,
        })
        .unwrap();
    assert!(html.contains("&amp;"), "URL 中的 & 应被转义");
    assert!(html.contains("&lt;Script&gt;"), "标题中的 <> 应被转义");
    assert!(
        !html.contains("<Script>Alert</Script></A>"),
        "不应有未转义的 HTML 标签"
    );
}

#[test]
fn test_export_unsupported_format() {
    let db = test_db();
    let result = db.export_links(&ExportParams {
        format: "xml".into(),
        category_id: None,
        tag: None,
        favorite_only: None,
    });
    assert!(result.is_err());
}
