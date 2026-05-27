<script>
  import { onMount } from "svelte";
  import { searchLinks, listLinks, openUrl, getSetting, setSetting, listCategories } from "../lib/api.js";
  import { waitForBackendReady } from "../lib/ready.js";
  import { formatAbsoluteTime, formatRelativeTime } from "../lib/utils/time.js";
  import { getDomain } from "../lib/utils/url.js";
  import { themeStore } from "../lib/stores/themeStore.svelte.js";
  import { emit, listen } from "@tauri-apps/api/event";
  import { getCurrentWindow, LogicalSize, LogicalPosition } from "@tauri-apps/api/window";

  const WIN_WIDTH = 560;
  const WIN_MIN_HEIGHT = 52;
  const WIN_MAX_HEIGHT = 420;
  // 默认列表与搜索结果均按 30 条/页加载，与主页 LinkList 行为一致
  const PAGE_SIZE = 30;

  let query = $state("");
  let results = $state([]);
  let selected_index = $state(-1);
  let searching = $state(false);
  let has_searched = $state(false);
  let search_timer = null;
  let input_el;
  let results_el;
  let categories = $state([]);
  let mouse_moved = $state(false);
  let hiding = false;
  let just_shown = false;
  let current_sort = $state("");
  // 是否在空查询时默认展示主页搜索结果（由设置项 spotlight-show-default 控制）
  let show_default_results = $state(false);
  // 标记当前 results 是"默认展示结果"而非"用户搜索结果"
  let is_showing_defaults = $state(false);
  // 分页状态：默认列表与搜索列表共用，按当前模式向后端要不同接口
  let current_page = $state(1);
  let total_count = $state(0);
  let has_more = $state(false);
  let loading_more = $state(false);

  function category_name(cid) {
    if (cid == null) return null;
    const cat = categories.find(c => c.id === cid);
    return cat?.name || null;
  }

  async function resize_window() {
    await new Promise(r => requestAnimationFrame(r));
    await new Promise(r => requestAnimationFrame(r));
    const searchEl = document.querySelector(".search-area");
    if (!searchEl) return;
    const search_h = searchEl.getBoundingClientRect().height;
    // 测量内部 wrapper 的真实内容高度而非 results-container 的 scrollHeight：
    // results-container 是 flex:1 + overflow:auto，scrollHeight 在多数浏览器下
    // 会 max(clientHeight, content-height)，导致内容很短时窗口仍按上一帧的高度
    // 锁定，无法收缩。
    const innerEl = document.querySelector(".results-inner");
    const containerEl = document.querySelector(".results-container");
    let content_h = 0;
    if (innerEl) {
      content_h = innerEl.scrollHeight;
    } else if (containerEl) {
      content_h = containerEl.scrollHeight;
    }
    // 加上 .results-container 自身的 padding（CSS: 0 16px 16px），
    // wrapper 的 scrollHeight 不含 container 的 padding-bottom
    if (innerEl && containerEl) {
      const cs = window.getComputedStyle(containerEl);
      content_h += parseFloat(cs.paddingTop || "0") + parseFloat(cs.paddingBottom || "0");
    }
    // +2 补偿 .spotlight 的 1px border 上下（border-box 下内容区 = 窗口高度 - 2px）
    const total_h = search_h + content_h + 2;
    const h = Math.min(total_h, WIN_MAX_HEIGHT);
    await getCurrentWindow().setSize(new LogicalSize(WIN_WIDTH, Math.ceil(h)));
  }

  // Spotlight 风格：保留 "www."，解析失败回退原 url
  const extract_domain = (url) => getDomain(url, { stripWww: false, fallback: "original" });

  // Spotlight 风格："3分钟前"（无空格），超 7 天回退到 "YYYY-MM-DD HH:mm"
  const format_last_opened = (ts) =>
    formatRelativeTime(ts, { spaceBeforeUnit: false, fallback: "absolute" });
  const format_absolute_time = formatAbsoluteTime;

  async function do_search(q) {
    if (!q.trim()) {
      // 空查询：若开启「显示默认结果」则加载主页同款列表第一页，否则清空
      if (show_default_results) {
        await load_default_results();
      } else {
        results = [];
        has_searched = false;
        is_showing_defaults = false;
        current_page = 1;
        total_count = 0;
        has_more = false;
        // 切回搜索区独立时滚动归零，避免上一次的 scrollTop 残留
        if (results_el) results_el.scrollTop = 0;
        resize_window();
      }
      return;
    }
    searching = true;
    try {
      const res = await searchLinks({
        query: q.trim(),
        page: 1,
        per_page: PAGE_SIZE,
        sort_by: current_sort || undefined,
      });
      results = res.items || [];
      total_count = res.total ?? results.length;
      current_page = res.page ?? 1;
      has_more = results.length < total_count;
      has_searched = true;
      is_showing_defaults = false;
    } catch {
      results = [];
      total_count = 0;
      current_page = 1;
      has_more = false;
      has_searched = true;
      is_showing_defaults = false;
    }
    searching = false;
    selected_index = -1;
    mouse_moved = false;
    // 新一轮搜索：滚动归零，再让窗口按新内容收缩/扩张
    if (results_el) results_el.scrollTop = 0;
    resize_window();
  }

  // 加载"默认展示结果":调用 listLinks 拿到主页同款列表第一页
  // 与主页 load_links 保持一致：无搜索、无分类/标签筛选，仅按 sort_by 排序
  async function load_default_results() {
    searching = true;
    try {
      const res = await listLinks({
        page: 1,
        per_page: PAGE_SIZE,
        sort_by: current_sort || undefined,
      });
      results = res.items || [];
      total_count = res.total ?? results.length;
      current_page = res.page ?? 1;
      has_more = results.length < total_count;
      // 不设 has_searched=true：避免空数据时显示"没有找到相关链接"
      has_searched = false;
      is_showing_defaults = results.length > 0;
    } catch {
      results = [];
      total_count = 0;
      current_page = 1;
      has_more = false;
      has_searched = false;
      is_showing_defaults = false;
    }
    searching = false;
    selected_index = -1;
    mouse_moved = false;
    if (results_el) results_el.scrollTop = 0;
    resize_window();
  }

  // 无限滚动：搜索 / 默认列表 下一页追加
  async function load_more() {
    if (loading_more || !has_more) return;
    const q = query.trim();
    // 搜索结果与默认列表分别走对应 API
    loading_more = true;
    try {
      const next_page = current_page + 1;
      const res = q
        ? await searchLinks({
            query: q,
            page: next_page,
            per_page: PAGE_SIZE,
            sort_by: current_sort || undefined,
          })
        : await listLinks({
            page: next_page,
            per_page: PAGE_SIZE,
            sort_by: current_sort || undefined,
          });
      const more_items = res.items || [];
      results = [...results, ...more_items];
      total_count = res.total ?? total_count;
      current_page = res.page ?? next_page;
      has_more = results.length < total_count;
    } catch {
      // 单次追加失败：保留已有数据，下次滚动还可重试
    }
    loading_more = false;
    // 追加新条目后内容增高，要求 resize 重新计算（在未到 max 高度时窗口同步增高）
    resize_window();
  }

  function on_results_scroll() {
    if (!results_el || loading_more || !has_more) return;
    if (results_el.scrollHeight - results_el.scrollTop - results_el.clientHeight < 200) {
      load_more();
    }
  }

  function on_input() {
    clearTimeout(search_timer);
    search_timer = setTimeout(() => do_search(query), 150);
  }

  function clear_input() {
    query = "";
    results = [];
    has_searched = false;
    is_showing_defaults = false;
    current_page = 1;
    total_count = 0;
    has_more = false;
    loading_more = false;
    selected_index = -1;
    searching = false;
    clearTimeout(search_timer);
    input_el?.focus();
    if (results_el) results_el.scrollTop = 0;
    // 清空后若开启默认展示，重新加载默认结果
    if (show_default_results) {
      load_default_results();
    } else {
      resize_window();
    }
  }

  async function hide_window() {
    if (hiding) return;
    hiding = true;
    query = "";
    has_searched = false;
    current_page = 1;
    total_count = 0;
    has_more = false;
    loading_more = false;
    selected_index = -1;
    searching = false;
    mouse_moved = false;
    clearTimeout(search_timer);
    if (show_default_results) {
      // 开启默认展示时保留 results 且不缩小窗口，
      // 下次 show 时窗口高度和内容都已就绪，零闪烁零抖动
      await getCurrentWindow().hide();
    } else {
      results = [];
      is_showing_defaults = false;
      // 等一帧让 DOM 更新，再缩小窗口
      await new Promise(r => requestAnimationFrame(r));
      if (!hiding) return;
      const searchEl = document.querySelector(".search-area");
      const h = searchEl ? Math.ceil(searchEl.getBoundingClientRect().height + 2) : WIN_MIN_HEIGHT;
      await getCurrentWindow().setSize(new LogicalSize(WIN_WIDTH, h));
      if (!hiding) return;
      await getCurrentWindow().hide();
    }
    hiding = false;
  }

  function scroll_to_selected() {
    requestAnimationFrame(() => {
      const el = document.querySelector(`.result-item[data-index="${selected_index}"]`);
      el?.scrollIntoView({ block: "nearest" });
    });
  }

  const SORT_CYCLE = ["", "click_count", "last_opened_at"];
  const SORT_LABELS = { "": "最近更新", "click_count": "最多访问", "last_opened_at": "最近打开" };

  function handle_keydown(e) {
    if (e.key === "Escape") {
      e.preventDefault();
      hide_window();
      return;
    }
    if (e.key === "Tab") {
      e.preventDefault();
      const idx = SORT_CYCLE.indexOf(current_sort);
      current_sort = SORT_CYCLE[(idx + 1) % SORT_CYCLE.length];
      setSetting("spotlight-sort-by", current_sort);
      if (query.trim()) {
        do_search(query);
      } else if (show_default_results) {
        // 空查询且开启默认展示：用新排序刷新默认结果
        load_default_results();
      }
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

    // 主题加载、系统主题监听、跨窗口同步全部由 themeStore 接管
    await themeStore.init();

    const savedSort = await getSetting("spotlight-sort-by");
    if (savedSort) current_sort = savedSort;

    // 读取"唤起后默认展示主页前 N 条"开关；默认关闭以保持原有行为
    try {
      show_default_results = (await getSetting("spotlight-show-default")) === "true";
    } catch {
      show_default_results = false;
    }

    listCategories().then(c => categories = c);

    // 预加载默认结果：窗口 hidden 时就把数据准备好，
    // 这样 Rust 端 show() 后内容立即可见，无闪烁
    if (show_default_results) {
      await load_default_results();
    }

    // 初始 resize：窗口 hidden 时默认高度仅 52px，DOM 被压缩导致测量不准。
    // 先临时放大到 WIN_MAX_HEIGHT 让 DOM 自然布局，测量后再设为正确高度。
    if (show_default_results) {
      await getCurrentWindow().setSize(new LogicalSize(WIN_WIDTH, WIN_MAX_HEIGHT));
    }
    await resize_window();

    const unlistenShown = await listen("spotlight-shown", async () => {
      hiding = false;
      just_shown = true;
      input_el?.focus();
      // 重读配置确保设置变更后再次唤起即时生效
      try {
        const raw = await getSetting("spotlight-show-default");
        show_default_results = raw === "true";
      } catch {
        // 保持现有值
      }
      // 窗口可见后 resize 确保高度正确（onMount 中 hidden 状态下测量可能不准）
      await resize_window();
      // 开启默认展示时，异步刷新数据
      if (show_default_results && !query.trim()) {
        load_default_results();
      }
      // 延迟解除保护，避免快捷键释放导致焦点变化误触发 hide
      setTimeout(() => { just_shown = false; }, 400);
    });

    const unlistenHide = await listen("spotlight-hide", () => {
      // Rust 端已 hide 窗口，只做状态清理
      hiding = true;
      query = "";
      has_searched = false;
      current_page = 1;
      total_count = 0;
      has_more = false;
      loading_more = false;
      selected_index = -1;
      searching = false;
      mouse_moved = false;
      clearTimeout(search_timer);
      if (!show_default_results) {
        results = [];
        is_showing_defaults = false;
      }
      hiding = false;
    });

    const unlistenFocus = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (focused) {
        if (!hiding) input_el?.focus();
      } else if (!hiding && !just_shown) {
        setTimeout(() => {
          if (!just_shown && !hiding) hide_window();
        }, 150);
      }
    });

    window.addEventListener("keydown", handle_keydown);

    return () => {
      window.removeEventListener("keydown", handle_keydown);
      unlistenShown();
      unlistenHide();
      unlistenFocus();
      clearTimeout(search_timer);
    };
  });
