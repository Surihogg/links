// 本地 HTTP 服务：为 Bookmarklet 提供 fetch 端点，绕过浏览器对自定义协议的频率限制。
// 监听 127.0.0.1:48927（固定端口，端口被占时自动 +1 重试），使用 token 校验防 CSRF。

use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use tiny_http::{Header, Method, Response, Server};

/// 固定基础端口（IANA 未注册，与常见服务无冲突）
pub const BASE_PORT: u16 = 48927;
/// 端口被占时的最大重试次数
pub const MAX_PORT_RETRIES: u16 = 10;

/// 生成 32 字符随机 token（URL 安全字符集）
pub fn generate_token() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| {
            let i = rng.gen_range(0..CHARSET.len());
            CHARSET[i] as char
        })
        .collect()
}

/// 解析 query string 为 HashMap（仅简单 form-urlencoded）
pub fn parse_query(query: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut kv = pair.splitn(2, '=');
        let key = kv.next().unwrap_or("");
        let val = kv.next().unwrap_or("");
        // 直接使用 url crate 的解码器处理 percent-encoding 与 +
        let key = url::form_urlencoded::parse(key.as_bytes())
            .next()
            .map(|(k, _)| k.into_owned())
            .unwrap_or_default();
        let decoded_val: String = url::form_urlencoded::parse(format!("v={}", val).as_bytes())
            .next()
            .map(|(_, v)| v.into_owned())
            .unwrap_or_default();
        if !key.is_empty() {
            map.insert(key, decoded_val);
        }
    }
    map
}

/// 解析请求路径与 query：返回 (path_only, query_string)
pub fn split_path_and_query(raw_url: &str) -> (&str, &str) {
    match raw_url.find('?') {
        Some(i) => (&raw_url[..i], &raw_url[i + 1..]),
        None => (raw_url, ""),
    }
}

/// 抽离的请求校验逻辑（便于测试）：
/// 返回 Ok((url, title)) 表示通过校验；Err(status_code) 表示拒绝。
pub fn validate_add_request(
    method_is_get: bool,
    path: &str,
    query: &str,
    expected_token: &str,
) -> Result<(String, String), u16> {
    if !method_is_get {
        return Err(405);
    }
    if path != "/add" {
        return Err(404);
    }
    let params = parse_query(query);
    let token = params.get("t").map(|s| s.as_str()).unwrap_or("");
    if token.is_empty() || token != expected_token {
        return Err(401);
    }
    let url = params.get("url").cloned().unwrap_or_default();
    if url.is_empty() {
        return Err(400);
    }
    let title = params.get("title").cloned().unwrap_or_default();
    Ok((url, title))
}

/// 启动本地 HTTP server。返回实际使用的 (port, token)。
/// `existing_token` 若提供则复用（支持跨重启持久化），否则新生成。
/// 端口从 BASE_PORT 开始尝试，被占时递增，最多重试 MAX_PORT_RETRIES 次。
pub fn start<F>(existing_token: Option<String>, on_add: F) -> std::io::Result<(u16, String)>
where
    F: Fn(String, String) + Send + Sync + 'static,
{
    let token = existing_token.unwrap_or_else(generate_token);

    let mut server = None;
    let mut actual_port: u16 = 0;
    for offset in 0..=MAX_PORT_RETRIES {
        let port = BASE_PORT + offset as u16;
        match Server::http(format!("127.0.0.1:{}", port)) {
            Ok(s) => {
                server = Some(s);
                actual_port = port;
                break;
            }
            Err(_) => continue,
        }
    }
    let server = server.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            format!("端口 {}-{} 均被占用", BASE_PORT, BASE_PORT + MAX_PORT_RETRIES as u16),
        )
    })?;

    let token_for_thread = token.clone();
    let on_add = Arc::new(on_add);
    thread::Builder::new()
        .name("links-http-server".into())
        .spawn(move || {
            for request in server.incoming_requests() {
                let method_is_get = matches!(request.method(), Method::Get);
                let (path, query) = split_path_and_query(request.url());
                let path = path.to_string();
                let query = query.to_string();

                // 处理 OPTIONS 预检请求
                if matches!(request.method(), Method::Options) {
                    let resp = Response::empty(204)
                        .with_header(cors_origin())
                        .with_header(cors_methods())
                        .with_header(cors_headers());
                    if let Err(e) = request.respond(resp) {
                        log::warn!("[http] OPTIONS 响应失败: {}", e);
                    }
                    continue;
                }

                match validate_add_request(method_is_get, &path, &query, &token_for_thread) {
                    Ok((url, title)) => {
                        let on_add = Arc::clone(&on_add);
                        on_add(url, title);
                        let resp = Response::from_string("{\"ok\":true}")
                            .with_header(content_type_json())
                            .with_header(cors_origin());
                        if let Err(e) = request.respond(resp) {
                            log::warn!("[http] 响应失败: {}", e);
                        }
                    }
                    Err(code) => {
                        log::warn!("[http] 请求被拒绝: {} {} → {}", request.method(), request.url(), code);
                        let resp = Response::from_string(format!("{{\"ok\":false,\"code\":{}}}", code))
                            .with_status_code(code)
                            .with_header(content_type_json())
                            .with_header(cors_origin());
                        if let Err(e) = request.respond(resp) {
                            log::warn!("[http] 响应失败: {}", e);
                        }
                    }
                }
            }
        })?;

    Ok((actual_port, token))
}

