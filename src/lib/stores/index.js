import { writable } from "svelte/store";
import * as api from "../api.js";

function createLinksStore() {
  const { subscribe, set, update } = writable({
    items: [],
    total: 0,
    page: 1,
    per_page: 30,
    loading: false,
    has_more: false,
  });

  return {
    subscribe,
    async load(params = {}) {
      update((s) => ({ ...s, loading: true }));
      try {
        const result = await api.listLinks(params);
        set({
          ...result,
          loading: false,
          has_more: result.items.length < result.total,
        });
      } catch (e) {
        console.error("[store] linksStore.load failed:", e);
        update((s) => ({ ...s, loading: false, items: [], total: 0, has_more: false }));
      }
    },
    async loadMore(params = {}) {
      update((s) => ({ ...s, loading: true }));
      const result = await api.listLinks(params);
      update((s) => ({
        ...result,
        items: [...s.items, ...result.items],
        loading: false,
        has_more: s.items.length + result.items.length < result.total,
      }));
    },
    async create(payload) {
      const link = await api.createLink(payload);
      update((s) => ({
        ...s,
        items: [link, ...s.items],
        total: s.total + 1,
      }));
      return link;
    },
    async update(payload) {
      const link = await api.updateLink(payload);
      update((s) => ({
        ...s,
        items: s.items.map((l) => (l.id === link.id ? link : l)),
      }));
      return link;
    },
    async remove(id) {
      await api.deleteLink(id);
      update((s) => ({
        ...s,
        items: s.items.filter((l) => l.id !== id),
        total: s.total - 1,
      }));
    },
    async search(params = {}, append = false) {
      update((s) => ({ ...s, loading: true }));
      try {
        const result = await api.searchLinks(params);
        if (append) {
          update((s) => ({
            ...result,
            items: [...s.items, ...result.items],
            loading: false,
            has_more: s.items.length + result.items.length < result.total,
          }));
        } else {
          set({
            ...result,
            loading: false,
            has_more: result.items.length < result.total,
          });
        }
      } catch {
        update((s) => ({ ...s, loading: false }));
      }
    },
  };
}

function createCategoriesStore() {
  const { subscribe, set, update } = writable([]);

  return {
    subscribe,
    async load() {
      try {
        const cats = await api.listCategories();
        set(cats);
      } catch (e) {
        console.error("[store] categoriesStore.load failed:", e);
        set([]);
      }
    },
    async create(payload) {
      const cat = await api.createCategory(payload);
      update((s) => [...s, cat]);
      return cat;
    },
    async update(payload) {
      const cat = await api.updateCategory(payload);
      return cat;
    },
    async remove(id) {
      await api.deleteCategory(id);
      update((s) => s.filter((c) => c.id !== id));
    },
  };
}

function createTagsStore() {
  const { subscribe, set, update } = writable([]);

  return {
    subscribe,
    async load() {
      try {
        const tags = await api.listTags();
        set(tags);
      } catch (e) {
        console.error("[store] tagsStore.load failed:", e);
        set([]);
      }
    },
    async remove(id) {
      await api.deleteTag(id);
      update((s) => s.filter((t) => t.id !== id));
    },
    async create(name) {
      const tag = await api.createTag(name);
      update((s) => [...s, tag].sort((a, b) => a.name.localeCompare(b.name)));
      return tag;
    },
  };
}

export const linksStore = createLinksStore();
export const categoriesStore = createCategoriesStore();
export const tagsStore = createTagsStore();
