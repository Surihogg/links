<script>
  import { onMount } from "svelte";
  import { linksStore, categoriesStore, tagsStore } from "./lib/stores/index.js";
  import SearchBar from "./lib/components/SearchBar.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import LinkList from "./lib/components/LinkList.svelte";
  import LinkForm from "./lib/components/LinkForm.svelte";
  import ExportDialog from "./lib/components/ExportDialog.svelte";

  let links = $derived($linksStore);
  let categories = $derived($categoriesStore);
  let tags = $derived($tagsStore);

  let selected_category = $state(null);
  let selected_tag = $state(null);
  let search_query = $state("");
  let show_add_form = $state(false);
  let edit_link = $state(null);
  let show_export = $state(false);
  let dark_mode = $state(false);

  onMount(() => {
    const saved = localStorage.getItem("links-dark-mode");
    dark_mode = saved === "true";
    load_data();
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
      await linksStore.search(search_query);
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
      await linksStore.search(query);
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

  function toggle_dark() {
    dark_mode = !dark_mode;
    localStorage.setItem("links-dark-mode", String(dark_mode));
  }

  let filtered_links = $derived(links.items);
  let current_title = $derived(
    selected_tag ? `标签: ${selected_tag}` :
    selected_category === "favorite" ? "特别关注" :
    selected_category != null ? categories.find(c => c.id === selected_category)?.name ?? "链接" :
    "全部链接"
  );
</script>

<div class={dark_mode ? "dark" : ""}>
  <div class="app-root">
    <div class="titlebar-drag" data-tauri-drag-region></div>
    <Sidebar
      {categories}
      tags={tags}
      selected_id={selected_category}
      selected_tag={selected_tag}
      onselect={on_category_select}
      onselect_tag={on_tag_select}
      oncreate={on_create_category}
      ontag_delete={refresh_current_view}
      dark={dark_mode}
      ontoggle_dark={toggle_dark}
      onexport={() => show_export = true}
    />

    <main class="main-content">
      <header class="content-header">
        <div class="header-left">
          <h2 class="header-title">{current_title}</h2>
          <span class="header-count">{filtered_links.length} 条</span>
        </div>
        <div class="header-right">
          <SearchBar bind:query={search_query} onsearch={on_search} />
        </div>
      </header>

      <LinkList
        links={filtered_links}
        {categories}
        loading={links.loading}
        highlight={search_query}
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

<style>
  .app-root {
    position: relative;
    display: flex;
    height: 100vh;
    background: var(--bg-0);
    color: var(--text-0);
    overflow: hidden;
    padding-top: 36px;
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    position: relative;
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
</style>
