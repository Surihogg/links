// URL 工具：取域名供卡片/列表/Spotlight 展示。
// LinkCard 与 Spotlight 此前各有一份，差异仅在是否去除 "www."。

/**
 * 从 URL 提取域名。解析失败时回退到原值（Spotlight 风格）或空串（LinkCard 风格）。
 *
 * @param {string} url
 * @param {object} [options]
 * @param {boolean} [options.stripWww=false] 是否去除 "www." 前缀
 * @param {"original"|"empty"} [options.fallback="empty"] 解析失败时的回退
 * @returns {string}
 */
export function getDomain(url, options = {}) {
  const { stripWww = false, fallback = "empty" } = options;
  try {
    const host = new URL(url).hostname;
    return stripWww ? host.replace(/^www\./, "") : host;
  } catch {
    return fallback === "original" ? url : "";
  }
}
