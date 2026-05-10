// 时间格式化工具：链接的"最后打开时间"在 LinkCard 与 Spotlight 都需要展示，
// 此前两处各写一份，文案细节略有差异。统一抽到这里，差异通过 options 控制。

/**
 * 将 Unix 时间戳格式化为绝对时间字符串：YYYY-MM-DD HH:mm。
 * @param {number|null|undefined} ts Unix 秒级时间戳
 * @returns {string} 格式化后的字符串；ts 为空时返回空串
 */
export function formatAbsoluteTime(ts) {
  if (!ts) return "";
  const d = new Date(ts * 1000);
  const y = d.getFullYear();
  const m = (d.getMonth() + 1).toString().padStart(2, "0");
  const day = d.getDate().toString().padStart(2, "0");
  const h = d.getHours().toString().padStart(2, "0");
  const min = d.getMinutes().toString().padStart(2, "0");
  return `${y}-${m}-${day} ${h}:${min}`;
}

/**
 * 将 Unix 时间戳格式化为相对时间字符串。
 * 7 天内显示"X 分钟前 / X 小时前 / X 天前"，超过 7 天回退到指定格式。
 *
 * @param {number|null|undefined} ts Unix 秒级时间戳
 * @param {object} [options]
 * @param {boolean} [options.spaceBeforeUnit=true] "X 分钟前" 中数字与单位之间是否空格
 *   （主窗口风格 true，Spotlight 风格 false）
 * @param {"absolute"|"short"} [options.fallback="absolute"] 超过 7 天的回退格式
 *   - "absolute"：YYYY-MM-DD HH:mm（Spotlight 用）
 *   - "short"：MM-DD HH:mm（LinkCard 用）
 * @returns {string}
 */
export function formatRelativeTime(ts, options = {}) {
  if (!ts) return "";
  const { spaceBeforeUnit = true, fallback = "absolute" } = options;
  const now = Math.floor(Date.now() / 1000);
  const diff = now - ts;
  const sep = spaceBeforeUnit ? " " : "";

  if (diff < 60) return "刚刚";
  if (diff < 3600) return `${Math.floor(diff / 60)}${sep}分钟前`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}${sep}小时前`;
  if (diff < 604800) return `${Math.floor(diff / 86400)}${sep}天前`;

  if (fallback === "short") {
    const d = new Date(ts * 1000);
    const m = (d.getMonth() + 1).toString().padStart(2, "0");
    const day = d.getDate().toString().padStart(2, "0");
    const h = d.getHours().toString().padStart(2, "0");
    const min = d.getMinutes().toString().padStart(2, "0");
    return `${m}-${day} ${h}:${min}`;
  }
  return formatAbsoluteTime(ts);
}
