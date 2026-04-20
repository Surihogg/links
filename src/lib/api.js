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

export async function searchLinks(query) {
  return invoke("links_search", { query });
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

export async function autocompleteTags(prefix) {
  return invoke("tags_autocomplete", { prefix });
}

export async function exportLinks(params) {
  return invoke("export_links", { params });
}
