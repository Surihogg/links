use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PageMeta {
    pub title: String,
    pub description: String,
    pub favicon_url: String,
    pub og_image_url: String,
}

pub async fn fetch_metadata(url: &str) -> Result<PageMeta, reqwest::Error> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
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
        });
    }

    let html = resp.text().await?;
    let doc = scraper::Html::parse_document(&html);

    let title = doc
        .select(&scraper::Selector::parse("title").unwrap())
        .next()
        .map(|e| e.inner_html().trim().to_string())
        .unwrap_or_default();

    let description = select_meta_content(&doc, "meta[property=\"og:description\"]")
        .or_else(|| select_meta_content(&doc, "meta[name=\"description\"]"))
        .unwrap_or_default();

    let og_image_url = select_meta_content(&doc, "meta[property=\"og:image\"]").unwrap_or_default();

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

fn resolve_url(base: Option<&url::Url>, href: &str) -> String {
    match base {
        Some(base) => base.join(href).map(|u| u.to_string()).unwrap_or_else(|_| href.to_string()),
        None => href.to_string(),
    }
}
