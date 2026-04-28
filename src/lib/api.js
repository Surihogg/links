import { invoke } from "@tauri-apps/api/core";

export async function listLinks(params = {}) {
  return invoke("links_list", { params });
}

export async function createLink(payload) {
  return invoke("links_create", { payload });
}

export async function updateLink(payload) {
  return invoke("links_update", { payload });
}

export async function deleteLink(id) {
  return invoke("links_delete", { id });
}

export async function searchLinks(params = {}) {
  return invoke("links_search", { params });
}

export async function listCategories() {
  return invoke("categories_list");
}

export async function createCategory(payload) {
  return invoke("categories_create", { payload });
}

export async function updateCategory(payload) {
  return invoke("categories_update", { payload });
}

export async function deleteCategory(id) {
  return invoke("categories_delete", { id });
}

export async function listTags() {
  return invoke("tags_list");
}

export async function deleteTag(id) {
  return invoke("tags_delete", { id });
}

export async function createTag(name) {
  return invoke("tags_create", { name });
}

export async function updateTag(payload) {
  return invoke("tags_update", { payload });
}

export async function autocompleteTags(prefix) {
  return invoke("tags_autocomplete", { prefix });
}

export async function exportLinks(params) {
  return invoke("export_links", { params });
}

export async function openUrl(url) {
  return invoke("open_url", { url });
}

export async function saveFile(content, filename) {
  return invoke("save_file", { content, filename });
}

export async function fetchMeta(url) {
  return invoke("fetch_metadata", { url });
}

export async function importBookmarks() {
  return invoke("import_bookmarks");
}

// Settings: migrate from localStorage to SQLite-backed settings table
export async function getSetting(key) {
  return invoke("get_setting", { key });
}

export async function setSetting(key, value) {
  return invoke("set_setting", { key, value });
}

export async function getShortcut() {
  return invoke("get_shortcut");
}

export async function setShortcut(shortcut) {
  return invoke("set_shortcut", { shortcut });
}

// Main window shortcut management
export async function getMainShortcut() {
  return invoke("get_main_shortcut");
}

export async function setMainShortcut(shortcut) {
  return invoke("set_main_shortcut", { shortcut });
}

// Frontend helpers for link sharing and status checks
export async function copyToClipboard(content) {
  return invoke("copy_to_clipboard", { content });
}

export async function checkDuplicate(url, excludeId = null) {
  return invoke("check_duplicate", { url, exclude_id: excludeId });
}

export async function checkLinkStatus(url) {
  return invoke("check_link_status", { url });
}

export async function exitApp() {
  return invoke("exit_app");
}

// Autostart
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

// Updater
export async function getSystemProxy() {
  return invoke("get_system_proxy");
}

export async function checkUpdate() {
  const { check } = await import("@tauri-apps/plugin-updater");
  const proxy = await getSystemProxy();
  return check(proxy ? { proxy } : undefined);
}

export async function downloadAndInstallUpdate(update, onProgress) {
  return update.downloadAndInstall(onProgress);
}

export async function relaunchApp() {
  const { relaunch } = await import("@tauri-apps/plugin-process");
  return relaunch();
}

export async function popPendingDeepLink() {
  return invoke("pop_pending_deep_link");
}

export async function getLocalServerInfo() {
  return invoke("get_local_server_info");
}
