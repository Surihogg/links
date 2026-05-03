<script>
  import { onMount } from "svelte";
  import { searchLinks, openUrl, getSetting, listCategories } from "../lib/api.js";
  import { waitForBackendReady } from "../lib/ready.js";
  import { emit, listen } from "@tauri-apps/api/event";
  import { getCurrentWindow, LogicalSize, LogicalPosition } from "@tauri-apps/api/window";

  const WIN_WIDTH = 560;
  const WIN_MIN_HEIGHT = 66;
  const WIN_MAX_HEIGHT = 420;

  let query = $state("");
  let results = $state([]);
  let selected_index = $state(-1);
  let searching = $state(false);
  let has_searched = $state(false);
  let search_timer = null;
  let input_el;
  let dark_mode = $state(false);
  let theme_mode = $state("system");
  let categories = $state([]);
  let mouse_moved = $state(false);
  let spotlight_ready = $state(false);
  let hiding = false;

  function category_name(cid) {
    if (cid == null) return null;
    const cat = categories.find(c => c.id === cid);
    return cat?.name || null;
  }

  async function resize_window() {
    await new Promise(r => requestAnimationFrame(r));
    await new Promise(r => requestAnimationFrame(r));
    const el = document.querySelector(".results-container");
    if (!el) return;
    const search_h = 78;
    const content_h = el.scrollHeight;
    const total_h = search_h + content_h;
    const h = Math.max(WIN_MIN_HEIGHT, Math.min(total_h, WIN_MAX_HEIGHT));
    getCurrentWindow().setSize(new LogicalSize(WIN_WIDTH, Math.ceil(h)));
  }

  function apply_theme(mode) {
    if (mode !== undefined) theme_mode = mode;
    if (theme_mode === "system") {
      dark_mode = window.matchMedia("(prefers-color-scheme: dark)").matches;
    } else {
      dark_mode = theme_mode === "dark";
    }
    const root = document.documentElement;
    root.classList.add("no-transition");
    root.classList.toggle("dark", dark_mode);
    root.offsetHeight;
    requestAnimationFrame(() => root.classList.remove("no-transition"));
  }

  function extract_domain(url) {
    try { return new URL(url).hostname; }
    catch { return url; }
  }

  async function do_search(q) {
    if (!q.trim()) {
      results = [];
      has_searched = false;
      resize_window();
      return;
    }
    searching = true;
    try {
      const res = await searchLinks({ query: q.trim() });
      results = res.items || [];
      has_searched = true;
    } catch {
      results = [];
      has_searched = true;
    }
    searching = false;
    selected_index = -1;
    mouse_moved = false;
    resize_window();
  }

  function on_input() {
    clearTimeout(search_timer);
    search_timer = setTimeout(() => do_search(query), 150);
  }

  function clear_input() {
    query = "";
    results = [];
    has_searched = false;
    selected_index = -1;
    searching = false;
    clearTimeout(search_timer);
    input_el?.focus();
    resize_window();
  }

  async function hide_window() {
    if (hiding) return;
    hiding = true;
    spotlight_ready = false;
    await new Promise(r => requestAnimationFrame(r));
    if (!hiding) return;
    await getCurrentWindow().setSize(new LogicalSize(WIN_WIDTH, WIN_MIN_HEIGHT));
    if (!hiding) return;
    await getCurrentWindow().hide();
    hiding = false;
  }

  function scroll_to_selected() {
    requestAnimationFrame(() => {
      const el = document.querySelector(`.result-item[data-index="${selected_index}"]`);
      el?.scrollIntoView({ block: "nearest" });
    });
  }

  function handle_keydown(e) {
    if (e.key === "Escape") {
      e.preventDefault();
      hide_window();
      return;
    }
    if (results.length === 0) return;
    if (e.key === "ArrowDown") {
      e.preventDefault();
      selected_index = selected_index < 0 ? 0 : Math.min(selected_index + 1, results.length - 1);
      scroll_to_selected();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (selected_index >= 0) {
        selected_index = Math.max(selected_index - 1, 0);
        scroll_to_selected();
      }
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (selected_index < 0) return;
      const link = results[selected_index];
      if (!link) return;
      if (e.metaKey || e.ctrlKey) {
        emit("spotlight-locate", { link_id: link.id });
      } else {
        openUrl(link.url);
      }
      hide_window();
    }
  }

  function select_and_open(link) {
    openUrl(link.url);
    hide_window();
  }

  onMount(async () => {
    await waitForBackendReady();

    let saved = await getSetting("theme-mode");
    if (!saved) {
      const legacyDark = await getSetting("dark-mode");
      saved = legacyDark === "true" ? "dark" : (legacyDark === "false" ? "light" : "system");
    }
    apply_theme(saved || "system");
    document.documentElement.classList.add("theme-ready");

    listCategories().then(c => categories = c);

    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    function on_system_theme() {
      if (theme_mode === "system") apply_theme();
    }
    if (mq) mq.addEventListener("change", on_system_theme);

    const unlistenTheme = await listen("theme-changed", (e) => {
      apply_theme(e.payload);
    });

    const unlistenShown = await listen("spotlight-shown", () => {
      hiding = false;
      spotlight_ready = false;
      const { availWidth, availHeight } = window.screen;
      getCurrentWindow().setPosition(new LogicalPosition(
        Math.round((availWidth - WIN_WIDTH) / 2),
        Math.round(availHeight * 0.25)
      ));
      getCurrentWindow().setSize(new LogicalSize(WIN_WIDTH, WIN_MIN_HEIGHT));
      query = "";
      results = [];
      has_searched = false;
      selected_index = -1;
      searching = false;
      mouse_moved = false;
      clearTimeout(search_timer);
      setTimeout(() => input_el?.focus(), 50);
      requestAnimationFrame(() => {
        spotlight_ready = true;
      });
    });

    const unlistenFocus = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (!focused) hide_window();
    });

    window.addEventListener("keydown", handle_keydown);

    spotlight_ready = true;

    return () => {
      window.removeEventListener("keydown", handle_keydown);
      unlistenTheme();
      unlistenShown();
      unlistenFocus();
      clearTimeout(search_timer);
    };
  });
