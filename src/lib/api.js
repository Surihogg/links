import { invoke } from "@tauri-apps/api/core";

// —— 链接 CRUD ——

export async function listLinks(params = {}) {
  return invoke("list_links", { params });
}

export async function createLink(payload) {
  return invoke("create_link", { payload });
}

export async function updateLink(payload) {
  return invoke("update_link", { payload });
}

export async function deleteLink(id) {
  return invoke("delete_link", { id });
}

export async function searchLinks(params = {}) {
  return invoke("search_links", { params });
}

export async function getLinksStats() {
  return invoke("get_links_stats");
}

export async function checkDuplicate(url, excludeId = null) {
  return invoke("check_duplicate", { url, exclude_id: excludeId });
}

export async function checkLinkStatus(url) {
  return invoke("check_link_status", { url });
}

// —— 分组 CRUD ——

export async function listCategories() {
  return invoke("list_categories");
}

export async function createCategory(payload) {
  return invoke("create_category", { payload });
}

export async function updateCategory(payload) {
  return invoke("update_category", { payload });
}

export async function deleteCategory(id) {
  return invoke("delete_category", { id });
}

// —— 标签 CRUD ——

export async function listTags() {
  return invoke("list_tags");
}

export async function deleteTag(id) {
  return invoke("delete_tag", { id });
}

export async function createTag(name) {
  return invoke("create_tag", { name });
}

export async function updateTag(payload) {
  return invoke("update_tag", { payload });
}

export async function autocompleteTags(prefix) {
  return invoke("autocomplete_tags", { prefix });
}

// —— 抓取与系统 ——

export async function fetchMeta(url) {
  return invoke("fetch_metadata", { url });
}

export async function openUrl(url) {
  return invoke("open_url", { url });
}

export async function openDataDir() {
  return invoke("open_data_dir");
}

export async function saveFile(content, filename) {
  return invoke("save_file", { content, filename });
}

export async function copyToClipboard(content) {
  return invoke("copy_to_clipboard", { content });
}

export async function exitApp() {
  return invoke("exit_app");
}

// —— 导入导出 ——

export async function exportLinks(params) {
  return invoke("export_links", { params });
}

export async function importBookmarks() {
  return invoke("import_bookmarks");
}

// —— 配置 ——
// 配置统一持久化到 config.json，禁止使用 localStorage

export async function getSetting(key) {
  return invoke("get_setting", { key });
}

export async function setSetting(key, value) {
  return invoke("set_setting", { key, value });
}

// —— 快捷键 ——

export async function getShortcut() {
  return invoke("get_shortcut");
}

export async function setShortcut(shortcut) {
  return invoke("set_shortcut", { shortcut });
}

export async function getMainShortcut() {
  return invoke("get_main_shortcut");
}

export async function setMainShortcut(shortcut) {
  return invoke("set_main_shortcut", { shortcut });
}

export async function getSpotlightShortcut() {
  return invoke("get_spotlight_shortcut");
}

export async function setSpotlightShortcut(shortcut) {
  return invoke("set_spotlight_shortcut", { shortcut });
}

export async function getHideShortcut() {
  return invoke("get_hide_shortcut");
}

export async function setHideShortcut(shortcut) {
  return invoke("set_hide_shortcut", { shortcut });
}

// —— 自动启动（Tauri 插件，懒加载减小 quick-add 启动开销） ——

export async function enableAutostart() {
  const { enable } = await import("@tauri-apps/plugin-autostart");
  return enable();
}

export async function disableAutostart() {
  const { disable } = await import("@tauri-apps/plugin-autostart");
  return disable();
}

export async function isAutostartEnabled() {
  const { isEnabled } = await import("@tauri-apps/plugin-autostart");
  return isEnabled();
}

// —— 自动更新 ——

export async function getSystemProxy() {
  return invoke("get_system_proxy");
}

export async function checkUpdate() {
  const { check } = await import("@tauri-apps/plugin-updater");
  const proxy = await getSystemProxy();
  const result = await Promise.race([
    check(proxy ? { proxy } : undefined),
    new Promise((_, reject) =>
      setTimeout(() => reject(new Error("检查更新超时，请检查网络连接")), 15000)
    ),
  ]);
  return result;
}

export async function downloadAndInstallUpdate(update, onProgress) {
  return update.downloadAndInstall(onProgress);
}

export async function relaunchApp() {
  const { relaunch } = await import("@tauri-apps/plugin-process");
  return relaunch();
}

// 从 GitHub Releases API 获取指定 tag 的 release notes（Markdown 正文）
export async function fetchReleaseNotes(tag) {
  const res = await fetch(`https://api.github.com/repos/Surihogg/links/releases/tags/${tag}`);
  if (!res.ok) return null;
  const data = await res.json();
  return data.body || "";
}

// —— Deep link / 浏览器扩展 ——

export async function popPendingDeepLink() {
  return invoke("pop_pending_deep_link");
}

export async function checkStartupDeepLink() {
  return invoke("check_startup_deep_link");
}

export async function getLocalServerInfo() {
  return invoke("get_local_server_info");
}

