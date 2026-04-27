<script>
  import { onMount } from "svelte";
  import TagInput from "../lib/components/TagInput.svelte";
  import CategoryInput from "../lib/components/CategoryInput.svelte";
  import { fetchMeta, checkDuplicate, createLink, listCategories, getSetting } from "../lib/api.js";
  import { waitForBackendReady } from "../lib/ready.js";
  import { emit, listen } from "@tauri-apps/api/event";

  let url = $state("");
  let title = $state("");
  let description = $state("");
  let notes = $state("");
  let category_id = $state(null);
  let tags = $state([]);
  let saving = $state(false);
  let fetching = $state(false);
  let fetch_error = $state("");
  let fetch_timer = null;
  let duplicate_warning = $state("");
  let user_edited = $state({ title: false, description: false });
  let fetched_meta = $state({ favicon_url: "", og_image_url: "" });
  let categories = $state([]);
  let message = $state("");
  let pending_fetch = null;
  let dark_mode = $state(false);
  let theme_mode = $state("system");
  let url_input;

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

  function reset_form() {
    url = "";
    title = "";
    description = "";
    notes = "";
    category_id = null;
    tags = [];
    saving = false;
    fetching = false;
    fetch_error = "";
    duplicate_warning = "";
    user_edited = { title: false, description: false };
    fetched_meta = { favicon_url: "", og_image_url: "" };
    message = "";
    clearTimeout(fetch_timer);
    pending_fetch = null;
  }

  function flatten_categories(tree) {
    const result = [];
    for (const cat of tree) {
      result.push({ id: cat.id, name: cat.name });
      if (cat.children) result.push(...flatten_categories(cat.children));
    }
    return result;
  }

  let flatCategories = $derived(flatten_categories(categories));

  onMount(async () => {
    await waitForBackendReady();

    let saved = await getSetting("theme-mode");
    if (!saved) {
      const legacyDark = await getSetting("dark-mode");
      saved = legacyDark === "true" ? "dark" : (legacyDark === "false" ? "light" : "system");
    }
    apply_theme(saved || "system");
    document.documentElement.classList.add("theme-ready");

    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    function on_system_theme(e) {
      if (theme_mode === "system") {
        apply_theme();
      }
    }
    if (mq) mq.addEventListener("change", on_system_theme);

    const unlistenTheme = await listen("theme-changed", (e) => {
      apply_theme(e.payload);
    });
    const unlistenShown = await listen("quick-add-shown", () => {
      reset_form();
      setTimeout(() => url_input?.focus(), 50);
    });

    listCategories().then(c => categories = c);
    const handle_keydown = (e) => {
      if (e.key === "Escape") {
        e.preventDefault();
        close_window();
      }
    };
    window.addEventListener("keydown", handle_keydown);
    return () => {
      window.removeEventListener("keydown", handle_keydown);
      unlistenTheme();
      unlistenShown();
    };
  });

  function mark_edited(field) {
    return (e) => { user_edited[field] = true; };
  }

  async function do_fetch(u) {
    fetching = true;
    fetch_error = "";
    try {
      const meta = await fetchMeta(u);
      if (meta.title || meta.description) {
        if (!user_edited.title && meta.title) title = meta.title;
        if (!user_edited.description && meta.description) description = meta.description;
      } else {
        fetch_error = "这个小站很神秘呢，手动补充一下缺失的信息吧";
      }
      fetched_meta = { favicon_url: meta.favicon_url || "", og_image_url: meta.og_image_url || "" };
      if (tags.length === 0 && meta.keywords && meta.keywords.length > 0) {
        tags = meta.keywords.slice(0, 5);
      }
    } catch {
      fetch_error = "人家不让抓，只能麻烦您动动小手了";
    }
    fetching = false;
    pending_fetch = null;
  }

  async function check_dup(u) {
    const existing = await checkDuplicate(u, null);
    if (existing) {
      duplicate_warning = existing.title ? `已有相同链接：${existing.title}` : "已有相同链接";
    } else {
      duplicate_warning = "";
    }
  }

  function on_url_input() {
    clearTimeout(fetch_timer);
    fetch_error = "";
    const u = url.trim();
    if (!u || !/^https?:\/\//.test(u)) return;
    setTimeout(() => check_dup(u), 300);
    fetch_timer = setTimeout(() => {
      pending_fetch = do_fetch(u);
    }, 500);
  }

  async function submit() {
    if (!url.trim()) return;
    saving = true;
    message = "";
    try {
      await createLink({
        url: url.trim(),
        title: title.trim() || undefined,
        description: description.trim() || undefined,
        notes: notes.trim() || undefined,
        category_id: category_id || -1,
        tags,
        favicon_url: fetched_meta.favicon_url || undefined,
        og_image_url: fetched_meta.og_image_url || undefined,
      });
      // 保存成功，通知主程序刷新，然后关闭窗口
      await emit("links-changed");
      await close_window();
    } catch {
      message = "保存失败 ✗";
    } finally {
      saving = false;
    }
  }

  async function refresh_meta() {
    const u = url.trim();
    if (!u) return;
    user_edited = { title: false, description: false };
    await do_fetch(u);
  }

  async function close_window() {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().hide();
  }

  let btn_text = $derived(saving ? "保存中..." : "保存");
  let btn_disabled = $derived(saving);
</script>

<div class="quick-add {dark_mode ? 'dark' : ''}">
  <div class="modal-header" data-tauri-drag-region>
    <h2 class="modal-title">添加链接</h2>
    <button class="modal-close" onclick={close_window}>
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
        <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
      </svg>
    </button>
  </div>

  <form class="modal-body" onsubmit={(e) => { e.preventDefault(); submit(); }}>
    <div class="field url-field">
      <label class="field-label">URL <span class="required">*</span></label>
      <div class="url-input-wrap">
        <input bind:this={url_input} type="url" bind:value={url} oninput={on_url_input} required placeholder="https://..." class="field-input" />
        <button type="button" class="refresh-btn" onclick={refresh_meta} disabled={fetching || !url.trim()} title="重新抓取元数据">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class={fetching ? 'spin-anim' : ''}>
            <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0118.8-4.3M22 12.5a10 10 0 01-18.8 4.2"/>
          </svg>
        </button>
      </div>
      {#if duplicate_warning}
        <span class="dup-warning">{duplicate_warning}</span>
      {/if}
    </div>

    <div class="field">
      <label class="field-label">标题</label>
      <input type="text" bind:value={title} oninput={mark_edited("title")} placeholder="会自动帮你抓取哦" class="field-input" />
    </div>

    <div class="field">
      <label class="field-label">分组</label>
      <CategoryInput bind:selectedId={category_id} categories={flatCategories} />
    </div>

    <div class="field tag-field">
      <label class="field-label">标签</label>
      <TagInput bind:tags />
    </div>

    <div class="field desc-field">
      <label class="field-label">描述</label>
      <textarea bind:value={description} oninput={mark_edited("description")} rows="2" placeholder="会自动帮你抓取哦" class="field-input field-textarea"></textarea>
    </div>

    <div class="field notes-field">
      <label class="field-label">备注</label>
      <textarea bind:value={notes} rows="2" placeholder="说说你的想法吧" class="field-input field-textarea"></textarea>
    </div>

    <div class="modal-footer">
      <div class="footer-left">
        {#if fetching}
          <span class="fetch-indicator">
            <span class="spinner-sm"></span>
            抓取中...
          </span>
        {:else if fetch_error}
          <span class="fetch-error">{fetch_error}</span>
        {:else if message}
          <span class="message">{message}</span>
        {/if}
      </div>
      <div class="footer-right">
        <button type="button" onclick={close_window} class="btn btn-secondary">取消</button>
        <button type="submit" disabled={btn_disabled} class="btn btn-primary">
          {btn_text}
        </button>
      </div>
    </div>
  </form>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    height: 100vh;
    width: 100vw;
  }

  .quick-add {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background: var(--bg-0);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px 12px;
    flex-shrink: 0;
  }

  .modal-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-0);
  }

  .modal-close {
    width: 28px;
    height: 28px;
    border: none;
    background: none;
    color: var(--text-3);
    cursor: pointer;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition);
  }

  .modal-close:hover {
    background: var(--bg-2);
    color: var(--text-1);
  }

  .modal-body {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px 16px;
    padding: 0 20px 16px;
    overflow-y: auto;
    flex: 1;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-2);
  }

  .required {
    color: var(--danger);
  }

  .field-input {
    width: 100%;
    padding: 7px 10px;
    border: 1px solid var(--border-1);
    border-radius: var(--radius-md);
    background: var(--bg-0);
    color: var(--text-0);
    font-size: 13px;
    outline: none;
    transition: all var(--transition);
  }

  .field-input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .field-input::placeholder {
    color: var(--text-3);
  }

  .url-input-wrap {
    position: relative;
    display: flex;
    align-items: center;
  }

  .url-input-wrap .field-input {
    flex: 1;
    padding-right: 42px;
  }

  .fetch-error {
    font-size: 12px;
    color: var(--text-3);
  }

  .message {
    font-size: 12px;
    color: var(--success);
  }

  .refresh-btn {
    position: absolute;
    right: 10px;
    width: 26px;
    height: 26px;
    border: none;
    background: var(--bg-2);
    color: var(--text-2);
    border-radius: var(--radius-sm);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition);
  }

  .refresh-btn:hover:not(:disabled) {
    background: var(--accent-soft);
    color: var(--accent);
  }

  .refresh-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .spin-anim {
    animation: spin 0.6s linear infinite;
  }

  .url-field {
    position: relative;
  }

  .dup-warning {
    position: absolute;
    right: 0;
    top: 100%;
    font-size: 11px;
    color: var(--warning);
    margin-top: 2px;
    pointer-events: none;
  }

  .field-textarea {
    resize: none;
    line-height: 1.5;
  }

  .url-field,
  .tag-field,
  .desc-field,
  .notes-field,
  .modal-footer {
    grid-column: span 2;
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 4px;
  }

  .footer-left {
    display: flex;
    align-items: center;
  }

  .footer-right {
    display: flex;
    gap: 8px;
  }

  .fetch-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-3);
  }

  .spinner-sm {
    width: 12px;
    height: 12px;
    border: 1.5px solid var(--border-1);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin { to { transform: rotate(360deg); } }
</style>
