import { listen } from "@tauri-apps/api/event";
import { linksStore, categoriesStore, tagsStore } from "../stores/index.js";

export async function registerEventListeners(mainWindow, cleanup, callbacks) {
  const {
    getSearchBar,
    resetFilters,
    refreshCurrentView,
    onSpotlightLocate,
    onEditExistingLink,
  } = callbacks;

  cleanup.add(await listen("main-shown", () => {
    getSearchBar()?.focus();
    refreshCurrentView();
    tagsStore.load();
  }));

  cleanup.add(await listen("main-hidden", () => {
    resetFilters();
  }));

  cleanup.add(await listen("links-changed", () => {
    refreshCurrentView();
    categoriesStore.load();
    tagsStore.load();
  }));

  cleanup.add(await listen("link-broken-changed", (e) => {
    const { id, is_broken } = e.payload || {};
    if (typeof id === "number") {
      linksStore.patchItem(id, { is_broken: !!is_broken });
    }
  }));

  cleanup.add(await listen("link-meta-changed", (e) => {
    const payload = e.payload || {};
    const { id, ...patch } = payload;
    if (typeof id === "number") {
      linksStore.patchItem(id, patch);
    }
  }));

  cleanup.add(await listen("spotlight-locate", onSpotlightLocate));

  cleanup.add(await listen("edit-existing-link", (event) => {
    if (event.payload?.id) onEditExistingLink(event.payload);
  }));
}
