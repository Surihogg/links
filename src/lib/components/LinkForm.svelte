<script>
  import TagInput from "./TagInput.svelte";
  import { fetchMeta } from "../api.js";
  import { checkDuplicate } from "../api.js";

  let { link = null, categories = [], onsave, oncancel } = $props();

  let url = $state(link?.url ?? "");
  let title = $state(link?.title ?? "");
  let description = $state(link?.description ?? "");
  let notes = $state(link?.notes ?? "");
  let category_id = $state(link?.category_id ?? null);
  let tags = $state(link?.tags?.slice() ?? []);
  let saving = $state(false);
  let fetching = $state(false);
  let fetch_error = $state("");
  let fetch_timer = null;
  let duplicate_warning = $state("");
  let user_edited = $state({ title: false, description: false });
  let fetched_meta = $state({ favicon_url: "", og_image_url: "" });
  let fetched_url = "";
  let pending_fetch = null;

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
        fetch_error = "这个小站不想被抓取呢，手动填写信息吧";
      }
      fetched_meta = { favicon_url: meta.favicon_url || "", og_image_url: meta.og_image_url || "" };
      fetched_url = u;
      // Auto-suggest tags from keywords when creating a new link with no user tags yet
      if (!link && tags.length === 0 && meta.keywords && meta.keywords.length > 0) {
        tags = meta.keywords.slice(0, 5);
      }
    } catch {
      fetch_error = "抓取失败了，手动填一下信息吧";
    }
    fetching = false;
    pending_fetch = null;
  }

  async function check_dup(u) {
    const existing = await checkDuplicate(u, link?.id ?? null);
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
    // Duplicate check (debounced 300ms)
    setTimeout(() => check_dup(u), 300);
    fetch_timer = setTimeout(() => {
      pending_fetch = do_fetch(u);
    }, 500);
  }

  async function submit() {
    if (!url.trim()) return;
    saving = true;
    onsave?.({
      id: link?.id,
      url: url.trim(),
      title: title.trim() || undefined,
      description: description.trim() || undefined,
      notes: notes.trim() || undefined,
      category_id: category_id || -1,
      tags,
      favicon_url: fetched_meta.favicon_url || undefined,
      og_image_url: fetched_meta.og_image_url || undefined,
    });
  }

  function on_overlay_click(e) {
    if (e.target === e.currentTarget) oncancel?.();
  }

  let btn_text = $derived(
    saving ? "保存中..." : "保存"
  );
  let btn_disabled = $derived(saving);

  async function refresh_meta() {
    const u = url.trim();
    if (!u) return;
    user_edited = { title: false, description: false };
    await do_fetch(u);
  }
</script>

<div class="modal-overlay" onclick={on_overlay_click}>
  <div class="modal">
    <div class="modal-header">
      <h2 class="modal-title">{link ? "编辑链接" : "添加链接"}</h2>
      <button class="modal-close" onclick={oncancel}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
        </svg>
      </button>
    </div>

    <form class="modal-body" onsubmit={(e) => { e.preventDefault(); submit(); }}>
      <div class="field url-field">
        <label class="field-label">URL <span class="required">*</span></label>
        <div class="url-input-wrap">
          <input type="url" bind:value={url} oninput={on_url_input} required placeholder="https://..." class="field-input" />
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
        <label class="field-label">描述</label>
        <textarea bind:value={description} oninput={mark_edited("description")} rows="2" placeholder="会自动帮你抓取哦" class="field-input field-textarea"></textarea>
      </div>

      <div class="field">
        <label class="field-label">备注</label>
        <textarea bind:value={notes} rows="2" placeholder="说说你的想法吧" class="field-input field-textarea"></textarea>
      </div>

      <div class="field">
        <label class="field-label">分组</label>
        <select bind:value={category_id} placeholder="确定不分个组吗" class="field-input" onchange={() => { if (category_id === "") category_id = null; }}>
          <option value="">无分组</option>
          {#each categories as cat}
            <option value={cat.id}>{cat.name}</option>
          {/each}
        </select>
      </div>

      <div class="field">
        <label class="field-label">标签</label>
        <TagInput bind:tags />
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
          {/if}
        </div>
        <div class="footer-right">
          <button type="button" onclick={oncancel} class="btn btn-secondary">取消</button>
          <button type="submit" disabled={btn_disabled} class="btn btn-primary">
            {btn_text}
          </button>
        </div>
      </div>
    </form>
  </div>
</div>

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
  }

  .modal {
    width: 100%;
    max-width: 480px;
    max-height: 90vh;
    background: var(--bg-0);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-xl);
    border: 1px solid var(--border-0);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px 12px;
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
    padding: 0 20px 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    overflow-y: auto;
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
    padding-right: 70px;
  }

  .fetch-hint {
    position: absolute;
    right: 10px;
    font-size: 11px;
    color: var(--accent);
    pointer-events: none;
  }

  .fetch-error {
    font-size: 12px;
    color: var(--text-3);
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

  .url-input-wrap .field-input {
    padding-right: 42px;
  }

  .url-field {
    position: relative;
  }

  .dup-warning {
    position: absolute;
    right: 0;
    top: 100%;
    font-size: 11px;
    color: #d97706;
    margin-top: 2px;
    pointer-events: none;
  }
  :global(.dark) .dup-warning {
    color: #fbbf24;
  }

  .field-textarea {
    resize: none;
    line-height: 1.5;
  }

  select.field-input {
    cursor: pointer;
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg width='10' height='6' viewBox='0 0 10 6' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%23999' stroke-width='1.4' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 10px center;
    padding-right: 28px;
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 8px;
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

  .btn {
    padding: 7px 16px;
    border: none;
    border-radius: var(--radius-md);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--transition);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--accent);
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .btn-secondary {
    background: var(--bg-2);
    color: var(--text-2);
  }

  .btn-secondary:hover {
    background: var(--border-1);
    color: var(--text-1);
  }
</style>
