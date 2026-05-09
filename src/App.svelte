<script>
  import { onMount, tick } from "svelte";
  import { marked } from "marked";
  import { linksStore, categoriesStore, tagsStore, settingsStore } from "./lib/stores/index.js";
  import * as api from "./lib/api.js";
  import { waitForBackendReady } from "./lib/ready.js";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { emit, listen } from "@tauri-apps/api/event";
  import SearchBar from "./lib/components/SearchBar.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import LinkList from "./lib/components/LinkList.svelte";
  import LinkForm from "./lib/components/LinkForm.svelte";
  import ExportDialog from "./lib/components/ExportDialog.svelte";
  import SettingsDialog from "./lib/components/SettingsDialog.svelte";
  import UpdateDialog from "./lib/components/UpdateDialog.svelte";

  let is_macos = $state(false);

  // Theme mode state: "light" | "dark" | "system". Default to "system".
  let theme_mode = $state("system");
  // Actual applied dark mode state derived from theme_mode (system or explicit)
  let dark_mode = $state(false);
  let system_unlisten; // cleanup for system theme listener

  let links = $derived($linksStore);
  let categories = $derived($categoriesStore);
  let tags = $derived($tagsStore);

  function find_category_by_id(id, nodes = categories) {
    for (const node of nodes) {
      if (node.id === id) return node;
      if (node.children?.length > 0) {
        const found = find_category_by_id(id, node.children);
        if (found) return found;
      }
    }
    return null;
  }

  let selected_category = $state(null);
  let selected_tag = $state(null);
  let search_query = $state("");
  let sort_by = $state(null);
  let show_add_form = $state(false);

  function reset_filters() {
    search_query = "";
    selected_category = null;
    selected_tag = null;
    selected_link_index = -1;
  }
  let edit_link = $state(null);
  let show_export = $state(false);
  let show_settings = $state(false);
  let show_close_dialog = $state(false);
  let search_bar;
  let selected_link_index = $state(-1);

  // 更新相关状态
  let update_available = $state(false);
  let update_info = $state(null);
  let release_notes = $state("");
  let show_update_dialog = $state(false);
  let show_release_notes = $state(false);
  let last_update_notes = $state("");
  let current_version = $state("");

  onMount(async () => {
    const mainWindow = getCurrentWindow();
    console.log("[startup] App onMount start");
    await waitForBackendReady();
    console.log("[startup] backend ready");
    const isDeepLinkStartup = await api.checkStartupDeepLink();
    if (!isDeepLinkStartup) {
      try { await mainWindow.show(); } catch (e) {}
    }
    // Load theme setting with backward-compatibility
    let savedTheme = await api.getSetting("theme-mode");
    if (!savedTheme) {
      // Backwards compatibility: migrate legacy dark-mode if present
      const legacyDark = await api.getSetting("dark-mode");
      if (legacyDark === "true") savedTheme = "dark";
      else if (legacyDark === "false") savedTheme = "light";
      else savedTheme = "system";
    }
    theme_mode = savedTheme || "system";
    const savedSort = await api.getSetting("sort-by");
    if (savedSort) sort_by = savedSort;
    try {
      const v = (await api.getSetting("check-link-reachability")) !== "false";
      settingsStore.update(s => ({ ...s, check_link_reachability: v }));
    } catch {}
    // Initialize dark_mode based on current theme_mode
    apply_theme();
    // Listen for OS theme changes if in system mode
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    function on_system_theme_change(e) {
      if (theme_mode === "system") {
        apply_theme();
      }
    }
    if (mq && typeof mq.addEventListener === "function") {
      mq.addEventListener("change", on_system_theme_change);
    } else if (mq && typeof mq.addListener === "function") {
      mq.addListener(on_system_theme_change);
    }
    system_unlisten = () => {
      if (mq) {
        if (typeof mq.removeEventListener === "function") mq.removeEventListener("change", on_system_theme_change);
        else if (typeof mq.removeListener === "function") mq.removeListener(on_system_theme_change);
      }
    };
    is_macos = /mac/i.test(navigator.userAgentData?.platform ?? navigator.platform);
    console.log("[startup] loading data...");
    await load_data();
    console.log("[startup] data loaded");

    // 检查应用更新（受配置控制）
    const auto_check_update = await api.getSetting("auto-check-update");
    if (auto_check_update !== "false") {
      check_for_update();
    }

    // 首次启动更新后显示更新说明（优先从 GitHub API 获取，缓存兜底）
    try {
      const last_version = await api.getSetting("last-known-version");
      const { getVersion } = await import("@tauri-apps/api/app");
      current_version = await getVersion();
      if (last_version && last_version !== current_version) {
        let notes = "";
        try {
          notes = await api.fetchReleaseNotes(`v${current_version}`) || "";
        } catch (e) {
          console.warn("[update] failed to fetch release notes for post-update:", e);
        }
        if (!notes) {
          notes = await api.getSetting("last-update-notes") || "";
        }
        if (notes) {
          last_update_notes = notes;
          show_release_notes = true;
        }
      }
      await api.setSetting("last-known-version", current_version);
    } catch (e) {
      console.warn("[update] version check failed:", e);
    }

    const { LogicalSize, PhysicalPosition } = await import("@tauri-apps/api/window");
    const unlisten = await listen("main-shown", () => {
      search_bar?.focus();
    });
    const unlistenHidden = await listen("main-hidden", () => {
      reset_filters();
    });

    const savedSize = await api.getSetting("window-size");
    if (savedSize) {
      try {
        const { width, height } = JSON.parse(savedSize);
        await mainWindow.setSize(new LogicalSize(width, height));
      } catch (e) {}
    }

    const savedPos = await api.getSetting("window-position");
    if (savedPos) {
      try {
        const { x, y } = JSON.parse(savedPos);
        await mainWindow.setPosition(new PhysicalPosition(x, y));
      } catch (e) {}
    }

    const splash = document.getElementById("splash");
    if (splash) {
      splash.classList.add("fade-out");
      setTimeout(() => splash.remove(), 300);
    }

    let resize_restore_done = false;
    const resize_restore_timeout = setTimeout(() => { resize_restore_done = true; }, 2000);

    let resize_timer;
    async function save_window_state() {
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
        const pos = await win.outerPosition();
        await api.setSetting("window-position", JSON.stringify({
          x: pos.x,
          y: pos.y
        }));
      } catch (e) {}
    }
    function on_resize() {
      clearTimeout(resize_timer);
      if (!resize_restore_done) return;
      resize_timer = setTimeout(save_window_state, 500);
    }
    window.addEventListener("resize", on_resize);
    let unlistenMoved;
    try {
      unlistenMoved = await mainWindow.onMoved(() => {
        clearTimeout(resize_timer);
        if (!resize_restore_done) return;
        resize_timer = setTimeout(save_window_state, 500);
      });
    } catch (e) {
      console.warn("[window] onMoved not available:", e);
    }

    await mainWindow.onCloseRequested(async (event) => {
      event.preventDefault();
      const behavior = (await api.getSetting("close-behavior")) || "ask";
      if (behavior === "exit") {
        await api.exitApp();
      } else if (behavior === "tray") {
        reset_filters();
        await mainWindow.hide();
      } else {
        show_close_dialog = true;
      }
    });

    // 监听 quick-add 窗口的保存事件
    let unlistenLinksChanged;
    let unlistenSpotlightLocate;
    try {
      const { listen } = await import("@tauri-apps/api/event");
      unlistenLinksChanged = await listen("links-changed", () => {
        refresh_current_view();
        categoriesStore.load();
        tagsStore.load();
      });
      unlistenSpotlightLocate = await listen("spotlight-locate", async (e) => {
        const { link_id } = e.payload;
        // 显示主窗口并聚焦
        try { await mainWindow.show(); } catch {}
        try { await mainWindow.setFocus(); } catch {}
        // 清除筛选
        selected_category = null;
        selected_tag = null;
        search_query = "";
        selected_link_index = -1;
        // 加载所有链接（最多100条）
        await linksStore.load({ per_page: 100 });
        // 等待 DOM 更新
        await tick();
        await new Promise(r => requestAnimationFrame(r));
        // 查找链接索引
        const index = filtered_links.findIndex(l => l.id === link_id);
        if (index >= 0) {
          selected_link_index = index;
          scroll_selected_into_view();
          edit_link = filtered_links[index];
        } else {
          console.warn("[spotlight-locate] link not found:", link_id);
        }
      });
    } catch (e) {}

    return () => {
      clearTimeout(resize_restore_timeout);
      clearTimeout(resize_timer);
      window.removeEventListener("resize", on_resize);
      if (unlisten) unlisten();
      if (unlistenHidden) unlistenHidden();
      if (unlistenLinksChanged) unlistenLinksChanged();
      if (unlistenSpotlightLocate) unlistenSpotlightLocate();
      if (unlistenMoved) unlistenMoved();
      if (system_unlisten) system_unlisten();
    };
  });

  async function load_data() {
    try {
      await Promise.all([
        categoriesStore.load(),
        tagsStore.load(),
        load_links(),
        load_stats(),
      ]);
      console.log("[startup] load_data done");
    } catch (e) {
      console.error("[startup] load_data failed:", e);
    }
  }

  async function load_links() {
    if (selected_category === 'stats') return;
    const params = {};
    if (selected_tag === "__untagged__") {
      params.untagged_only = true;
    } else if (selected_tag) {
      params.tag = selected_tag;
    } else if (selected_category === "favorite") {
      params.favorite_only = true;
    } else if (selected_category === "uncategorized") {
      params.uncategorized_only = true;
    } else if (selected_category != null) {
      params.category_id = selected_category;
    }
    if (sort_by) {
      params.sort_by = sort_by;
    }
    await linksStore.load(params);
  }

  function get_scroll_el() {
    return document.querySelector('.link-list');
  }

