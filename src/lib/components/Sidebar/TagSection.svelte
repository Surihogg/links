<script>
  // 侧边栏标签管理节：折叠头 / 新建表单 / 搜索框 / 无标签项 /
  // 标签列表（含重命名 / 删除两阶段确认）
  //
  // 与 CategorySection 结构相似但更简单（无树形、无拖拽、无子项）。
  // 共享的 .cat-name / .cat-delete-hint / .rename-input / .section-* 样式
  // 部分在本组件内部维护一份（与 CategorySection 重复但保证独立运行）。

  import {
    cyclePlaceholder,
    DEFAULT_PLACEHOLDER,
  } from "../../utils/cyclePlaceholder.js";

  /**
   * @typedef {object} Tag
   * @property {number} id
   * @property {string} name
   *
   * @typedef {object} Props
   * @property {Array<Tag>} tags
   * @property {string|null} selected_tag 当前选中的标签名 / '__untagged__'
   * @property {boolean} collapsed
   * @property {(tagName: string|null) => void} [onSelect]
   * @property {(name: string) => void} [onCreate]
   * @property {(id: number) => void} [onDelete]
   * @property {(payload: { id: number, name: string }) => void} [onRename]
   * @property {() => void} [onToggle]
   */
  let {
    tags = [],
    selected_tag = null,
    collapsed = false,
    onSelect,
    onCreate,
    onDelete,
    onRename,
    onToggle,
  } = $props();

  let tag_search = $state("");
  let show_new_tag = $state(false);
  let new_tag_name = $state("");
  let tag_placeholder = $state(DEFAULT_PLACEHOLDER);
  let deleting_tag_id = $state(null);
  let editing_tag_id = $state(null);
  let editing_tag_name = $state("");

  let filtered_tags = $derived(
    tag_search.trim()
      ? tags.filter((t) =>
          t.name.toLowerCase().includes(tag_search.trim().toLowerCase())
        )
      : tags
  );

  function submit_tag() {
    if (!new_tag_name.trim()) {
      tag_placeholder = cyclePlaceholder(tag_placeholder);
      return;
    }
    const name = new_tag_name.trim();
    if (tags.some((t) => t.name.toLowerCase() === name.toLowerCase())) {
      new_tag_name = "";
      tag_placeholder = "已经有这个标签了";
      return;
    }
    onCreate?.(name);
    new_tag_name = "";
    tag_placeholder = DEFAULT_PLACEHOLDER;
  }

  function handle_delete_tag(e, id) {
    e.stopPropagation();
    if (deleting_tag_id === id) {
      onDelete?.(id);
      deleting_tag_id = null;
    } else {
      deleting_tag_id = id;
    }
  }

  function reset_tag_delete() {
    deleting_tag_id = null;
  }

  function start_rename_tag(e, tag) {
    e.stopPropagation();
    editing_tag_id = tag.id;
    editing_tag_name = tag.name;
  }

  function confirm_rename_tag(tag) {
    const name = editing_tag_name.trim();
    if (name && name !== tag.name) {
      onRename?.({ id: tag.id, name });
    }
    editing_tag_id = null;
    editing_tag_name = "";
  }

  function cancel_rename_tag() {
    editing_tag_id = null;
    editing_tag_name = "";
  }
</script>

