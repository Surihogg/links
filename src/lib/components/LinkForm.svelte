<script>
  import TagInput from "./TagInput.svelte";
  import CategoryInput from "./CategoryInput.svelte";
  import { fetchMeta } from "../api.js";
  import { checkDuplicate } from "../api.js";
  import { createImeGuard } from "../utils/imeGuard.svelte.js";

  /**
   * @typedef {object} Props
   * @property {object|null} [link] 待编辑的链接；为空表示新建
   * @property {Array} [categories] 分组树
   * @property {(payload: object) => void} [onsave] 保存按钮回调（payload 见 submit）
   * @property {() => void} [oncancel] 取消按钮回调
   * @property {"modal"|"standalone"} [mode] 渲染模式：modal=独立弹窗（默认），
   *   standalone=不带 modal-overlay 与 header，直接嵌入父容器（QuickAdd 用）
   * @property {string} [submitText] 自定义保存按钮文案
   * @property {string} [cancelText] 自定义取消按钮文案
   * @property {string} [savingText] 保存中文案
   */
  let {
    link = null,
    categories = [],
    onsave,
    oncancel,
    mode = "modal",
    submitText = "保存",
    cancelText = "取消",
    savingText = "保存中...",
  } = $props();
  // IME 输入法组词期间禁止表单提交，避免误触 Enter 关闭弹窗
  const ime_guard = createImeGuard();
  $effect(() => ime_guard.attach());

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
  let url_input;

  $effect(() => {
    if (url_input) setTimeout(() => url_input.focus(), 50);
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
      fetched_url = u;
      // Auto-suggest tags from keywords when creating a new link with no user tags yet
      if (!link && tags.length === 0 && meta.keywords && meta.keywords.length > 0) {
        tags = meta.keywords.slice(0, 5);
      }
    } catch {
      fetch_error = "人家不让抓，只能麻烦您动动小手了";
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
      notes: notes.trim(),
      category_id: category_id || -1,
      tags,
      favicon_url: fetched_meta.favicon_url || undefined,
      og_image_url: fetched_meta.og_image_url || undefined,
    });
  }

  function on_overlay_click(e) {
    // 只有直接点击 overlay（不是从 modal 内拖拽出来的）才关闭窗口
    if (e.target === e.currentTarget && !mouseDownInside) {
      oncancel?.();
    }
    mouseDownInside = false;
  }

  let btn_text = $derived(saving ? savingText : submitText);
  let btn_disabled = $derived(saving);

  /**
   * 父组件可调用此方法把保存状态归位（standalone 模式下保存成功后会用到）。
   * 也可用于在 QuickAdd 这类窗口"提交后清空表单"的场景。
   */
  export function reset(initial = {}) {
    url = initial.url ?? "";
    title = initial.title ?? "";
    description = initial.description ?? "";
    notes = initial.notes ?? "";
    category_id = initial.category_id ?? null;
    tags = initial.tags?.slice() ?? [];
    saving = false;
    fetching = false;
    fetch_error = "";
    duplicate_warning = "";
    user_edited = { title: false, description: false };
    fetched_meta = { favicon_url: "", og_image_url: "" };
    fetched_url = "";
    clearTimeout(fetch_timer);
    pending_fetch = null;
  }

  export function focusUrl() {
    setTimeout(() => url_input?.focus(), 50);
  }

  /** 父组件可设置 saving=false 解锁按钮（如保存失败时） */
  export function setSaving(v) {
    saving = !!v;
  }

  /** 父组件可在"添加成功"后填入 message（仅 standalone 模式样式有效） */
  export function getValues() {
    return { url, title, description, notes, category_id, tags, fetched_meta };
  }

  /** 由父组件触发一次去重检查 + 元数据抓取（如 deep-link 预填后）。 */
  export function triggerFetch() {
    on_url_input();
  }

  async function refresh_meta() {
    const u = url.trim();
    if (!u) return;
    // 不重置 user_edited，保留用户已手动编辑的字段
    await do_fetch(u);
  }

  let mouseDownInside = false;

  function on_overlay_mousedown(e) {
    // 记录 mousedown 是否发生在 modal 内容区内
    // 如果是，说明用户可能在选中文本或拖拽，后续 click 不应关闭窗口
    mouseDownInside = !!e.target.closest('.modal');
  }

  function on_input_keydown(e) {
    if (e.key === 'Enter') {
      e.preventDefault();
    }
  }
</script>

<!-- 公共表单主体（modal 与 standalone 共用） -->
{#snippet formBody()}
  <form class={mode === "modal" ? "modal-body" : "form-body standalone"}
        onsubmit={(e) => { e.preventDefault(); if (ime_guard.active) return; submit(); }}>
    <div class="field url-field">
      <div class="field-label-row">
        <label class="field-label">URL <span class="required">*</span></label>
        {#if duplicate_warning}
          <span class="dup-warning">{duplicate_warning}</span>
        {/if}
      </div>
      <div class="url-input-wrap">
        <input type="url" bind:this={url_input} bind:value={url} oninput={on_url_input} onkeydown={on_input_keydown} required placeholder="https://..." class="field-input" />
        <button type="button" class="refresh-btn" onclick={refresh_meta} disabled={fetching || !url.trim()} title="重新抓取元数据">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class={fetching ? 'spin-anim' : ''}>
            <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0118.8-4.3M22 12.5a10 10 0 01-18.8 4.2"/>
          </svg>
        </button>
      </div>
    </div>

    <div class="field">
      <label class="field-label">标题</label>
      <input type="text" bind:value={title} oninput={mark_edited("title")} onkeydown={on_input_keydown} placeholder="会自动帮你抓取哦" class="field-input" />
    </div>

    <div class="field">
      <label class="field-label">分组</label>
      <CategoryInput bind:selectedId={category_id} {categories} />
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
        {/if}
      </div>
      <div class="footer-right">
        <button type="button" onclick={oncancel} class="btn btn-secondary">{cancelText}</button>
        <button type="submit" disabled={btn_disabled} class="btn btn-primary">
          {btn_text}
        </button>
      </div>
    </div>
  </form>
{/snippet}

{#if mode === "modal"}
  <div class="modal-overlay" onclick={on_overlay_click} onmousedown={on_overlay_mousedown}>
    <div class="modal">
      <div class="modal-header">
        <h2 class="modal-title">{link ? "编辑链接" : "添加链接"}</h2>
        <button type="button" class="modal-close" onclick={oncancel}>
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
            <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
          </svg>
        </button>
      </div>
      {@render formBody()}
    </div>
  </div>
{:else}
  {@render formBody()}
{/if}

<style>
  .modal {
    max-width: 480px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
  }

  .modal-body {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px 16px;
    overflow-y: auto;
  }

  /* standalone 模式：去掉 modal-body 的 padding/overflow，由父容器控制 */
  .form-body.standalone {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px 16px;
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

  .field-label-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .dup-warning {
    font-size: 11px;
    color: var(--warning);
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
</style>
