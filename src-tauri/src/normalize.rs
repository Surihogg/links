use url::Url;

/// Normalize a URL for deduplication comparison.
///
/// - http → https
/// - remove www. prefix
/// - remove fragment (#section)
/// - keep query parameters
/// - lowercase host
/// - remove trailing slash from path (except root)
pub fn normalize_url(raw: &str) -> String {
    let trimmed = raw.trim();
    let mut parsed = match Url::parse(trimmed) {
        Ok(u) => u,
        Err(_) => return trimmed.to_string(),
    };

    // http → https
    if parsed.scheme() == "http" {
        let _ = parsed.set_scheme("https");
    }

    // Remove www. prefix + lowercase host
    if let Some(host) = parsed.host_str() {
        let normalized = host.strip_prefix("www.").unwrap_or(host).to_lowercase();
        let _ = parsed.set_host(Some(&normalized));
    }

    // Remove fragment
    parsed.set_fragment(None);

    // Remove trailing slash from path (but keep root "/")
    let path = parsed.path().to_owned();
    if path.len() > 1 && path.ends_with('/') {
        parsed.set_path(&path[..path.len() - 1]);
    }

    parsed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_to_https() {
        assert_eq!(normalize_url("http://example.com"), "https://example.com/");
    }

    #[test]
    fn test_remove_www() {
        assert_eq!(normalize_url("https://www.example.com"), "https://example.com/");
    }

    #[test]
    fn test_http_www_combined() {
        assert_eq!(normalize_url("http://www.example.com"), "https://example.com/");
    }

    #[test]
    fn test_remove_trailing_slash() {
        assert_eq!(normalize_url("https://example.com/path/"), "https://example.com/path");
    }

    #[test]
    fn test_root_path_kept() {
        // Root path "/" is preserved by the url crate
        assert_eq!(normalize_url("https://example.com/"), "https://example.com/");
    }

    #[test]
    fn test_remove_fragment() {
        let result = normalize_url("https://example.com/page#section");
        assert!(!result.contains('#'));
        assert!(result.starts_with("https://example.com/page"));
    }

    #[test]
    fn test_keep_query_params() {
        let result = normalize_url("https://example.com?foo=bar");
        assert!(result.contains("foo=bar"));
    }

    #[test]
    fn test_lowercase_host() {
        assert_eq!(normalize_url("https://Example.COM"), "https://example.com/");
    }

    #[test]
    fn test_malformed_url() {
        // Should return trimmed original
        assert_eq!(normalize_url("not-a-url"), "not-a-url");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(normalize_url(""), "");
    }

    #[test]
    fn test_whitespace_trimmed() {
        assert_eq!(normalize_url("  https://example.com  "), "https://example.com/");
    }

    #[test]
    fn test_full_normalization() {
        assert_eq!(
            normalize_url("http://www.Example.COM/path/?q=1#top"),
            "https://example.com/path?q=1"
        );
    }

    #[test]
    fn test_path_without_trailing_slash() {
        assert_eq!(normalize_url("https://example.com/path"), "https://example.com/path");
    }

    #[test]
    fn test_bilibili_variants() {
        let a = normalize_url("https://www.bilibili.com/");
        let b = normalize_url("http://bilibili.com");
        let c = normalize_url("https://www.bilibili.com");
        // All should normalize to the same thing
        assert_eq!(a, b);
        assert_eq!(b, c);
    }
}
