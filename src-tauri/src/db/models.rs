//! 数据模型与 DTO。
//!
//! 与前端通过 serde 序列化的边界类型集中在此，方便对照命名约定。
//! 数据库行映射函数见 `super::row_mapping`。

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Link {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub description: String,
    pub notes: String,
    pub favicon_url: String,
    pub og_image_url: String,
    pub category_id: Option<i64>,
    pub is_favorite: bool,
    pub is_broken: bool,
    pub click_count: i64,
    pub last_opened_at: Option<i64>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLinkPayload {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub category_id: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub favicon_url: Option<String>,
    pub og_image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLinkPayload {
    pub id: i64,
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub category_id: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: Option<bool>,
    pub is_broken: Option<bool>,
    pub favicon_url: Option<String>,
    pub og_image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub sort_order: i32,
    pub children: Vec<Category>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryPayload {
    pub name: String,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryPayload {
    pub id: i64,
    pub name: Option<String>,
    #[serde(default)]
    pub parent_id: Option<i64>,
    #[serde(default)]
    pub unset_parent: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTagPayload {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopLink {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub click_count: i64,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LinksStats {
    pub total: i64,
    pub this_week: i64,
    pub top: Vec<TopLink>,
}

#[derive(Debug, Deserialize)]
pub struct ListLinksParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub category_id: Option<Option<i64>>,
    pub tag: Option<String>,
    pub query: Option<String>,
    pub favorite_only: Option<bool>,
    pub untagged_only: Option<bool>,
    pub uncategorized_only: Option<bool>,
    pub sort_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub query: String,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub category_id: Option<Option<i64>>,
    pub tag: Option<String>,
    pub favorite_only: Option<bool>,
    pub untagged_only: Option<bool>,
    pub sort_by: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResult<T: Serialize> {
    pub items: Vec<T>,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Deserialize)]
pub struct ExportParams {
    pub format: String,
    pub category_id: Option<i64>,
    pub tag: Option<String>,
    pub favorite_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlatCategory {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonExport {
    pub links: Vec<Link>,
    pub categories: Vec<FlatCategory>,
    #[serde(default)]
    pub stats: Option<LinksStats>,
}

impl Default for UpdateLinkPayload {
    fn default() -> Self {
        Self {
            id: 0,
            url: None,
            title: None,
            description: None,
            notes: None,
            category_id: None,
            tags: None,
            is_favorite: None,
            is_broken: None,
            favicon_url: None,
            og_image_url: None,
        }
    }
}
