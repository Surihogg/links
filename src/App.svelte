<script>
  import { onMount } from "svelte";
  import { linksStore, categoriesStore, tagsStore } from "./lib/stores/index.js";
  import * as api from "./lib/api.js";
  import SearchBar from "./lib/components/SearchBar.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import LinkList from "./lib/components/LinkList.svelte";
  import LinkForm from "./lib/components/LinkForm.svelte";
  import ExportDialog from "./lib/components/ExportDialog.svelte";
  import SettingsDialog from "./lib/components/SettingsDialog.svelte";

  let is_macos = $state(false);

  let links = $derived($linksStore);
  let categories = $derived($categoriesStore);
  let tags = $derived($tagsStore);

  let selected_category = $state(null);
  let selected_tag = $state(null);
  let search_query = $state("");
  let show_add_form = $state(false);
  let edit_link = $state(null);
  let show_export = $state(false);
  let show_settings = $state(false);
  let dark_mode = $state(false);
  let show_close_dialog = $state(false);

  onMount(async () => {
    const darkVal = await api.getSetting("dark-mode");
    dark_mode = darkVal === "true";
    is_macos = /mac/i.test(navigator.userAgentData?.platform ?? navigator.platform);
    load_data();

    const { getCurrentWindow, LogicalSize } = await import("@tauri-apps/api/window");
    const mainWindow = getCurrentWindow();

    const savedSize = await api.getSetting("window-size");
    if (savedSize) {
      try {
        const { width, height } = JSON.parse(savedSize);
        await mainWindow.setSize(new LogicalSize(width, height));
      } catch (e) {}
    }

    const splash = document.getElementById("splash");
    if (splash) {
      splash.classList.add("fade-out");
      setTimeout(() => splash.remove(), 300);
    }

    let resize_restore_done = false;
    setTimeout(() => { resize_restore_done = true; }, 2000);

    let resize_timer;
    window.addEventListener("resize", () => {
      clearTimeout(resize_timer);
      if (!resize_restore_done) return;
      resize_timer = setTimeout(async () => {
        try {
          const { getCurrentWindow } = await import("@tauri-apps/api/window");
          const win = getCurrentWindow();
          const physicalSize = await win.innerSize();
          const scaleFactor = await win.scaleFactor();
          const logical = physicalSize.toLogical(scaleFactor);
          await api.setSetting("window-size", JSON.stringify({
            width: Math.round(logical.width),
            height: Math.round(logical.height)
          }));
        } catch (e) {}
      }, 500);
    });

    await mainWindow.onCloseRequested(async (event) => {
      event.preventDefault();
      const behavior = (await api.getSetting("close-behavior")) || "ask";
      if (behavior === "exit") {
        await mainWindow.destroy();
      } else if (behavior === "tray") {
        await mainWindow.hide();
      } else {
        show_close_dialog = true;
      }
    });
  });

  async function load_data() {
    await Promise.all([
      categoriesStore.load(),
      tagsStore.load(),
      load_links(),
    ]);
  }

  async function load_links() {
    const params = {};
    if (selected_tag) {
      params.tag = selected_tag;
    } else if (selected_category === "favorite") {
      params.favorite_only = true;
    } else if (selected_category != null) {
      params.category_id = selected_category;
    }
    await linksStore.load(params);
  }

  async function refresh_current_view() {
    if (search_query.trim()) {
      await linksStore.search({ query: search_query, per_page: 30, ...build_filter_params() });
    } else {
      await load_links();
    }
    await categoriesStore.load();
  }

  function on_category_select(id) {
    selected_category = id;
    selected_tag = null;
    search_query = "";
    load_links();
  }

  function on_tag_select(tag) {
    if (selected_tag === tag) {
      selected_tag = null;
    } else {
      selected_tag = tag;
      selected_category = null;
    }
    search_query = "";
    load_links();
  }

  async function on_search(query) {
    if (query.trim()) {
      await linksStore.search({ query, per_page: 30, ...build_filter_params() });
    } else {
      await load_links();
    }
  }

  async function on_save_link(data) {
    if (data.id) {
      await linksStore.update(data);
    } else {
      await linksStore.create(data);
    }
    show_add_form = false;
    edit_link = null;
    await refresh_current_view();
    await tagsStore.load();
  }

  async function on_toggle_favorite(link) {
    await linksStore.update({ id: link.id, is_favorite: !link.is_favorite });
    await refresh_current_view();
  }

  async function on_delete_link(link) {
    await linksStore.remove(link.id);
    await refresh_current_view();
  }

  async function on_create_category(payload) {
    await categoriesStore.create(payload);
  }

  async function on_delete_category(id) {
    await categoriesStore.remove(id);
    // 如果当前正在查看被删除的分组，切回"全部链接"
    // （该分组的子分组会因 schema 的 ON DELETE SET NULL 自动提升为根级分组，不必处理）
    if (selected_category === id) {
      selected_category = null;
    }
    await refresh_current_view();
  }

  async function on_delete_tag(id) {
    await tagsStore.remove(id);
    if (selected_tag) {
      selected_tag = null;
    }
    await refresh_current_view();
  }

  async function on_create_tag(name) {
    await tagsStore.create(name);
  }

  let importing = $state(false);

  async function on_import_bookmarks() {
    importing = true;
    try {
      const count = await api.importBookmarks();
      if (count > 0) {
        await load_data();
      }
    } finally {
      importing = false;
    }
  }

  async function close_to_tray() {
    show_close_dialog = false;
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().hide();
  }

  async function close_exit() {
    show_close_dialog = false;
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().destroy();
  }

  async function toggle_dark() {
    dark_mode = !dark_mode;
    await api.setSetting("dark-mode", String(dark_mode));
  }

  let filtered_links = $derived(links.items);
  let has_more = $derived(links.has_more);
  let current_page = $derived(links.page);
  let total_count = $derived(links.total);
  let current_title = $derived(
    search_query.trim() ? `搜索: ${search_query}` :
    selected_tag ? `标签: ${selected_tag}` :
    selected_category === "favorite" ? "特别关注" :
    selected_category != null ? categories.find(c => c.id === selected_category)?.name ?? "链接" :
    "全部链接"
  );

  function build_filter_params() {
    const params = {};
    if (selected_tag) {
      params.tag = selected_tag;
    } else if (selected_category === "favorite") {
      params.favorite_only = true;
    } else if (selected_category != null) {
      params.category_id = selected_category;
    }
    return params;
  }

  async function load_more() {
    if (links.loading || !has_more) return;
    const next_page = current_page + 1;
    if (search_query.trim()) {
      await linksStore.search({ query: search_query, page: next_page, per_page: 30, ...build_filter_params() }, true);
    } else {
      await linksStore.loadMore({ page: next_page, per_page: 30, ...build_filter_params() });
    }
  }
