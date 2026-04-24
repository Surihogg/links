use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PageMeta {
    pub title: String,
    pub description: String,
    pub favicon_url: String,
    pub og_image_url: String,
    pub keywords: Vec<String>,
}

pub async fn fetch_metadata(url: &str) -> Result<PageMeta, reqwest::Error> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .build()?;

    let resp = client.get(url).send().await?;

    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !content_type.starts_with("text/html") {
        return Ok(PageMeta {
            title: String::new(),
            description: String::new(),
            favicon_url: String::new(),
            og_image_url: String::new(),
            keywords: vec![],
        });
    }

    let bytes = resp.bytes().await?;
    let cap = 512 * 1024;
    let html = if bytes.len() > cap {
        String::from_utf8_lossy(&bytes[..cap]).to_string()
    } else {
        String::from_utf8_lossy(&bytes).to_string()
    };

    let doc = scraper::Html::parse_document(&html);

    let title: String = doc
        .select(&scraper::Selector::parse("title").unwrap())
        .next()
        .map(|e| e.inner_html().trim().to_string())
        .unwrap_or_default()
        .chars()
        .take(500)
        .collect();

    let description: String = select_meta_content(&doc, "meta[property=\"og:description\"]")
        .or_else(|| select_meta_content(&doc, "meta[name=\"description\"]"))
        .unwrap_or_default()
        .chars()
        .take(2000)
        .collect();

    let og_image_url = select_meta_content(&doc, "meta[property=\"og:image\"]").unwrap_or_default();

    let keywords: Vec<String> = select_meta_content(&doc, "meta[name=\"keywords\"]")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .take(8)
        .collect();

    let base_url = url::Url::parse(url).ok();
    let favicon_url = select_link_href(&doc, "link[rel~=\"icon\"]")
        .or_else(|| select_link_href(&doc, "link[rel~=\"shortcut icon\"]"))
        .map(|href| resolve_url(base_url.as_ref(), &href))
        .unwrap_or_else(|| {
            base_url
                .as_ref()
                .map(|u| format!("{}/favicon.ico", u.origin().ascii_serialization()))
                .unwrap_or_default()
        });

    Ok(PageMeta {
        title,
        description,
        favicon_url,
        og_image_url,
        keywords,
    })
}

fn select_meta_content(doc: &scraper::Html, selector: &str) -> Option<String> {
    let sel = scraper::Selector::parse(selector).ok()?;
    doc.select(&sel)
        .next()
        .and_then(|e| e.value().attr("content").map(|s| s.trim().to_string()))
}

fn select_link_href(doc: &scraper::Html, selector: &str) -> Option<String> {
    let sel = scraper::Selector::parse(selector).ok()?;
    doc.select(&sel)
        .next()
        .and_then(|e| e.value().attr("href").map(|s| s.trim().to_string()))
}