</script>

<div class="spotlight" class:hidden={!spotlight_ready}>
  <div class="search-area">
    <div class="input-wrap">
      <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
      </svg>
      <input
        bind:this={input_el}
        bind:value={query}
        oninput={on_input}
        placeholder="搜搜看~"
        class="search-input"
        autocomplete="off"
        spellcheck="false"
      />
      {#if searching}
        <span class="spinner"></span>
      {/if}
    </div>
  </div>

  <div class="results-container" class:scrollable={results.length > 0} onmousemove={() => mouse_moved = true}>
    {#if has_searched && results.length === 0}
      <div class="empty-state">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
          <line x1="8" y1="11" x2="14" y2="11"/>
        </svg>
        <span>没有找到相关链接</span>
      </div>
    {:else if results.length > 0}
      {#each results as link, i (link.id)}
        <div
          class="result-item {i === selected_index ? 'selected' : ''}"
          data-index={i}
          onclick={() => select_and_open(link)}
          onmouseenter={() => { if (mouse_moved) selected_index = i; }}
        >
          <div class="result-main">
            <span class="result-title">{link.title || link.url}</span>
            <span class="result-domain">{extract_domain(link.url)}</span>
          </div>
          {#if category_name(link.category_id) || (link.tags && link.tags.length > 0)}
            <div class="result-tags">
              {#if category_name(link.category_id)}
                <span class="result-cat">{category_name(link.category_id)}</span>
              {/if}
              {#each link.tags?.slice(0, 3) || [] as tag}
                <span class="tag-chip">{tag}</span>
              {/each}
              {#if link.tags && link.tags.length > 3}
                <span class="tag-more">+{link.tags.length - 3}</span>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    {:else if !has_searched}
      <div class="hint">输入关键词搜索已保存的链接</div>
    {/if}
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    height: 100vh;
    width: 100vw;
    background: transparent;
  }

  .spotlight {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-0);
    border: 1px solid var(--border-0);
    overflow: hidden;
  }

  .spotlight.hidden {
    opacity: 0;
  }

  .search-area {
    padding: 14px 16px 20px;
    flex-shrink: 0;
  }

  .input-wrap {
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 12px;
    flex-shrink: 0;
    color: var(--text-3);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    border: none;
    outline: none;
    background: var(--bg-1);
    color: var(--text-0);
    font-size: 14px;
    padding: 8px 36px 8px 36px;
    border-radius: 12px;
  }

  .search-input::placeholder {
    color: var(--text-3);
  }

  .spinner {
    position: absolute;
    right: 12px;
    width: 16px;
    height: 16px;
    border: 2px solid var(--border-1);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  .results-container {
    flex: 1;
    min-height: 0;
    padding: 0 8px 8px;
  }

  .results-container.scrollable {
    overflow-y: auto;
  }

  .result-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px 12px;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: background var(--transition);
  }

  .result-item.selected {
    background: var(--bg-2);
  }

  .result-main {
    display: flex;
    align-items: baseline;
    gap: 8px;
    min-width: 0;
  }

  .result-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-0);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }

  .result-domain {
    font-size: 12px;
    color: var(--text-3);
    flex-shrink: 0;
    white-space: nowrap;
  }

  .result-cat {
    font-size: 11px;
    color: var(--cat-text);
    background: var(--cat-soft);
    padding: 0 5px;
    border-radius: 3px;
    flex-shrink: 0;
    white-space: nowrap;
  }

  .result-tags {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .tag-chip {
    font-size: 11px;
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--accent-soft);
    color: var(--accent-text);
  }

  .tag-more {
    font-size: 11px;
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--bg-2);
    color: var(--text-3);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 28px 0 20px;
    color: var(--text-3);
    font-size: 13px;
    animation: fade-in 200ms ease;
  }

  .hint {
    text-align: center;
    padding: 20px 0;
    color: var(--text-3);
    font-size: 13px;
    animation: fade-in 200ms ease;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }
</style>
