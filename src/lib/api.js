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