<div class="section-header" onclick={onToggle}>
  <span class="section-label" style="cursor:pointer;">
    <svg
      class="chevron"
      width="10"
      height="10"
      viewBox="0 0 10 10"
      fill="none"
      stroke="currentColor"
      stroke-width="1.4"
      stroke-linecap="round"
      stroke-linejoin="round"
      style="transform: rotate({collapsed ? 0 : 90}deg); transition: transform var(--transition);"
    >
      <path d="M3 1l4 4-4 4"/>
    </svg>
    标签管理
  </span>
  {#if !collapsed}
    <button
      class="section-action"
      onclick={(e) => { e.stopPropagation(); show_new_tag = true; }}
      title="新建标签"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
        <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
    </button>
  {/if}
</div>

{#if !collapsed}
  {#if show_new_tag}
    <form
      class="new-cat-form"
      onsubmit={(e) => { e.preventDefault(); submit_tag(); }}
      onfocusout={(e) => {
        if (!e.currentTarget.contains(e.relatedTarget)) {
          show_new_tag = false;
          new_tag_name = "";
          tag_placeholder = DEFAULT_PLACEHOLDER;
        }
      }}
    >
      <input
        type="text"
        bind:value={new_tag_name}
        placeholder={tag_placeholder}
        class="new-cat-input"
        onkeydown={(e) => {
          if (e.key === "Enter" && e.isComposing) return;
          if (e.key === "Escape") {
            show_new_tag = false;
            new_tag_name = "";
            tag_placeholder = DEFAULT_PLACEHOLDER;
          }
        }}
        autofocus
      />
    </form>
  {/if}
  {#if tags.length > 10}
    <div class="section-search">
      <input
        type="text"
        bind:value={tag_search}
        placeholder="找找你的标签"
        class="section-search-input"
      />
    </div>
  {/if}
  <div class="tag-list">
    <button
      class="nav-item tag-item"
      class:active={selected_tag === "__untagged__"}
      style="padding-left: 12px"
      onclick={() => onSelect?.("__untagged__")}
    >
      <span class="tag-icon-area">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="3 2" class="tag-icon icon-static">
          <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/>
          <line x1="7" y1="7" x2="7.01" y2="7"/>
        </svg>
      </span>
      <span class="cat-name">无标签</span>
    </button>
    {#each filtered_tags as tag (tag.id)}
      <button
        class="nav-item tag-item"
        class:active={selected_tag === tag.name}
        style="padding-left: 12px"
        onclick={() => { if (editing_tag_id !== tag.id) onSelect?.(tag.name); }}
        onmouseleave={() => reset_tag_delete()}
      >
        <span
          class="tag-icon-area"
          onclick={(e) => handle_delete_tag(e, tag.id)}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="tag-icon icon-tag">
            <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/>
            <line x1="7" y1="7" x2="7.01" y2="7"/>
          </svg>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="tag-icon icon-delete">
            <path d="M3 6h18M8 6V4a2 2 0 012-2h4a2 2 0 012 2v2M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6"/>
          </svg>
        </span>
        {#if editing_tag_id === tag.id}
          <input
            type="text"
            class="rename-input"
            bind:value={editing_tag_name}
            onkeydown={(e) => {
              if (e.key === "Enter") { e.preventDefault(); confirm_rename_tag(tag); }
              if (e.key === "Escape") cancel_rename_tag();
            }}
            onblur={() => confirm_rename_tag(tag)}
            onclick={(e) => e.stopPropagation()}
            autofocus
          />
        {:else if deleting_tag_id === tag.id}
          <span class="cat-delete-hint">再点一下就删除</span>
        {:else}
          <span class="cat-name">{tag.name}</span>
        {/if}
        <span
          class="tag-action-btn"
          onclick={(e) => start_rename_tag(e, tag)}
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/>
          </svg>
        </span>
      </button>
    {/each}
  </div>
{/if}

<style>
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 6px 6px;
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: background var(--transition);
  }

  .section-header:hover {
    background: var(--bg-hover);
  }

  .section-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-2);
    letter-spacing: 0.3px;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .section-action {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-3);
    padding: 2px;
    border-radius: 4px;
    display: flex;
    transition: all var(--transition);
  }

  .section-action:hover {
    color: var(--text-1);
    background: var(--bg-2);
  }

  /* —— 标签列表特有 —— */

  .tag-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .tag-item {
    font-size: 13px;
    position: relative;
  }

  .tag-item:hover {
    background: var(--bg-2);
  }

  .tag-item.active {
    background: var(--accent-soft);
    color: var(--accent-text);
    font-weight: 500;
  }

  /* —— 图标区 hover 切换为删除图标 —— */

  .tag-icon-area {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    margin: 0 1px;
    cursor: pointer;
  }

  .tag-icon {
    flex-shrink: 0;
    margin: 0 5px;
  }

  .tag-icon-area .tag-icon {
    margin: 0;
  }

  .icon-static {
    opacity: 0.5;
  }

  .icon-tag {
    display: block;
  }

  .icon-delete {
    display: none;
    color: var(--danger);
  }

  .nav-item:hover .tag-icon-area .icon-tag {
    display: none;
  }

  .nav-item:hover .tag-icon-area .icon-delete {
    display: block;
  }

  /* —— action 按钮 —— */

  .tag-action-btn {
    display: none;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: 4px;
    margin-left: auto;
    flex-shrink: 0;
    color: var(--text-3);
    cursor: pointer;
    transition: all var(--transition);
  }

  .tag-item:hover .tag-action-btn {
    display: flex;
  }

  .tag-action-btn:hover {
    color: var(--text-1);
    background: var(--bg-hover);
  }

  /* —— 名字 / 删除提示 / 重命名输入（与 CategorySection 共享但本组件
       独立维护一份，避免跨组件依赖） —— */

  .cat-name,
  .cat-delete-hint {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
  }

  .cat-delete-hint {
    color: var(--danger);
    flex: 1;
    text-align: left;
  }

  .rename-input {
    flex: 1;
    min-width: 0;
    padding: 1px 4px;
    border: 1px solid var(--accent);
    border-radius: 3px;
    background: var(--bg-0);
    color: var(--text-0);
    font-size: 13px;
    outline: none;
    box-shadow: 0 0 0 2px var(--accent-soft);
  }

  /* —— 新建表单 / 节内搜索 —— */

  .new-cat-form {
    padding: 8px 4px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .new-cat-input {
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

  .new-cat-input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .section-search {
    padding: 6px 6px 4px;
  }

  .section-search-input {
    width: 100%;
    padding: 4px 8px;
    border: 1px solid var(--border-1);
    border-radius: var(--radius-sm);
    background: var(--bg-0);
    color: var(--text-0);
    font-size: 11px;
    outline: none;
    transition: border-color var(--transition);
  }

  .section-search-input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
</style>
