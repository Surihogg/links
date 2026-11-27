// 在页面加载任何模块前安装 Tauri IPC 测试替身。
// 每个 Playwright 浏览器上下文都有独立数据，不读写真实 links.db / config.json。
export function installTauriMock(fixture = {}) {
  const clone = (value) => structuredClone(value);
  const now = "2026-11-26 10:00:00";
  const makeLink = (value) => ({
    id: value.id,
    url: value.url,
    title: value.title || "",
    description: value.description || "",
    notes: value.notes || "",
    favicon_url: value.favicon_url || "",
    og_image_url: value.og_image_url || "",
    category_id: value.category_id ?? null,
    is_favorite: !!value.is_favorite,
    is_broken: !!value.is_broken,
    click_count: value.click_count || 0,
    last_opened_at: value.last_opened_at ?? null,
    tags: [...(value.tags || [])],
    created_at: value.created_at || now,
    updated_at: value.updated_at || now,
  });

  const state = {
    links: (fixture.links || [
      {
        id: 1,
        url: "https://example.com/rust",
        title: "Rust 编程语言",
        description: "桌面应用开发指南",
        tags: ["runtime"],
        category_id: 1,
      },
      {
        id: 2,
        url: "https://example.com/other",
        title: "烹饪手册",
        description: "家常菜谱",
        tags: ["生活"],
      },
    ]).map(makeLink),
    categories: clone(fixture.categories || [
      { id: 1, name: "开发工具", parent_id: null, sort_order: 0, children: [], created_at: now, updated_at: now },
    ]),
    settings: {
      "theme-mode": "light",
      "auto-check-update": "false",
      "last-known-version": "1.3.6",
      "check-link-reachability": "false",
      ...fixture.settings,
    },
    windowLabel: fixture.windowLabel || "main",
    size: { width: 900, height: 600 },
    position: { x: 100, y: 100 },
    visible: false,
    minimized: false,
    nextLinkId: 100,
  };
  const calls = [];
  const frontendLogs = [];
  const callbacks = new Map();
  const listeners = new Map();
  let callbackId = 0;
  let listenerId = 0;

  const emitEvent = (event, payload) => {
    for (const [id, listener] of listeners) {
      if (listener.event !== event) continue;
      const callback = callbacks.get(listener.handler);
      callback?.({ event, id, payload });
      if (callback?.once) {
        callbacks.delete(listener.handler);
        listeners.delete(id);
      }
    }
  };

  const monitor = {
    name: "Mock Monitor",
    scaleFactor: 1,
    position: { x: 0, y: 0 },
    size: { width: 1920, height: 1080 },
    workArea: { position: { x: 0, y: 0 }, size: { width: 1920, height: 1040 } },
  };

  const unwrapValue = (value) => {
    if (value?.size) return value.size;
    return value?.Logical || value?.Physical || value;
  };

  const categoryName = (id) => {
    const walk = (items) => {
      for (const item of items) {
        if (item.id === id) return item.name;
        const nested = walk(item.children || []);
        if (nested) return nested;
      }
      return "";
    };
    return walk(state.categories);
  };

  const filteredLinks = (params = {}, search = false) => {
    const terms = search ? String(params.query || "").trim().toLocaleLowerCase().split(/\s+/).filter(Boolean) : [];
    return state.links.filter((link) => {
      if (params.category_id !== undefined) {
        if (params.category_id === null && link.category_id !== null) return false;
        if (params.category_id !== null && link.category_id !== params.category_id) return false;
      }
      if (params.favorite_only && !link.is_favorite) return false;
      if (params.untagged_only && link.tags.length) return false;
      if (params.tag && !link.tags.includes(params.tag)) return false;
      if (!terms.length) return true;
      const fields = [
        link.title,
        link.description,
        link.notes,
        link.url,
        link.tags.join(" "),
        categoryName(link.category_id),
      ].join(" ").toLocaleLowerCase();
      return terms.every((term) => fields.includes(term));
    });
  };

  const pageResult = (items, params = {}) => {
    const page = params.page || 1;
    const perPage = params.per_page || 30;
    const offset = (page - 1) * perPage;
    return {
      items: clone(items.slice(offset, offset + perPage)),
      total: items.length,
      page,
      per_page: perPage,
    };
  };

  const handleCommand = async (cmd, args) => {
    switch (cmd) {
      case "get_setting":
        return state.settings[args.key] ?? null;
      case "set_setting":
        state.settings[args.key] = args.value;
        return null;
      case "check_startup_deep_link":
        return false;
      case "pop_pending_deep_link":
        return null;
      case "list_links":
        return pageResult(filteredLinks(args.params), args.params);
      case "search_links":
        return pageResult(filteredLinks(args.params, true), args.params);
      case "check_duplicate": {
        const normalized = args.url.trim().replace(/\/+$/, "");
        const found = state.links.find((link) => link.id !== args.exclude_id
          && link.url.replace(/\/+$/, "") === normalized);
        return found ? clone(found) : null;
      }
      case "create_link": {
        const link = makeLink({ ...args.payload, id: state.nextLinkId++ });
        state.links.unshift(link);
        return clone(link);
      }
      case "update_link": {
        const link = state.links.find((item) => item.id === args.payload.id);
        if (!link) throw new Error("链接不存在");
        const patch = { ...args.payload };
        if (patch.tags) patch.tags = [...patch.tags];
        if (patch.category_id === -1) patch.category_id = null;
        Object.assign(link, patch);
        return clone(link);
      }
      case "delete_link":
        state.links = state.links.filter((link) => link.id !== args.id);
        return null;
      case "list_categories":
        return clone(state.categories);
      case "list_tags": {
        const names = [...new Set(state.links.flatMap((link) => link.tags))];
        return names.map((name, index) => ({ id: index + 1, name, updated_at: now }));
      }
      case "get_links_stats":
        return { total: state.links.length, this_week: state.links.length, top: [] };
      case "fetch_metadata":
        return clone(fixture.metadata || { title: "", description: "", favicon_url: "", og_image_url: "", keywords: [] });
      case "check_link_status":
        return true;
      case "log_frontend_event":
        frontendLogs.push(clone(args.payload));
        return null;
      case "get_local_server_info":
        return { port: 12345, token: "test-token" };
      case "plugin:app|version":
        return "1.3.6";
      case "plugin:event|listen": {
        listenerId += 1;
        listeners.set(listenerId, { event: args.event, handler: args.handler });
        return listenerId;
      }
      case "plugin:event|unlisten":
        listeners.delete(args.eventId);
        return null;
      case "plugin:event|emit":
        emitEvent(args.event, args.payload);
        return null;
      case "plugin:window|get_all_windows":
        return ["main", "quick-add", "spotlight"];
      case "plugin:window|inner_size":
      case "plugin:window|outer_size":
        return clone(state.size);
      case "plugin:window|outer_position":
      case "plugin:window|inner_position":
        return clone(state.position);
      case "plugin:window|scale_factor":
        return 1;
      case "plugin:window|set_size":
        state.size = { width: unwrapValue(args.value).width, height: unwrapValue(args.value).height };
        return null;
      case "plugin:window|set_position":
        state.position = { x: unwrapValue(args.value).x, y: unwrapValue(args.value).y };
        return null;
      case "plugin:window|center":
        state.position = { x: 100, y: 100 };
        return null;
      case "plugin:window|available_monitors":
      case "plugin:window|current_monitor":
      case "plugin:window|primary_monitor":
        return cmd.endsWith("available_monitors") ? [monitor] : monitor;
      case "plugin:window|is_visible":
        return state.visible;
      case "plugin:window|is_minimized":
        return state.minimized;
      case "plugin:window|show":
        state.visible = true;
        return null;
      case "plugin:window|hide":
        state.visible = false;
        return null;
      case "plugin:window|minimize":
        state.minimized = true;
        return null;
      case "plugin:window|unminimize":
        state.minimized = false;
        return null;
      case "plugin:window|set_focus":
        return null;
      default:
        return null;
    }
  };

  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
    unregisterListener(event, id) {
      listeners.delete(id);
    },
  };
  window.__TAURI_INTERNALS__ = {
    metadata: {
      currentWindow: { label: state.windowLabel },
      currentWebview: { label: state.windowLabel },
    },
    transformCallback(callback, once = false) {
      callbackId += 1;
      callbacks.set(callbackId, callback);
      callbacks.get(callbackId).once = once;
      return callbackId;
    },
    unregisterCallback(id) {
      callbacks.delete(id);
    },
    convertFileSrc(path) {
      return path;
    },
    async invoke(cmd, args = {}) {
      const serializedArgs = JSON.parse(JSON.stringify(args));
      calls.push({ cmd, args: serializedArgs, windowLabel: state.windowLabel });
      return handleCommand(cmd, args);
    },
  };

  window.__LINKS_TEST__ = {
    state,
    calls,
    frontendLogs,
    emit: emitEvent,
    serializedCalls() {
      return clone(calls);
    },
  };
}
