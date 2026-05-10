<script>
  // 侧边栏：品牌头 + 顶部固定 nav（全部 / 特别关注 / 统计） +
  // CategorySection（分组节） + TagSection（标签节） + 底部 footer。
  //
  // 主组件只承担：折叠状态、拖拽对象、跨节联动（拖拽期间禁止 rename），
  // 各节内部状态由各 Section 自管。

  import { categoriesStore } from "../stores/index.js";
  import Brand from "./Sidebar/Brand.svelte";
  import CategorySection from "./Sidebar/CategorySection.svelte";
  import TagSection from "./Sidebar/TagSection.svelte";
  import { createCategoryDrag } from "../utils/categoryDragHandler.svelte.js";

  let {
    categories = [],
    tags = [],
    selected_id = null,
    selected_tag = null,
    onselect,
    onselect_tag,
    oncreate,
    ondelete_cat,
    onrename_cat,
    ontag_delete,
    onrename_tag,
    oncreate_tag,
    dark = false,
    ontoggle_dark,
    onexport,
    onimport,
    onsettings,
    importing = false,
    has_update = false,
    onupdate,
  } = $props();

  let collapsed = $state(new Set());
  /** @type {CategorySection | undefined} */
  let categorySectionRef;

  // 分组拖拽：通过 ref 询问 CategorySection 当前是否在重命名某条
  const drag = createCategoryDrag({
    getCategories: () => categories,
    onDropToParent: (id, parent_id) => categoriesStore.update({ id, parent_id }),
    onDropToRoot: (id) => categoriesStore.update({ id, unset_parent: true }),
    canStartDrag: (cat) => !categorySectionRef?.isEditing(cat.id),
  });

  function toggle_section(key) {
    const next = new Set(collapsed);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    collapsed = next;
  }
</script>

<aside class="sidebar">
  <Brand hasUpdate={has_update} onSettings={onsettings} onUpdate={onupdate} />

  <nav class="sidebar-nav">
    <button
      class="nav-item"
      class:active={selected_id === null && !selected_tag}
      onclick={() => onselect?.(null)}
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <rect x="3" y="3" width="7" height="7" rx="1"/>
        <rect x="14" y="3" width="7" height="7" rx="1"/>
        <rect x="3" y="14" width="7" height="7" rx="1"/>
        <rect x="14" y="14" width="7" height="7" rx="1"/>
      </svg>
      <span>全部链接</span>
    </button>

    <button
      class="nav-item"
      class:active={selected_id === 'favorite'}
      onclick={() => onselect?.('favorite')}
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"/>
      </svg>
      <span>特别关注</span>
    </button>

    <button
      class="nav-item"
      class:active={selected_id === 'stats'}
      onclick={() => onselect?.('stats')}
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <line x1="18" y1="20" x2="18" y2="10"/><line x1="12" y1="20" x2="12" y2="4"/><line x1="6" y1="20" x2="6" y2="14"/>
      </svg>
      <span>统计</span>
    </button>
  </nav>

  <div class="sidebar-section">
    <CategorySection
      bind:this={categorySectionRef}
      {categories}
      {selected_id}
      collapsed={collapsed.has('categories')}
      {drag}
      onSelect={onselect}
      onCreate={oncreate}
      onDelete={ondelete_cat}
      onRename={onrename_cat}
      onToggle={() => toggle_section('categories')}
    />

    <TagSection
      {tags}
      {selected_tag}
      collapsed={collapsed.has('tags')}
      onSelect={onselect_tag}
      onCreate={oncreate_tag}
      onDelete={ontag_delete}
      onRename={onrename_tag}
      onToggle={() => toggle_section('tags')}
    />
  </div>

  <div class="sidebar-footer">
    <button class="footer-btn" onclick={ontoggle_dark}>
      {#if dark}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="4"/>
          <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"/>
        </svg>
      {:else}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z"/>
        </svg>
      {/if}
    </button>

    <button class="footer-btn" onclick={onexport}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M17 8l-5-5-5 5M12 3v12"/>
      </svg>
    </button>

    <button class="footer-btn" onclick={onimport} disabled={importing}>
      {#if importing}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" class="spin"><path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"/></svg>
      {:else}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3"/>
        </svg>
      {/if}
    </button>
  </div>
</aside>

<style>
  .sidebar {
    width: 220px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: var(--bg-1);
    border-right: 1px solid var(--border-0);
    overflow: clip;
  }

  /* —— 顶部 nav（与 CategorySection / TagSection 共享 .nav-item，
       基础样式见 app.css；以下仅是顶部 nav 的容器与 svg 调整） —— */

  .sidebar-nav {
    padding: 0 8px;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  /* sidebar-nav 下的 nav-item 自带 svg；缩进与 CategorySection 不同 */
  .sidebar-nav .nav-item :global(svg) {
    flex-shrink: 0;
    margin: 0 5px;
  }

  /* —— 主滚动容器（包裹两个 Section） —— */
  .sidebar-section {
    flex: 1;
    padding: 8px 8px 0;
    overflow-y: auto;
  }

  /* —— 暗色模式下分组/标签的 hover/active：
       两个 Section 内部的 .cat-item / .tag-item 用 :global(.dark) 选择器
       覆盖，但 :global(.dark) 在子组件 scoped style 中不生效（class 不匹配
       到子组件根的 .cat-item），所以这里在父组件用 :global(...) 定义 —— */
  :global(.dark) .cat-item:hover {
    background: var(--bg-2);
  }
  :global(.dark) .cat-item.active {
    background: var(--cat-soft);
    color: var(--cat-text);
  }
  :global(.dark) .cat-item.expanded::before {
    background: var(--cat-text);
  }
  :global(.dark) .tag-item:hover {
    background: var(--bg-2);
  }
  :global(.dark) .tag-item.active {
    background: var(--accent-soft);
    color: var(--accent-text);
  }

  /* —— 底部 footer —— */
  .sidebar-footer {
    padding: 8px 12px;
    background: var(--bg-2);
    display: flex;
    gap: 4px;
  }

  .footer-btn {
    width: 32px;
    height: 32px;
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

  .footer-btn:hover {
    background: var(--border-1);
    color: var(--text-1);
  }

  .footer-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .spin {
    animation: spin 1s linear infinite;
  }

  /* —— 分组拖拽时附着到 body 的浮层（categoryDragHandler 创建） —— */
  :global(.drag-ghost) {
    position: fixed;
    pointer-events: none;
    z-index: 9999;
    background: var(--bg-2);
    border: 1px solid var(--border-1);
    border-radius: var(--radius-sm);
    padding: 6px 12px;
    font-size: 13px;
    color: var(--text-0);
    opacity: 0.8;
    box-shadow: var(--shadow-drag);
    white-space: nowrap;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