</script>

<div class="spotlight">
  <div class="search-area">
    <div class="input-wrap">
      <div class="search-field">
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
      <div class="sort-hint">
        <span class="sort-hint-label">{SORT_LABELS[current_sort]}</span>
        <span class="sort-hint-key">Tab ↹</span>
      </div>
    </div>
  </div>

  {#if has_searched || results.length > 0}
  <div
    bind:this={results_el}
    class="results-container"
    class:scrollable={results.length > 0}
    onmousemove={() => mouse_moved = true}
    onscroll={on_results_scroll}
  >
  <div class="results-inner">
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
          <div class="result-bottom">
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
              <div class="result-stats">
                {#if link.click_count > 0}
                  <span class="result-stat">
                    <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>
                    {link.click_count}
                  </span>
                {/if}
                {#if link.last_opened_at}
                  <span class="result-stat" title={`上次打开：${format_absolute_time(link.last_opened_at)}`}>
                    <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                    {format_last_opened(link.last_opened_at)}
                  </span>
                {/if}
              </div>
            </div>
        </div>
      {/each}
      {#if loading_more}
        <div class="load-more-state">
          <span class="spinner"></span>
        </div>
      {/if}
    {:else if !has_searched}
      <div class="hint">输入关键词搜索已保存的链接</div>
    {/if}
  </div>
  </div>
  {/if}
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

  .search-area {
    padding: 6px 10px;
    flex-shrink: 0;
  }

  .input-wrap {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .sort-hint {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 8px;
    height: 34px;
    border: 1px solid var(--border-1);
    border-radius: var(--radius-md);
    background: var(--bg-1);
    color: var(--text-2);
    font-size: 12px;
    flex-shrink: 0;
    user-select: none;
    transition: all var(--transition);
  }

  .sort-hint-label {
    font-weight: 500;
  }

  .sort-hint-key {
    font-size: 10px;
    color: var(--text-3);
    background: var(--bg-2);
    padding: 2px 4px;
    border-radius: 3px;
    border: 1px solid var(--border-0);
  }

  .search-field {
    position: relative;
    flex: 1;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 10px;
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
    padding: 0 36px 0 36px;
    height: 34px;
    border-radius: 10px;
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
    padding: 0 16px 16px;
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

  .result-bottom {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
  }

  .result-tags {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
    min-width: 0;
    overflow: hidden;
  }

  .result-stats {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
    margin-left: auto;
  }

  .result-stat {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    font-size: 10px;
    color: var(--text-3);
    white-space: nowrap;
  }

  .result-tags .result-stat:first-of-type {
    margin-left: auto;
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

  .result-stat {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    font-size: 10px;
    color: var(--text-3);
    margin-left: auto;
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

  .load-more-state {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 12px 0 8px;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }
</style>