</script>

<div class={dark_mode ? "dark" : ""}>
  <div class="app-root" class:has-titlebar={is_macos}>
    {#if is_macos}
      <div class="titlebar-drag" data-tauri-drag-region></div>
    {/if}
    <Sidebar
      {categories}
      tags={tags}
      selected_id={selected_category}
      selected_tag={selected_tag}
      onselect={on_category_select}
      onselect_tag={on_tag_select}
      oncreate={on_create_category}
      ondelete_cat={on_delete_category}
      ontag_delete={on_delete_tag}
      oncreate_tag={on_create_tag}
      dark={dark_mode}
      ontoggle_dark={toggle_dark}
      onexport={() => show_export = true}
      onimport={on_import_bookmarks}
      onsettings={() => show_settings = true}
      {importing}
    />

    <main class="main-content">
      <header class="content-header">
        {#if importing}
          <div class="import-banner">
            <span class="spinner-sm"></span>
            正在导入书签，请稍等...
          </div>
        {:else}
          <div class="header-left">
            <h2 class="header-title">{current_title}</h2>
            <span class="header-count">{total_count} 条</span>
          </div>
        {/if}
        <div class="header-right">
          <SearchBar bind:query={search_query} onsearch={on_search} />
        </div>
      </header>

      <LinkList
        links={filtered_links}
        {categories}
        loading={links.loading}
        highlight={search_query}
        has_more={has_more}
        onloadmore={load_more}
        onedit={(link) => edit_link = link}
        ondelete={on_delete_link}
        ontoggle_favorite={on_toggle_favorite}
      />

      <button class="fab" onclick={() => show_add_form = true} title="添加链接">
        <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <line x1="10" y1="4" x2="10" y2="16"/>
          <line x1="4" y1="10" x2="16" y2="10"/>
        </svg>
      </button>
    </main>
  </div>

  {#if show_add_form}
    <LinkForm categories={categories} onsave={on_save_link} oncancel={() => show_add_form = false} />
  {/if}

  {#if edit_link}
    <LinkForm link={edit_link} categories={categories} onsave={on_save_link} oncancel={() => edit_link = null} />
  {/if}

  {#if show_export}
    <ExportDialog onclose={() => show_export = false} />
  {/if}

  {#if show_settings}
    <SettingsDialog onclose={() => show_settings = false} />
  {/if}

  {#if show_close_dialog}
    <div class="close-overlay" onclick={() => show_close_dialog = false}>
      <div class="close-dialog" onclick={(e) => e.stopPropagation()}>
        <p class="close-title">要走了吗？</p>
        <p class="close-desc">选择一下你希望的离开方式~</p>
        <div class="close-actions">
          <button class="btn btn-primary" style="flex:1" onclick={close_to_tray}>最小化到托盘</button>
          <button class="btn btn-secondary" style="flex:1" onclick={close_exit}>退出应用</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .app-root {
    position: relative;
    display: flex;
    height: 100vh;
    background: var(--bg-0);
    color: var(--text-0);
    overflow: hidden;
  }

  .app-root.has-titlebar {
    padding-top: 36px;
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    position: relative;
    z-index: 1;
  }

  .content-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 24px;
    border-bottom: 1px solid var(--border-0);
    gap: 16px;
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: baseline;
    gap: 8px;
    min-width: 0;
  }

  .header-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-0);
    white-space: nowrap;
  }

  .header-count {
    font-size: 12px;
    color: var(--text-3);
    flex-shrink: 0;
  }

  .header-right {
    flex-shrink: 0;
  }

  .fab {
    position: absolute;
    bottom: 24px;
    right: 24px;
    width: 44px;
    height: 44px;
    border-radius: var(--radius-lg);
    background: var(--accent);
    color: white;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: var(--shadow-lg);
    transition: all var(--transition);
    z-index: 10;
  }

  .fab:hover {
    background: var(--accent-hover);
    transform: scale(1.05);
    box-shadow: var(--shadow-xl);
  }

  .fab:active {
    transform: scale(0.97);
  }

  .titlebar-drag {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 36px;
    background: var(--bg-0);
    z-index: 100;
    pointer-events: auto;
  }

  .dark .titlebar-drag {
    background: #0a0a0b;
  }

  .close-overlay {
    position: fixed;
    inset: 0;
    z-index: 200;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
  }

  .close-dialog {
    background: var(--bg-0);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-xl);
    border: 1px solid var(--border-0);
    padding: 24px;
    min-width: 300px;
  }

  .close-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-0);
    margin-bottom: 4px;
  }

  .close-desc {
    font-size: 13px;
    color: var(--text-2);
    margin-bottom: 20px;
  }

  .close-actions {
    display: flex;
    gap: 8px;
  }

  .import-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--text-2);
  }

  .spinner-sm {
    width: 14px;
    height: 14px;
    border: 1.5px solid var(--border-1);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin { to { transform: rotate(360deg); } }
</style>
