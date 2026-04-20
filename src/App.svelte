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

  let selected_category = $state(null);
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
    if (selected_category === "favorite") {
      params.favorite_only = true;
    } else if (selected_category != null) {
      params.category_id = selected_category;
    }
    await linksStore.load(params);
  }

  function on_category_select(id) {
    selected_category = id;
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
  }

  async function on_toggle_favorite(link) {
    await linksStore.update({ id: link.id, is_favorite: !link.is_favorite });
  }

  async function on_delete_link(link) {
    if (confirm(`确定删除 "${link.title || link.url}" 吗？`)) {
      await linksStore.remove(link.id);
    }
  }

  async function on_create_category(payload) {
    await categoriesStore.create(payload);
  }

  function toggle_dark() {
    dark_mode = !dark_mode;
    localStorage.setItem("links-dark-mode", String(dark_mode));
  }

  let filtered_links = $derived(links.items);
</script>

<svelte:head>
  <title>Links</title>
</svelte:head>

<div class={dark_mode ? "dark" : ""}>
  <div class="h-screen flex flex-col" style="background:var(--color-bg);color:var(--color-text)">
    <header class="flex items-center justify-between px-4 py-2" style="background:var(--color-bg-secondary);border-bottom:1px solid var(--color-border)">
      <h1 class="text-sm font-bold" style="color:var(--color-text)">🔗 Links</h1>
      <div class="flex items-center gap-2">
        <button onclick={toggle_dark} class="px-2 py-1 rounded text-sm" style="color:var(--color-text-secondary)" title="切换主题">
          {dark_mode ? "☀" : "🌙"}
        </button>
        <button onclick={() => show_export = true} class="px-2 py-1 rounded text-sm" style="color:var(--color-text-secondary)" title="导出">📤</button>
      </div>
    </header>

    <SearchBar bind:query={search_query} onsearch={on_search} />

    <div class="flex flex-1 overflow-hidden">
      <Sidebar {categories} selected_id={selected_category} onselect={on_category_select} oncreate={on_create_category} />

      <main class="flex-1 flex flex-col overflow-hidden">
        <div class="px-4 py-2 flex items-center justify-between" style="border-bottom:1px solid var(--color-border)">
          <span class="text-xs" style="color:var(--color-text-secondary)">
            {filtered_links.length} 个链接
          </span>
        </div>

        <LinkList
          links={filtered_links}
          loading={links.loading}
          onedit={(link) => edit_link = link}
          ondelete={on_delete_link}
          ontoggle_favorite={on_toggle_favorite}
        />
      </main>
    </div>

    <footer class="p-3" style="background:var(--color-bg-secondary);border-top:1px solid var(--color-border)">
      <button
        onclick={() => show_add_form = true}
        class="w-full py-2.5 rounded-lg text-sm font-medium text-white"
        style="background:var(--color-primary)"
      >
        + 添加链接
      </button>
    </footer>
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