async function with_scroll_preserve(fn) {
    const el = get_scroll_el();
    if (!el) {
        await fn();
        return;
    }

    // Identify the first visible LinkCard in the viewport to anchor scrolling
    const containerRect = el.getBoundingClientRect();
    let anchorId = null;
    let anchorOffsetTop = 0;

    const cards = el.querySelectorAll('.link-card[data-link-id]');
    let firstVisible = null;
    for (let i = 0; i < cards.length; i++) {
        const c = cards[i];
        const r = c.getBoundingClientRect();
        if (r.bottom > containerRect.top && r.top < containerRect.bottom) {
            firstVisible = c;
            break;
        }
    }
    if (firstVisible) {
        anchorId = firstVisible.getAttribute('data-link-id');
        anchorOffsetTop = firstVisible.offsetTop;
    }

    const scrollTopBefore = el.scrollTop;
    const scrollHeightBefore = el.scrollHeight;

    await fn();
    await tick();
    await new Promise(r => requestAnimationFrame(r));

    const el_after = get_scroll_el();
    if (el_after && anchorId) {
        const anchorEl2 = el_after.querySelector(`.link-card[data-link-id="${anchorId}"]`);
        if (anchorEl2) {
            const anchorTop2 = anchorEl2.offsetTop;
            const newScrollTop = anchorTop2 - anchorOffsetTop;
            const maxScroll = Math.max(0, (el_after.scrollHeight - el_after.clientHeight));
            el_after.scrollTop = Math.max(0, Math.min(newScrollTop, maxScroll));
            return;
        } else {
            // Try to scroll to the next card if the anchored one disappeared (e.g., deletion)
            const nextCard = el_after.querySelector('.link-card[data-link-id]');
            if (nextCard) {
                const targetTop = nextCard.offsetTop;
                const maxScroll = Math.max(0, (el_after.scrollHeight - el_after.clientHeight));
                el_after.scrollTop = Math.max(0, Math.min(targetTop, maxScroll));
                return;
            }
        }
    }

    // Fallback: preserve proportional scroll position if no anchor could be restored
    const newTotal = el_after?.scrollHeight ?? 0;
    const clientHeight = el_after?.clientHeight ?? 0;
    if (scrollHeightBefore > 0 && newTotal > 0 && clientHeight > 0) {
        const ratio = scrollTopBefore / scrollHeightBefore;
        const newScrollTop = Math.round(newTotal * ratio);
        const maxScroll = Math.max(0, newTotal - clientHeight);
        el_after.scrollTop = Math.max(0, Math.min(newScrollTop, maxScroll));
    }
}

  // 仅用于视图参数变化（切换分组/标签）或跨窗口同步等需要全量刷新的场景，
  // 普通链接增删改用本地 store mutation 即可，避免 load() 破坏无限滚动。
  async function refresh_current_view() {
    await with_scroll_preserve(async () => {
      if (search_query.trim()) {
        await linksStore.search({ query: search_query, per_page: 30, ...build_filter_params() });
      } else {
        await load_links();
      }
      await categoriesStore.load();
    });
    load_stats();
  }

  function on_category_select(id) {
    selected_category = id;
    selected_tag = null;
    search_query = "";
    selected_link_index = -1;
    if (id === 'stats') {
      load_stats();
    } else {
      load_links();
    }
  }

  function on_tag_select(tag) {
    if (selected_tag === tag) {
      selected_tag = null;
    } else {
      selected_tag = tag;
      selected_category = null;
    }
    search_query = "";
    selected_link_index = -1;
    load_links();
  }

  async function on_search(query) {
    selected_link_index = -1;
    if (query.trim()) {
      await linksStore.search({ query, per_page: 30, ...build_filter_params() });
    } else {
      await load_links();
    }
  }

  // 不做列表全量刷新，本地 store mutation 已正确更新数据，
  // 避免 load() 重置为第 1 页破坏无限滚动状态。
  async function on_save_link(data) {
    show_add_form = false;
    edit_link = null;
    if (data.id) {
      await linksStore.update(data);
    } else {
      await linksStore.create(data);
    }
    await categoriesStore.load();
    await tagsStore.load();
    load_stats();
  }