fn cors_origin() -> Header {
    Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap()
}
fn cors_methods() -> Header {
    Header::from_bytes(&b"Access-Control-Allow-Methods"[..], &b"GET, OPTIONS"[..]).unwrap()
}
fn cors_headers() -> Header {
    Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"Content-Type"[..]).unwrap()
}
fn content_type_json() -> Header {
    Header::from_bytes(&b"Content-Type"[..], &b"application/json; charset=utf-8"[..]).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_is_32_chars_alnum() {
        let t = generate_token();
        assert_eq!(t.len(), 32);
        assert!(t.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn token_is_unique_each_call() {
        let a = generate_token();
        let b = generate_token();
        assert_ne!(a, b);
    }

    #[test]
    fn parse_query_basic() {
        let m = parse_query("a=1&b=2");
        assert_eq!(m.get("a"), Some(&"1".to_string()));
        assert_eq!(m.get("b"), Some(&"2".to_string()));
    }

    #[test]
    fn parse_query_url_encoded() {
        let m = parse_query("url=https%3A%2F%2Fexample.com%2Fpath%3Fq%3Dhello&title=Test%20%26%20More");
        assert_eq!(m.get("url"), Some(&"https://example.com/path?q=hello".to_string()));
        assert_eq!(m.get("title"), Some(&"Test & More".to_string()));
    }

    #[test]
    fn parse_query_chinese_utf8() {
        let m = parse_query("title=%E4%B8%AD%E6%96%87");
        assert_eq!(m.get("title"), Some(&"中文".to_string()));
    }

    #[test]
    fn parse_query_empty() {
        let m = parse_query("");
        assert!(m.is_empty());
    }

    #[test]
    fn split_path_and_query_with_query() {
        let (p, q) = split_path_and_query("/add?url=foo&t=abc");
        assert_eq!(p, "/add");
        assert_eq!(q, "url=foo&t=abc");
    }

    #[test]
    fn split_path_and_query_no_query() {
        let (p, q) = split_path_and_query("/add");
        assert_eq!(p, "/add");
        assert_eq!(q, "");
    }

    #[test]
    fn validate_rejects_non_get() {
        let r = validate_add_request(false, "/add", "url=x&t=tok", "tok");
        assert_eq!(r, Err(405));
    }

    #[test]
    fn validate_rejects_wrong_path() {
        let r = validate_add_request(true, "/other", "url=x&t=tok", "tok");
        assert_eq!(r, Err(404));
    }

    #[test]
    fn validate_rejects_missing_token() {
        let r = validate_add_request(true, "/add", "url=x", "tok");
        assert_eq!(r, Err(401));
    }

    #[test]
    fn validate_rejects_wrong_token() {
        let r = validate_add_request(true, "/add", "url=x&t=bad", "tok");
        assert_eq!(r, Err(401));
    }

    #[test]
    fn validate_rejects_missing_url() {
        let r = validate_add_request(true, "/add", "t=tok", "tok");
        assert_eq!(r, Err(400));
    }

    #[test]
    fn validate_accepts_valid_request() {
        let r = validate_add_request(
            true,
            "/add",
            "url=https%3A%2F%2Fexample.com&title=Hello&t=tok",
            "tok",
        );
        assert_eq!(r, Ok(("https://example.com".to_string(), "Hello".to_string())));
    }

    #[test]
    fn validate_accepts_no_title() {
        let r = validate_add_request(true, "/add", "url=https%3A%2F%2Fexample.com&t=tok", "tok");
        assert_eq!(r, Ok(("https://example.com".to_string(), "".to_string())));
    }

    /// 端到端：实际启动服务并发送请求，确认 token 鉴权与回调触发
    #[test]
    fn end_to_end_server_accepts_and_invokes_callback() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Mutex;

        let counter = Arc::new(AtomicUsize::new(0));
        let captured: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(Vec::new()));
        let counter_clone = Arc::clone(&counter);
        let captured_clone = Arc::clone(&captured);

        let (port, token) = start(None, move |url, title| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            captured_clone.lock().unwrap().push((url, title));
        })
        .expect("server start");

        // 用同步 reqwest 在测试线程发请求；reqwest 已在依赖里
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let client = reqwest::Client::new();
            // 1. 合法请求
            let url = format!(
                "http://127.0.0.1:{}/add?url=https%3A%2F%2Fa.test&title=hi&t={}",
                port, token
            );
            let resp = client.get(&url).send().await.unwrap();
            assert_eq!(resp.status(), 200);

            // 2. 错误 token
            let url = format!("http://127.0.0.1:{}/add?url=https%3A%2F%2Fa.test&t=bad", port);
            let resp = client.get(&url).send().await.unwrap();
            assert_eq!(resp.status(), 401);

            // 3. 重复点击（这是核心场景，验证可重复触发）
            for _ in 0..5 {
                let url = format!(
                    "http://127.0.0.1:{}/add?url=https%3A%2F%2Fb.test&t={}",
                    port, token
                );
                let resp = client.get(&url).send().await.unwrap();
                assert_eq!(resp.status(), 200);
            }
        });

        assert_eq!(counter.load(Ordering::SeqCst), 6);
        let captured = captured.lock().unwrap();
        assert_eq!(captured[0], ("https://a.test".to_string(), "hi".to_string()));
        assert_eq!(captured[5].0, "https://b.test");
    }
}
