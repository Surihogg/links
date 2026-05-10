<script>
  import { onMount, tick } from "svelte";
  import { linksStore, categoriesStore, tagsStore, settingsStore } from "./lib/stores/index.js";
  import { themeStore } from "./lib/stores/themeStore.svelte.js";
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
  import SortSelect from "./lib/components/SortSelect.svelte";
  import StatsPanel from "./lib/components/StatsPanel.svelte";
  import CloseDialog from "./lib/components/CloseDialog.svelte";
  import ReleaseNotesDialog from "./lib/components/ReleaseNotesDialog.svelte";
  import { findCategoryById } from "./lib/utils/categoryTree.js";

  let is_macos = $state(false);

  // 主题模式与暗色判定全部委托给 themeStore（跨窗口同步、系统跟随、持久化）
  // 模板里仍以 dark_mode / theme_mode 名字读取，保持原有 UI 代码改动最小
  let theme_mode = $derived(themeStore.mode);
  let dark_mode = $derived(themeStore.isDark);

  let links = $derived($linksStore);
  let categories = $derived($categoriesStore);
  let tags = $derived($tagsStore);

  // 在分组树中按 id 查找节点（递归）
  const find_category_by_id = (id) => findCategoryById(categories, id);

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
    // 主题加载、系统主题监听、跨窗口同步全部由 themeStore 接管
    await themeStore.init();

    const savedSort = await api.getSetting("sort-by");
    if (savedSort) sort_by = savedSort;
    try {
      const v = (await api.getSetting("check-link-reachability")) !== "false";
      settingsStore.update(s => ({ ...s, check_link_reachability: v }));
    } catch {}
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
      refresh_current_view();
      tagsStore.load();
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
    let unlistenLinkBroken;
    let unlistenLinkMeta;
    try {
      const { listen } = await import("@tauri-apps/api/event");
      unlistenLinksChanged = await listen("links-changed", () => {
        refresh_current_view();
        categoriesStore.load();
        tagsStore.load();
      });
      // 后端在 create_link / update_link 后异步重检可达性,通过此事件回推结果
      unlistenLinkBroken = await listen("link-broken-changed", (e) => {
        const { id, is_broken } = e.payload || {};
        if (typeof id === "number") {
          linksStore.patchItem(id, { is_broken: !!is_broken });
        }
      });
      // 元数据抓取完成后回推 title / description / favicon / og_image
      unlistenLinkMeta = await listen("link-meta-changed", (e) => {
        const payload = e.payload || {};
        const { id, ...patch } = payload;
        if (typeof id === "number") {
          linksStore.patchItem(id, patch);
        }
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
      if (unlistenLinkBroken) unlistenLinkBroken();
      if (unlistenLinkMeta) unlistenLinkMeta();
      if (unlistenMoved) unlistenMoved();
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
            // 还原 anchor 在视口中的位置：新 offsetTop - 旧视口偏移（旧 offsetTop - 旧 scrollTop）
            const newScrollTop = anchorTop2 - anchorOffsetTop + scrollTopBefore;
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
    const { id, name: new_name } = payload;
    // 在 tagsStore.update 之前找到旧名字
    const old_name = $tagsStore.find(t => t.id === id)?.name;
    await tagsStore.update(payload);
    if (old_name && old_name !== new_name) {
      // 本地替换 linksStore 中所有包含旧标签名的链接，与分组重命名效果一致
      linksStore.renameTag(old_name, new_name);
      // 如果当前正在筛选该标签，更新搜索框中的标签名
      if (selected_tag === old_name) {
        selected_tag = new_name;
      }
    }
  }

  async function on_create_tag(name) {
    await tagsStore.create(name);
  }

  let importing = $state(false);
  let sidebar_stats = $state(null);

  async function load_stats() {
    try {
      sidebar_stats = await api.getLinksStats();
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
    // 当前是暗色就切到亮色，反之到暗色（跳过 system 状态）
    await themeStore.setMode(themeStore.isDark ? "light" : "dark");
  }

  async function on_theme_change(mode) {
    await themeStore.setMode(mode);
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
            <SortSelect
              bind:value={sort_by}
              options={[
                { value: "", label: "最近更新" },
                { value: "click_count", label: "最多访问" },
                { value: "last_opened_at", label: "最近打开" },
              ]}
              onchange={(v) => on_sort_change(v)}
            />
          {/if}
          <SearchBar bind:this={search_bar} bind:query={search_query} {filter_chip} onremovefilter={on_remove_filter} onsearch={on_search} ontab={on_tab_sort} />
        </div>
      </header>

      {#if show_stats_view}
        <StatsPanel stats={sidebar_stats} />
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
    <CloseDialog
      onCancel={() => show_close_dialog = false}
      onMinimize={close_to_tray}
      onExit={close_exit}
    />
  {/if}

  {#if show_update_dialog && update_info}
    <UpdateDialog update_info={update_info} {release_notes} onclose={on_update_close} />
  {/if}

  {#if show_release_notes}
    <ReleaseNotesDialog
      version={current_version}
      notes={last_update_notes}
      onClose={() => show_release_notes = false}
    />
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
    /* var(--bg-0) 在亮/暗模式下分别为白/深底，无需再写 .dark 覆盖 */
    background: var(--bg-0);
    z-index: 100;
    pointer-events: auto;
  }

  .import-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--text-2);
  }

  /* import-banner 的小转圈复用 app.css 的 .spinner-sm */
</style>
