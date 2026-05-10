// 链接分享格式化：把链接渲染为 url / markdown / html 三种文本格式。
//
// 此前完全内嵌在 LinkCard 的 copy_as 函数里。抽到 utils 后便于将来在 Spotlight
// 或菜单栏分享时复用，且更便于测试。

/**
 * 转义 Markdown 中的 [] / *_`~ 字符，避免链接文本里的 markdown 影响渲染。
 * @param {string} text
 * @returns {string}
 */
function escapeMarkdown(text) {
  return text.replace(/[\[\]]/g, "\\$&").replace(/[*_`~]/g, "\\$&");
}

/**
 * HTML 实体转义。
 * @param {string} text
 * @returns {string}
 */
function escapeHtml(text) {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/**
 * 按指定格式输出链接的可复制字符串。
 *
 * @param {{ url: string, title?: string }} link
 * @param {"url"|"markdown"|"html"} format
 * @returns {string}
 */
export function formatLinkAs(link, format) {
  const title = link.title || link.url;
  if (format === "url") return link.url;
  if (format === "markdown") {
    return `[${escapeMarkdown(title)}](${link.url})`;
  }
  // 默认 html
  return `<a href="${escapeHtml(link.url)}">${escapeHtml(title)}</a>`;
}
