import { writable } from "svelte/store";
import * as api from "../api.js";

function createLinksStore() {
  const { subscribe, set, update } = writable({
    items: [],
    total: 0,
    page: 1,
    per_page: 30,
    loading: false,
  });

  return {
    subscribe,
    async load(params = {}) {
      update((s) => ({ ...s, loading: true }));
      const result = await api.listLinks(params);
      set({ ...result, loading: false });
    },
    async loadMore(params = {}) {
      update((s) => ({ ...s, loading: true }));
      const result = await api.listLinks(params);
      update((s) => ({
        ...result,
        items: [...s.items, ...result.items],
        loading: false,
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
    async search(query) {
      update((s) => ({ ...s, loading: true }));
      const items = await api.searchLinks(query);
      update((s) => ({ ...s, items, total: items.length, loading: false }));
    },
  };
}

function createCategoriesStore() {
  const { subscribe, set, update } = writable([]);

  return {
    subscribe,
    async load() {
      const cats = await api.listCategories();
      set(cats);
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
  const { subscribe, set } = writable([]);

  return {
    subscribe,
    async load() {
      const tags = await api.listTags();
      set(tags);
    },
  };
}

export const linksStore = createLinksStore();
export const categoriesStore = createCategoriesStore();
export const tagsStore = createTagsStore();
