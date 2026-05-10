<script>
  // 侧边栏分组管理节：折叠头 / 新建表单 / 搜索框 / 未分组项 /
  // 分组列表（含拖拽 / 重命名 / 删除两阶段确认 / 子分组创建）
  //
  // 从 Sidebar.svelte 抽出，与 TagSection 并列。
  // 拖拽逻辑由 categoryDragHandler 提供；树扁平化由 categoryTree 提供；
  // placeholder 状态机由 cyclePlaceholder 提供。

  import {
    cyclePlaceholder,
    DEFAULT_PLACEHOLDER,
  } from "../../utils/cyclePlaceholder.js";
  import { flattenCategories } from "../../utils/categoryTree.js";

  /**
   * @typedef {object} Props
   * @property {Array} categories 分组树
   * @property {string|number|null} selected_id 当前选中的分组 id 或 'uncategorized'
   * @property {boolean} collapsed 整节是否折叠
   * @property {{ dragId: any, dropTargetId: any, isDescendant: (a: any, b: any) => boolean, start: (e: PointerEvent, cat: any) => void }} drag 拖拽状态对象
   * @property {(id: number|string|null) => void} [onSelect]
   * @property {(payload: { name: string, parent_id: number|null }) => void} [onCreate]
   * @property {(id: number) => void} [onDelete]
   * @property {(payload: { id: number, name: string }) => void} [onRename]
   * @property {() => void} [onToggle] 切换整节折叠
   */
  let {
    categories = [],
    selected_id = null,
    collapsed = false,
    drag,
    onSelect,
    onCreate,
    onDelete,
    onRename,
    onToggle,
  } = $props();

  let expanded = $state(new Set());
  let cat_search = $state("");
  let show_new = $state(false);
  let new_name = $state("");
  let cat_placeholder = $state(DEFAULT_PLACEHOLDER);
  let deleting_id = $state(null);
  let editing_cat_id = $state(null);
  let editing_cat_name = $state("");
  let sub_create_parent_id = $state(null);
  let sub_create_name = $state("");
  let sub_placeholder = $state(DEFAULT_PLACEHOLDER);

  let flat_categories = $derived(flattenCategories(categories, { expanded }));
  let filtered_categories = $derived(
    cat_search.trim()
      ? flat_categories.filter((c) =>
          c.name.toLowerCase().includes(cat_search.trim().toLowerCase())
        )
      : flat_categories
  );

  function toggle(id) {
    const next = new Set(expanded);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    expanded = next;
  }

  function submit_category() {
    if (!new_name.trim()) {
      cat_placeholder = cyclePlaceholder(cat_placeholder);
      return;
    }
    const name = new_name.trim();
    if (name.includes("/")) {
      new_name = "";
      cat_placeholder = "分组名不能包含 /";
      return;
    }
    if (flat_categories.some((c) => c.name.toLowerCase() === name.toLowerCase())) {
      new_name = "";
      cat_placeholder = "已经有这个分组了";
      return;
    }
    onCreate?.({ name, parent_id: null });
    new_name = "";
    cat_placeholder = DEFAULT_PLACEHOLDER;
  }

  // 删除两阶段确认：第一次显示"再点一下"，第二次才真删
  function handle_delete_cat(e, id) {
    e.stopPropagation();
    if (deleting_id === id) {
      onDelete?.(id);
      deleting_id = null;
    } else {
      deleting_id = id;
    }
  }

  function reset_cat_delete() {
    deleting_id = null;
  }

  function start_rename_cat(e, cat) {
    e.stopPropagation();
    editing_cat_id = cat.id;
    editing_cat_name = cat.name;
  }

  function confirm_rename_cat(cat) {
    const name = editing_cat_name.trim();
    if (name && name !== cat.name) {
      onRename?.({ id: cat.id, name });
    }
    editing_cat_id = null;
    editing_cat_name = "";
  }

  function cancel_rename_cat() {
    editing_cat_id = null;
    editing_cat_name = "";
  }

  function start_sub_create(e, cat) {
    e.stopPropagation();
    sub_create_parent_id = cat.id;
    sub_create_name = "";
    sub_placeholder = DEFAULT_PLACEHOLDER;
    if (!expanded.has(cat.id)) expanded.add(cat.id);
  }

  function submit_sub_create() {
    if (!sub_create_name.trim()) {
      sub_placeholder = cyclePlaceholder(sub_placeholder);
      return;
    }
    const name = sub_create_name.trim();
    if (name.includes("/")) {
      sub_create_name = "";
      sub_placeholder = "分组名不能包含 /";
      return;
    }
    if (flat_categories.some((c) => c.name.toLowerCase() === name.toLowerCase())) {
      sub_create_name = "";
      sub_placeholder = "已经有这个分组了";
      return;
    }
    onCreate?.({ name, parent_id: sub_create_parent_id });
    sub_create_parent_id = null;
    sub_create_name = "";
    sub_placeholder = DEFAULT_PLACEHOLDER;
  }

  function cancel_sub_create() {
    sub_create_parent_id = null;
    sub_create_name = "";
    sub_placeholder = DEFAULT_PLACEHOLDER;
  }

  /** 给父组件用：判断当前是否在重命名某条（影响拖拽启动条件） */
  export function isEditing(id) {
    return editing_cat_id === id;
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
    分组管理
  </span>
  {#if !collapsed}
    <button
      class="section-action"
      onclick={(e) => { e.stopPropagation(); show_new = true; }}
      title="新建分组"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
        <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
    </button>
  {/if}
</div>

{#if !collapsed}
  {#if show_new}
    <form
      class="new-cat-form"
      onsubmit={(e) => { e.preventDefault(); submit_category(); }}
      onfocusout={(e) => {
        if (!e.currentTarget.contains(e.relatedTarget)) {
          show_new = false;
          new_name = "";
          cat_placeholder = DEFAULT_PLACEHOLDER;
        }
      }}
    >
      <input
        type="text"
        bind:value={new_name}
        placeholder={cat_placeholder}
        class="new-cat-input"
        onkeydown={(e) => {
          if (e.key === "Enter" && e.isComposing) return;
          if (e.key === "Escape") {
            show_new = false;
            new_name = "";
            cat_placeholder = DEFAULT_PLACEHOLDER;
          }
        }}
        autofocus
      />
    </form>
  {/if}
  {#if flat_categories.length > 10}
    <div class="section-search">
      <input
        type="text"
        bind:value={cat_search}
        placeholder="翻翻你的分组"
        class="section-search-input"
      />
    </div>
  {/if}
  <div class="category-list">
    <button
      class="nav-item cat-item"
      class:active={selected_id === "uncategorized"}
      style="padding-left: 8px"
      onclick={() => onSelect?.("uncategorized")}
    >
      <span class="cat-icon-area">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="3 2" class="cat-icon icon-static">
          <path d="M3 7V17C3 18.1046 3.89543 19 5 19H19C20.1046 19 21 18.1046 21 17V9C21 7.89543 20.1046 7 19 7H13L11 5H5C3.89543 5 3 5.89543 3 7Z"/>
        </svg>
      </span>
      <span class="cat-name">未分组</span>
    </button>
    {#if drag.dragId !== null}
      <div class="root-drop-zone" class:drop-target={drag.dropTargetId === "root"}>
        移到根级
      </div>
    {/if}
    {#each filtered_categories as cat (cat.id)}
      <button
        class="nav-item cat-item"
        class:active={selected_id === cat.id}
        class:expanded={cat.children?.length > 0 && expanded.has(cat.id)}
        class:dragging={drag.dragId === cat.id}
        class:drop-target={drag.dropTargetId === cat.id && drag.dragId !== cat.id && !drag.isDescendant(drag.dragId, cat.id)}
        style="--indent: {cat.depth * 12}px; padding-left: {8 + cat.depth * 12}px"
        data-cat-id={cat.id}
        onclick={() => {
          if (editing_cat_id !== cat.id) {
            if (cat.children?.length > 0) toggle(cat.id);
            onSelect?.(cat.id);
          }
        }}
        onmouseleave={() => reset_cat_delete()}
        onpointerdown={(e) => drag.start(e, cat)}
      >
        {#if cat.children?.length > 0}
          <span
            class="cat-child-indicator"
            style="transform: rotate({expanded.has(cat.id) ? 90 : 0}deg)"
          >
            <svg width="8" height="8" viewBox="0 0 8 8" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M2.5 1l3.5 3-3.5 3"/>
            </svg>
          </span>
        {/if}
        <span
          class="cat-icon-area"
          onclick={(e) => handle_delete_cat(e, cat.id)}
          onpointerdown={(e) => e.stopPropagation()}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="cat-icon icon-folder">
            <path d="M3 7V17C3 18.1046 3.89543 19 5 19H19C20.1046 19 21 18.1046 21 17V9C21 7.89543 20.1046 7 19 7H13L11 5H5C3.89543 5 3 5.89543 3 7Z"/>
          </svg>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="cat-icon icon-open-folder">
            <path d="M3 7h1.5l1.2-1.8c.2-.4.6-.7 1.1-.7H9l2 2h8c1.1 0 2 .9 2 2v8c0 1.1-.9 2-2 2H5c-1.1 0-2-.9-2-2V7z"/>
          </svg>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="cat-icon icon-delete">
            <path d="M3 6h18M8 6V4a2 2 0 012-2h4a2 2 0 012 2v2M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6"/>
          </svg>
        </span>
        {#if editing_cat_id === cat.id}
          <input
            type="text"
            class="rename-input"
            bind:value={editing_cat_name}
            onkeydown={(e) => {
              if (e.key === "Enter") { e.preventDefault(); confirm_rename_cat(cat); }
              if (e.key === "Escape") cancel_rename_cat();
            }}
            onblur={() => confirm_rename_cat(cat)}
            onclick={(e) => e.stopPropagation()}
            autofocus
          />
        {:else if deleting_id === cat.id}
          <span class="cat-delete-hint">再点一下就删除</span>
        {:else}
          <span class="cat-name">{cat.name}</span>
        {/if}
        <span
          class="cat-action-btn"
          onclick={(e) => start_rename_cat(e, cat)}
          onpointerdown={(e) => e.stopPropagation()}
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/>
          </svg>
        </span>
        <span
          class="cat-action-btn"
          onclick={(e) => start_sub_create(e, cat)}
          onpointerdown={(e) => e.stopPropagation()}
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
        </span>
      </button>
      {#if sub_create_parent_id === cat.id}
        <form
          class="new-cat-form"
          style="padding-left: {8 + (cat.depth + 1) * 12}px"
          onsubmit={(e) => { e.preventDefault(); submit_sub_create(); }}
          onfocusout={(e) => {
            if (!e.currentTarget.contains(e.relatedTarget)) cancel_sub_create();
          }}
        >
          <input
            type="text"
            bind:value={sub_create_name}
            placeholder={sub_placeholder}
            class="new-cat-input"
            onkeydown={(e) => { if (e.key === "Escape") cancel_sub_create(); }}
            autofocus
          />
        </form>
      {/if}
    {/each}
  </div>
{/if}

<style>
  /* —— 节折叠头（与 TagSection 共享视觉，所以这里也保留一份；
       未来若提到 app.css，TagSection 也可去除一份） —— */
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

  /* —— 分组列表特有 —— */

  .category-list {
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .cat-item {
    font-size: 13px;
    position: relative;
    touch-action: none;
  }

  .cat-item:hover {
    background: var(--bg-2);
  }

  .cat-item.active {
    background: var(--cat-soft);
    color: var(--cat-text);
    font-weight: 500;
  }

  .cat-item.expanded::before {
    content: "";
    position: absolute;
    left: var(--indent, 0px);
    top: 0;
    bottom: 0;
    width: 3px;
    background: var(--cat-text);
    border-radius: 0 2px 2px 0;
  }

  .cat-item.dragging {
    opacity: 0.4;
  }

  .cat-item.drop-target {
    background: var(--accent-soft);
    outline: 2px dashed var(--accent);
    outline-offset: -2px;
    border-radius: 4px;
  }

  .root-drop-zone {
    padding: 6px 10px;
    margin: 2px 0;
    border: 1px solid var(--border-2);
    border-radius: var(--radius-sm);
    text-align: center;
    color: var(--text-3);
    font-size: 12px;
    background: var(--bg-1);
  }

  .root-drop-zone.drop-target {
    border-color: transparent;
    color: var(--accent);
    background: var(--accent-soft);
    outline: 2px dashed var(--accent);
    outline-offset: -2px;
  }

  /* —— 图标区（hover 时切换为删除图标） —— */

  .cat-icon-area {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    margin: 0 1px;
    cursor: pointer;
  }

  .cat-icon {
    flex-shrink: 0;
    margin: 0 5px;
  }

  .cat-icon-area .cat-icon {
    margin: 0;
  }

  .icon-static {
    opacity: 0.5;
  }

  .icon-folder,
  .icon-open-folder {
    display: block;
  }

  .icon-open-folder {
    display: none;
  }

  .icon-delete {
    display: none;
    color: var(--danger);
  }

  .cat-item.expanded .icon-folder {
    display: none;
  }

  .cat-item.expanded .icon-open-folder {
    display: block;
    opacity: 0.7;
  }

  .nav-item:hover .cat-icon-area .icon-folder,
  .nav-item:hover .cat-icon-area .icon-open-folder {
    display: none;
  }

  .nav-item:hover .cat-icon-area .icon-delete {
    display: block;
  }

  /* —— action 按钮（重命名 / 新建子项），hover 时显现 —— */

  .cat-action-btn {
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

  .cat-item:hover .cat-action-btn {
    display: flex;
  }

  .cat-action-btn:hover {
    color: var(--text-1);
    background: var(--bg-hover);
  }

  .cat-action-btn + .cat-action-btn {
    margin-left: 2px;
  }

  /* —— 子标识、名字、删除提示、重命名输入框 —— */

  .cat-child-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 8px;
    height: 8px;
    flex-shrink: 0;
    margin-right: 4px;
    color: var(--text-3);
    transition: transform 0.15s ease;
  }

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