async function on_toggle_favorite(link) {
    await linksStore.update({ id: link.id, is_favorite: !link.is_favorite });
    // Do not refresh the view; local update suffices for bookmark toggle
}

  async function on_delete_link(link) {
    selected_link_index = -1;
    await linksStore.remove(link.id);
    await categoriesStore.load();
    load_stats();
  }

  async function on_remove_category(link) {
    await linksStore.update({ id: link.id, category_id: -1 });
    await categoriesStore.load();
  }

  async function on_remove_tag(link, tag) {
    const remaining = link.tags.filter(t => t !== tag);
    await linksStore.update({ id: link.id, tags: remaining });
    await tagsStore.load();
  }

  async function on_remove_notes(link) {
    await linksStore.update({ id: link.id, notes: "" });
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

  async function on_rename_category(payload) {
    await categoriesStore.update(payload);
  }

  async function on_delete_tag(id) {
    await tagsStore.remove(id);
    if (selected_tag) {
      selected_tag = null;
    }
    await refresh_current_view();
  }

  async function on_rename_tag(payload) {
    await tagsStore.update(payload);
  }

  async function on_create_tag(name) {
    await tagsStore.create(name);
  }

  let importing = $state(false);
  let sidebar_stats = $state(null);

  async function load_stats() {
    try {
      sidebar_stats = await api.linksStats();
    } catch (e) {
      console.warn("[stats] failed:", e);
    }
  }

  async function on_import_bookmarks() {
    importing = true;
    try {
      const [linkCount, catCount] = await api.importBookmarks();
      if (linkCount > 0 || catCount > 0) {
        await load_data();
      }
    } finally {
      importing = false;
    }
  }

  async function close_to_tray() {
    show_close_dialog = false;
    reset_filters();
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().hide();
  }

  async function close_exit() {
    show_close_dialog = false;
    await api.exitApp();
  }

  async function toggle_dark() {
    theme_mode = dark_mode ? "light" : "dark";
    apply_theme();
    await api.setSetting("theme-mode", theme_mode);
    emit("theme-changed", theme_mode);
  }

  function apply_theme() {
    if (theme_mode === "system") {
      dark_mode = window.matchMedia("(prefers-color-scheme: dark)").matches;
    } else {
      dark_mode = (theme_mode === "dark");
    }
    const root = document.documentElement;
    root.classList.add("no-transition");
    root.classList.toggle("dark", dark_mode);
    root.offsetHeight;
    requestAnimationFrame(() => root.classList.remove("no-transition"));
  }

  async function on_theme_change(mode) {
    theme_mode = mode;
    apply_theme();
    emit("theme-changed", mode);
  }

  let filtered_links = $derived(links.items);
  let has_more = $derived(links.has_more);
  let current_page = $derived(links.page);
  let total_count = $derived(links.total);
  let show_stats_view = $derived(selected_category === 'stats');
  let current_title = $derived(
    selected_category === 'stats' ? '统计' :
    search_query.trim() ? `搜索: ${search_query}` :
    selected_tag === "__untagged__" ? "无标签" :
    selected_tag ? `标签: ${selected_tag}` :
    selected_category === "favorite" ? "特别关注" :
    selected_category === "uncategorized" ? "未分组" :
    selected_category != null ? find_category_by_id(selected_category)?.name ?? "链接" :
    "全部链接"
  );

  function build_filter_params() {
    const params = {};
    if (selected_tag === "__untagged__") {
      params.untagged_only = true;
    } else if (selected_tag) {
      params.tag = selected_tag;
    } else if (selected_category === "favorite") {
      params.favorite_only = true;
    } else if (selected_category === "uncategorized") {
      params.uncategorized_only = true;
    } else if (selected_category != null) {
      params.category_id = selected_category;
    }
    if (sort_by) {
      params.sort_by = sort_by;
    }
    return params;
  }

  // 搜索框筛选条件芯片
  let filter_chip = $derived.by(() => {
    if (selected_tag === "__untagged__") {
      return { label: "无标签", type: "tag" };
    } else if (selected_tag) {
      return { label: selected_tag, type: "tag" };
    } else if (selected_category === "favorite") {
      return { label: "特别关注", type: "favorite" };
    } else if (selected_category === "uncategorized") {
      return { label: "未分组", type: "category" };
    } else if (selected_category != null) {
      const cat = find_category_by_id(selected_category);
      return { label: cat?.name ?? "分组", type: "category" };
    }
    return null;
  });

  function on_remove_filter() {
    selected_category = null;
    selected_tag = null;
    selected_link_index = -1;
    if (search_query.trim()) {
      linksStore.search({ query: search_query, per_page: 30 });
    } else {
      load_links();
    }
  }

  function on_sort_change(value) {
    sort_by = value || null;
    api.setSetting("sort-by", sort_by || "");
    selected_link_index = -1;
    if (search_query.trim()) {
      linksStore.search({ query: search_query, per_page: 30, ...build_filter_params() });
    } else {
      load_links();
    }
  }

  const SORT_CYCLE = [null, "click_count", "last_opened_at"];

  function on_tab_sort() {
    const idx = SORT_CYCLE.indexOf(sort_by);
    const next = SORT_CYCLE[(idx + 1) % SORT_CYCLE.length];
    on_sort_change(next || "");
  }

  async function fetch_github_notes(version) {
    try {
      const notes = await api.fetchReleaseNotes(`v${version}`);
      if (notes) release_notes = notes;
    } catch (e) {
      console.warn("[update] failed to fetch release notes from GitHub:", e);
    }
  }

  async function check_for_update() {
    try {
      const update = await api.checkUpdate();
      if (update) {
        update_available = true;
        update_info = update;
        fetch_github_notes(update.version);
      }
    } catch (e) {
      // 开发阶段或无发布版本时 endpoint 返回 404 属于正常情况，不警告
      const msg = e?.message || String(e);
      if (!msg.includes("404") && !msg.includes("Not Found")) {
        console.warn("[update] check failed:", e);
      }
    }
  }

  function on_update_click() {
    show_update_dialog = true;
  }

  function on_update_close() {
    show_update_dialog = false;
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

  async function on_global_keydown(e) {
    // 键盘导航：上下箭头、Enter、Space
    const nav_keys = ["ArrowUp", "ArrowDown", "Enter", " "];
    if (nav_keys.includes(e.key)) {
      // 仅在无模态框/表单打开时处理
      const any_modal_open = show_settings || show_export || show_add_form || edit_link || show_close_dialog || show_update_dialog || show_release_notes;
      if (!any_modal_open) {
        const active_tag = document.activeElement?.tagName;
        const in_input = active_tag === "INPUT" || active_tag === "TEXTAREA" || active_tag === "SELECT";
        const count = filtered_links.length;
        // 箭头键在搜索框聚焦时也生效（单行输入框中箭头键无其他用途）
        if (e.key === "ArrowDown") {
          e.preventDefault();
          if (count === 0) return;
          document.activeElement?.blur();
          selected_link_index = Math.min(selected_link_index + 1, count - 1);
          scroll_selected_into_view();
          return;
        }
        if (e.key === "ArrowUp") {
          e.preventDefault();
          if (count === 0) return;
          document.activeElement?.blur();
          selected_link_index = Math.max(selected_link_index - 1, 0);
          scroll_selected_into_view();
          return;
        }
        // Enter 和 Space 仅在非输入框聚焦时生效
        if (!in_input) {
          if (e.key === "Enter" && selected_link_index >= 0 && selected_link_index < count) {
            e.preventDefault();
            const link = filtered_links[selected_link_index];
            api.openUrl(link.url);
            selected_link_index = -1;
            if ((await api.getSetting("auto-minimize-on-open")) === "true") {
              reset_filters();
              const { getCurrentWindow } = await import("@tauri-apps/api/window");
              await getCurrentWindow().hide();
            }
            return;
          }
          if (e.key === " " && selected_link_index >= 0 && selected_link_index < count) {
            e.preventDefault();
            edit_link = filtered_links[selected_link_index];
            selected_link_index = -1;
            return;
          }
        }
      }
    }

    if (e.key !== "Escape") return;
    if (show_settings) { show_settings = false; return; }
    if (show_export) { show_export = false; return; }
    if (show_add_form) { show_add_form = false; categoriesStore.load(); tagsStore.load(); return; }
    if (edit_link) { edit_link = null; categoriesStore.load(); tagsStore.load(); return; }
    if (show_close_dialog) { show_close_dialog = false; return; }
    if (show_update_dialog) { show_update_dialog = false; return; }
    if (show_release_notes) { show_release_notes = false; return; }
  }

  function scroll_selected_into_view() {
    // 使用 requestAnimationFrame 确保 DOM 已更新后再滚动
    requestAnimationFrame(() => {
      const list_el = document.querySelector('.link-list');
      if (!list_el) return;
      const cards = list_el.querySelectorAll('.link-card[data-link-id]');
      if (selected_link_index >= 0 && selected_link_index < cards.length) {
        cards[selected_link_index].scrollIntoView({ block: 'nearest', behavior: 'smooth' });
      }
    });
  }
</script>

<svelte:window onkeydown={on_global_keydown} />

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
      onrename_cat={on_rename_category}
      ontag_delete={on_delete_tag}
      onrename_tag={on_rename_tag}
      oncreate_tag={on_create_tag}
      dark={dark_mode}
      ontoggle_dark={toggle_dark}
      onexport={() => show_export = true}
      onimport={on_import_bookmarks}
      onsettings={() => show_settings = true}
      {importing}
      has_update={update_available}
      onupdate={on_update_click}
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
            {#if !show_stats_view}
              <span class="header-count">{total_count} 条</span>
            {/if}
          </div>
        {/if}
        <div class="header-right">
          {#if !show_stats_view}
            <select class="sort-select" onchange={(e) => on_sort_change(e.target.value)}>
              <option value="">最近更新</option>
              <option value="click_count" selected={sort_by === "click_count"}>最多访问</option>
              <option value="last_opened_at" selected={sort_by === "last_opened_at"}>最近打开</option>
            </select>
          {/if}
          <SearchBar bind:this={search_bar} bind:query={search_query} {filter_chip} onremovefilter={on_remove_filter} onsearch={on_search} ontab={on_tab_sort} />
        </div>
      </header>

      {#if show_stats_view}
        <div class="stats-panel">
          {#if sidebar_stats}
            <div class="stats-overview">
              <div class="stat-card">
                <span class="stat-value">{sidebar_stats.total}</span>
                <span class="stat-label">收藏总数</span>
              </div>
              <div class="stat-card">
                <span class="stat-value">+{sidebar_stats.this_week}</span>
                <span class="stat-label">本周新增</span>
              </div>
            </div>
            {#if sidebar_stats.top.length > 0}
              <div class="stats-top-section">
                <h3 class="stats-section-title">最常访问</h3>
                <div class="stats-top-list">
                  {#each sidebar_stats.top as link, i}
                    <div class="stats-top-row">
                      <span class="stats-rank">{i + 1}</span>
                      <div class="stats-top-info">
                        <span class="stats-top-title">{link.title}</span>
                        <span class="stats-top-url">{link.url}</span>
                      </div>
                      <span class="stats-top-count">
                        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>
                        {link.click_count} 次
                      </span>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}
          {:else}
            <div class="stats-loading">加载中...</div>
          {/if}
        </div>
      {:else}
        <LinkList
          links={filtered_links}
          {categories}
          loading={links.loading}
          highlight={search_query}
          has_more={has_more}
          selected_index={selected_link_index}
          onloadmore={load_more}
          onedit={(link) => edit_link = link}
          ondelete={on_delete_link}
          ontoggle_favorite={on_toggle_favorite}
          onremovecategory={on_remove_category}
          onremovetag={on_remove_tag}
          onremovenotes={on_remove_notes}
        />
      {/if}

      {#if !show_stats_view}
        <button class="fab" onclick={() => show_add_form = true} title="添加链接">
          <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <line x1="10" y1="4" x2="10" y2="16"/>
            <line x1="4" y1="10" x2="16" y2="10"/>
          </svg>
        </button>
      {/if}
    </main>
  </div>

  {#if show_add_form}
    <LinkForm categories={categories} onsave={on_save_link} oncancel={() => { show_add_form = false; categoriesStore.load(); tagsStore.load(); }} />
  {/if}

  {#if edit_link}
    <LinkForm link={edit_link} categories={categories} onsave={on_save_link} oncancel={() => { edit_link = null; categoriesStore.load(); tagsStore.load(); }} />
  {/if}

  {#if show_export}
    <ExportDialog onclose={() => show_export = false} />
  {/if}

  {#if show_settings}
    <SettingsDialog onclose={async () => { show_settings = false; const v = (await api.getSetting("check-link-reachability")) !== "false"; settingsStore.update(s => ({ ...s, check_link_reachability: v })); linksStore.load({ page: 1 }); }} onthemechange={on_theme_change} oncheckupdate={async (info) => { update_available = true; update_info = info; await fetch_github_notes(info.version); show_update_dialog = true; }} />
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

  {#if show_update_dialog && update_info}
    <UpdateDialog update_info={update_info} {release_notes} onclose={on_update_close} />
  {/if}

  {#if show_release_notes}
    <div class="modal-overlay" onclick={() => show_release_notes = false}>
      <div class="modal" onclick={(e) => e.stopPropagation()}>
        <div class="modal-header">
          <h3 class="modal-title">当前版本：v{current_version}</h3>
          <button class="modal-close" onclick={() => show_release_notes = false}>
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
              <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
            </svg>
          </button>
        </div>
        <div class="modal-body">
          <p class="release-notes-intro">版本更新内容：</p>
          <div class="release-notes-content markdown-body">
            {#if last_update_notes}
              {@html marked(last_update_notes)}
            {:else}
              暂无更新说明
            {/if}
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn btn-primary" onclick={() => show_release_notes = false}>知道了</button>
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
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .sort-select {
    padding: 5px 8px;
    border: 1px solid var(--border-1);
    border-radius: var(--radius-md);
    background: var(--bg-1);
    color: var(--text-2);
    font-size: 12px;
    outline: none;
    cursor: pointer;
    transition: all var(--transition);
    appearance: none;
    -webkit-appearance: none;
    padding-right: 22px;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6' viewBox='0 0 10 6'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%23999' stroke-width='1.5' fill='none' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 7px center;
  }

  .sort-select:hover {
    border-color: var(--border-2);
  }

  .sort-select:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-soft);
  }

  .fab {
    position: absolute;
    bottom: 24px;
    right: 24px;
    width: 44px;
    height: 44px;
    border-radius: 50%;
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

  .release-notes-intro {
    font-size: 13px;
    color: var(--text-2);
    margin-bottom: 12px;
  }

  .release-notes-content {
    background: var(--bg-1);
    border-radius: var(--radius-md);
    padding: 16px;
    max-height: 300px;
    overflow-y: auto;
  }

  .modal {
    max-width: 420px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
  }

  .modal-body {
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 20px 20px;
    flex-shrink: 0;
  }

  .stats-panel {
    flex: 1;
    padding: 24px;
    overflow-y: auto;
  }

  .stats-overview {
    display: flex;
    gap: 16px;
    margin-bottom: 28px;
  }

  .stat-card {
    flex: 1;
    background: var(--bg-1);
    border: 1px solid var(--border-0);
    border-radius: var(--radius-lg);
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .stat-value {
    font-size: 28px;
    font-weight: 700;
    color: var(--text-0);
    letter-spacing: -0.5px;
  }

  .stat-label {
    font-size: 12px;
    color: var(--text-3);
  }

  .stats-section-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-1);
    margin-bottom: 12px;
  }

  .stats-top-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stats-top-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: var(--bg-1);
    border: 1px solid var(--border-0);
    border-radius: var(--radius-md);
    transition: background var(--transition);
  }

  .stats-top-row:hover {
    background: var(--bg-hover);
  }

  .stats-rank {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    background: var(--accent-soft);
    color: var(--accent);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    font-weight: 700;
    flex-shrink: 0;
  }

  .stats-top-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stats-top-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-0);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .stats-top-url {
    font-size: 11px;
    color: var(--text-3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .stats-top-count {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-2);
    font-weight: 500;
    flex-shrink: 0;
  }

  .stats-loading {
    color: var(--text-3);
    font-size: 13px;
    padding: 40px 0;
    text-align: center;
  }
</style>