pub(crate) fn resolve_url(base: Option<&url::Url>, href: &str) -> String {
    match base {
        Some(base) => base.join(href).map(|u| u.to_string()).unwrap_or_else(|_| href.to_string()),
        None => href.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 直接解析 HTML 字符串，提取元数据（不经过 HTTP）
    fn parse_html_meta(html: &str, url: &str) -> (String, String, String, String) {
        let doc = scraper::Html::parse_document(html);
        let base_url = url::Url::parse(url).ok();

        let title: String = doc
            .select(&scraper::Selector::parse("title").unwrap())
            .next()
            .map(|e| e.inner_html().trim().to_string())
            .unwrap_or_default()
            .chars()
            .take(500)
            .collect();

        let description: String = select_meta_content(&doc, "meta[property=\"og:description\"]")
            .or_else(|| select_meta_content(&doc, "meta[name=\"description\"]"))
            .unwrap_or_default()
            .chars()
            .take(2000)
            .collect();

        let og_image_url =
            select_meta_content(&doc, "meta[property=\"og:image\"]").unwrap_or_default();

        let favicon_url = select_link_href(&doc, "link[rel~=\"icon\"]")
            .or_else(|| select_link_href(&doc, "link[rel~=\"shortcut icon\"]"))
            .map(|href| resolve_url(base_url.as_ref(), &href))
            .unwrap_or_else(|| {
                base_url
                    .as_ref()
                    .map(|u| format!("{}/favicon.ico", u.origin().ascii_serialization()))
                    .unwrap_or_default()
            });

        (title, description, favicon_url, og_image_url)
    }

    #[test]
    fn test_complete_html() {
        let html = r#"
        <html><head>
            <title>Test Page</title>
            <meta name="description" content="A test description">
            <link rel="icon" href="/favicon.ico">
            <meta property="og:image" content="https://example.com/og.png">
        </head><body></body></html>"#;
        let (title, desc, favicon, og) = parse_html_meta(html, "https://example.com/page");
        assert_eq!(title, "Test Page");
        assert_eq!(desc, "A test description");
        assert_eq!(favicon, "https://example.com/favicon.ico");
        assert_eq!(og, "https://example.com/og.png");
    }

    #[test]
    fn test_missing_title() {
        let html = "<html><head></head><body></body></html>";
        let (title, _, _, _) = parse_html_meta(html, "https://example.com");
        assert_eq!(title, "");
    }

    #[test]
    fn test_og_description_preferred() {
        let html = r#"
        <html><head>
            <title>T</title>
            <meta property="og:description" content="OG desc">
            <meta name="description" content="Meta desc">
        </head></html>"#;
        let (_, desc, _, _) = parse_html_meta(html, "https://example.com");
        assert_eq!(desc, "OG desc");
    }

    #[test]
    fn test_fallback_to_meta_description() {
        let html = r#"
        <html><head>
            <title>T</title>
            <meta name="description" content="Meta desc">
        </head></html>"#;
        let (_, desc, _, _) = parse_html_meta(html, "https://example.com");
        assert_eq!(desc, "Meta desc");
    }

    #[test]
    fn test_favicon_relative_path() {
        let html = r#"
        <html><head>
            <link rel="icon" href="/assets/img/favicon.png">
        </head></html>"#;
        let (_, _, favicon, _) = parse_html_meta(html, "https://example.com/page");
        assert_eq!(favicon, "https://example.com/assets/img/favicon.png");
    }

    #[test]
    fn test_favicon_default_when_missing() {
        let html = "<html><head></head></html>";
        let (_, _, favicon, _) = parse_html_meta(html, "https://example.com/page");
        assert_eq!(favicon, "https://example.com/favicon.ico");
    }

    #[test]
    fn test_title_truncation() {
        let long_title: String = "A".repeat(600);
        let html = format!("<html><head><title>{}</title></head></html>", long_title);
        let (title, _, _, _) = parse_html_meta(&html, "https://example.com");
        assert_eq!(title.len(), 500);
    }

    #[test]
    fn test_description_truncation() {
        let long_desc: String = "B".repeat(3000);
        let html = format!(
            "<html><head><title>T</title><meta name=\"description\" content=\"{}\"></head></html>",
            long_desc
        );
        let (_, desc, _, _) = parse_html_meta(&html, "https://example.com");
        assert_eq!(desc.len(), 2000);
    }

    #[test]
    fn test_empty_html() {
        let html = "";
        let (title, desc, favicon, og) = parse_html_meta(html, "https://example.com");
        assert_eq!(title, "");
        assert_eq!(desc, "");
        assert_eq!(favicon, "https://example.com/favicon.ico");
        assert_eq!(og, "");
    }

    #[test]
    fn test_special_chars_in_title() {
        let html = r#"<html><head><title>Tom &amp; Jerry's "Great" Show &lt;3</title></head></html>"#;
        let (title, _, _, _) = parse_html_meta(html, "https://example.com");
        assert!(title.contains("Tom &amp; Jerry"));
    }

    #[test]
    fn test_resolve_url_relative() {
        let base = url::Url::parse("https://example.com/path/page").unwrap();
        assert_eq!(
            resolve_url(Some(&base), "/favicon.ico"),
            "https://example.com/favicon.ico"
        );
    }

    #[test]
    fn test_resolve_url_protocol_relative() {
        let base = url::Url::parse("https://example.com").unwrap();
        assert_eq!(
            resolve_url(Some(&base), "//cdn.example.com/icon.png"),
            "https://cdn.example.com/icon.png"
        );
    }

    #[test]
    fn test_resolve_url_no_base() {
        assert_eq!(resolve_url(None, "/favicon.ico"), "/favicon.ico");
    }

    #[test]
    fn test_resolve_url_absolute() {
        let base = url::Url::parse("https://example.com").unwrap();
        assert_eq!(
            resolve_url(Some(&base), "https://other.com/icon.png"),
            "https://other.com/icon.png"
        );
    }

    #[test]
    fn test_shortcut_icon_fallback() {
        let html = r#"
        <html><head>
            <link rel="shortcut icon" href="/sfavicon.ico">
        </head></html>"#;
        let (_, _, favicon, _) = parse_html_meta(html, "https://example.com");
        assert_eq!(favicon, "https://example.com/sfavicon.ico");
    }

    #[test]
    fn test_bilibili_like_favicon() {
        // Simulate Bilibili's exact HTML structure with apple-touch-icon and shortcut icon
        let html = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8" />
    <title>哔哩哔哩 (゜-゜)つロ 干杯~-bilibili</title>
    <link rel="dns-prefetch" href="//s1.hdslb.com" />
    <link rel="apple-touch-icon" href="https://i0.hdslb.com/bfs/static/jinkela/long/images/512.png" />
    <link rel="shortcut icon" href="https://i0.hdslb.com/bfs/static/jinkela/long/images/favicon.ico" />
</head><body></body></html>"#;
        let (_, _, favicon, _) = parse_html_meta(html, "https://www.bilibili.com/");
        assert_eq!(favicon, "https://i0.hdslb.com/bfs/static/jinkela/long/images/favicon.ico");
    }
}
