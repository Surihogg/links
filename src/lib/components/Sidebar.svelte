<script>
  let { categories = [], tags = [], selected_id = null, selected_tag = null, onselect, onselect_tag, oncreate, ondelete_cat, ontag_delete, oncreate_tag, dark = false, ontoggle_dark, onexport, onimport, importing = false } = $props();
  let expanded = $state(new Set());
  let show_new = $state(false);
  let new_name = $state("");
  let new_parent_id = $state(null);
  let collapsed = $state(new Set());
  let cat_search = $state("");
  let tag_search = $state("");
  let deleting_id = $state(null);
  let deleting_tag_id = $state(null);
  let show_new_tag = $state(false);
  let new_tag_name = $state("");

  function toggle_section(key) {
    const next = new Set(collapsed);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    collapsed = next;
  }

  function toggle(id) {
    if (expanded.has(id)) expanded.delete(id);
    else expanded.add(id);
  }

  function submit_category() {
    if (!new_name.trim()) return;
    oncreate?.({ name: new_name.trim(), parent_id: new_parent_id });
    new_name = "";
    show_new = false;
  }

  function flatten_categories(cats, depth = 0) {
    const result = [];
    for (const cat of cats) {
      result.push({ ...cat, depth });
      if (cat.children?.length > 0) {
        result.push(...flatten_categories(cat.children, depth + 1));
      }
    }
    return result;
  }

  let flat_categories = $derived(flatten_categories(categories));
  let filtered_categories = $derived(
    cat_search.trim()
      ? flat_categories.filter(c => c.name.toLowerCase().includes(cat_search.trim().toLowerCase()))
      : flat_categories
  );
  let filtered_tags = $derived(
    tag_search.trim()
      ? tags.filter(t => t.name.toLowerCase().includes(tag_search.trim().toLowerCase()))
      : tags
  );

  async function handle_delete_cat(e, id) {
    e.stopPropagation();
    if (deleting_id === id) {
      ondelete_cat?.(id);
      deleting_id = null;
    } else {
      deleting_id = id;
    }
  }

  async function handle_delete_tag(e, id) {
    e.stopPropagation();
    if (deleting_tag_id === id) {
      ontag_delete?.(id);
      deleting_tag_id = null;
    } else {
      deleting_tag_id = id;
    }
  }

  function submit_tag() {
    if (!new_tag_name.trim()) return;
    oncreate_tag?.(new_tag_name.trim());
    new_tag_name = "";
    show_new_tag = false;
  }
</script>

<aside class="sidebar">
  <div class="sidebar-brand">
    <span class="brand-icon">◈</span>
    <span class="brand-text">Links</span>
  </div>

  <nav class="sidebar-nav">
    <button
      class="nav-item"
      class:active={selected_id === null}
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
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round">
        <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01z"/>
      </svg>
      <span>特别关注</span>
    </button>
  </nav>

  <div class="sidebar-section">
    <div class="section-header" onclick={() => toggle_section('categories')}>
      <span class="section-label" style="cursor:pointer;">
        <svg class="chevron" width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" style="transform: rotate({collapsed.has('categories') ? 0 : 90}deg); transition: transform var(--transition);">
          <path d="M3 1l4 4-4 4"/>
        </svg>
        分组管理
      </span>
      {#if !collapsed.has('categories')}
        <button class="section-action" onclick={(e) => { e.stopPropagation(); show_new = true; }} title="新建分组">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
        </button>
      {/if}
    </div>

    {#if !collapsed.has('categories')}
    {#if flat_categories.length > 10}
    <div class="section-search">
      <input type="text" bind:value={cat_search} placeholder="搜索分组..." class="section-search-input" />
    </div>
    {/if}
    <div class="category-list">
      {#each filtered_categories as cat (cat.id)}
        <button
          class="nav-item cat-item"
          class:active={selected_id === cat.id}
          style="padding-left: {12 + cat.depth * 16}px"
          onclick={() => onselect?.(cat.id)}
        >
          {#if cat.children?.length > 0}
            <span class="cat-toggle" onclick={(e) => { e.stopPropagation(); toggle(cat.id); }}>
              <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" style="transform: rotate({expanded.has(cat.id) ? 90 : 0}deg); transition: transform var(--transition);">
                <path d="M3 1l4 4-4 4"/>
              </svg>
            </span>
          {:else}
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="cat-icon"><path d="M3 7V17C3 18.1046 3.89543 19 5 19H19C20.1046 19 21 18.1046 21 17V9C21 7.89543 20.1046 7 19 7H13L11 5H5C3.89543 5 3 5.89543 3 7Z"/></svg>
          {/if}
          <span class="cat-name">{cat.name}</span>
          {#if deleting_id === cat.id}
            <span class="cat-delete-hint">确认?</span>
          {/if}
          <span class="cat-delete-btn" onclick={(e) => handle_delete_cat(e, cat.id)} title="删除分组">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
              <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </span>
        </button>
      {/each}
    </div>

    {#if show_new}
      <form class="new-cat-form" onsubmit={(e) => { e.preventDefault(); submit_category(); }}>
        <input
          type="text"
          bind:value={new_name}
          placeholder="分组名称"
          class="new-cat-input"
          autofocus
        />
        <div class="new-cat-actions">
          <button type="submit" class="new-cat-btn primary">确定</button>
          <button type="button" class="new-cat-btn" onclick={() => { show_new = false; new_name = ""; }}>取消</button>
        </div>
      </form>
    {/if}
    {/if}

    <div class="section-header" onclick={() => toggle_section('tags')}>
      <span class="section-label" style="cursor:pointer;">
        <svg class="chevron" width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" style="transform: rotate({collapsed.has('tags') ? 0 : 90}deg); transition: transform var(--transition);">
          <path d="M3 1l4 4-4 4"/>
        </svg>
        标签管理
      </span>
      {#if !collapsed.has('tags')}
        <button class="section-action" onclick={(e) => { e.stopPropagation(); show_new_tag = true; }} title="新建标签">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
        </button>
      {/if}
    </div>
    {#if !collapsed.has('tags')}
    {#if show_new_tag}
      <form class="new-cat-form" onsubmit={(e) => { e.preventDefault(); submit_tag(); }}>
        <input
          type="text"
          bind:value={new_tag_name}
          placeholder="标签名称"
          class="new-cat-input"
          autofocus
        />
        <div class="new-cat-actions">
          <button type="submit" class="new-cat-btn primary">确定</button>
          <button type="button" class="new-cat-btn" onclick={() => { show_new_tag = false; new_tag_name = ""; }}>取消</button>
        </div>
      </form>
    {/if}
    {#if tags.length > 10}
    <div class="section-search">
      <input type="text" bind:value={tag_search} placeholder="搜索标签..." class="section-search-input" />
    </div>
    {/if}
    <div class="tag-list">
      {#each filtered_tags as tag (tag.id)}
        <button
          class="nav-item tag-item"
          class:active={selected_tag === tag.name}
          onclick={() => onselect_tag?.(tag.name)}
        >
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="tag-icon"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>
          <span class="cat-name">{tag.name}</span>
          {#if deleting_tag_id === tag.id}
            <span class="cat-delete-hint">确认?</span>
          {/if}
          <span class="tag-delete-btn" onclick={(e) => handle_delete_tag(e, tag.id)} title="删除标签">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
              <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </span>
        </button>
      {/each}
    </div>
    {/if}
  </div>

  <div class="sidebar-footer">
    <button class="footer-btn" onclick={ontoggle_dark} title={dark ? '切换亮色' : '切换暗色'}>
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

    <button class="footer-btn" onclick={onexport} title="导出">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M17 8l-5-5-5 5M12 3v12"/>
      </svg>
    </button>

    <button class="footer-btn" onclick={onimport} title="导入书签" disabled={importing}>
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
    overflow: hidden;
  }

  .sidebar-brand {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 16px 16px 12px;
  }

  .brand-icon {
    font-size: 18px;
    color: var(--accent);
  }

  .brand-text {
    font-size: 15px;
    font-weight: 700;
    color: var(--text-0);
    letter-spacing: -0.3px;
  }

  .sidebar-nav {
    padding: 0 8px;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 10px;
    border: none;
    background: none;
    color: var(--text-2);
    font-size: 13px;
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: all var(--transition);
    text-align: left;
  }

  .nav-item:hover {
    background: var(--bg-hover);
    color: var(--text-1);
  }

  .nav-item.active {
    background: var(--accent-soft);
    color: var(--accent);
  }

  .cat-item.active {
    background: var(--cat-soft);
    color: var(--cat-text);
  }

  .cat-icon, .tag-icon {
    flex-shrink: 0;
    margin: 0 5px;
  }

  .sidebar-section {
    flex: 1;
    padding: 8px 8px 0;
    overflow-y: auto;
  }

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

  .category-list {
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .cat-item {
    font-size: 13px;
    position: relative;
  }

  .cat-item:hover {
    background: var(--bg-2);
  }

  .cat-item.active {
    background: var(--cat-soft);
    font-weight: 500;
  }

  :global(.dark) .cat-item:hover {
    background: var(--bg-2);
  }

  :global(.dark) .cat-item.active {
    background: #166534;
  }

  .tag-item {
    font-size: 12px;
    position: relative;
  }

  .tag-item:hover {
    background: var(--bg-2);
  }

  .tag-item.active {
    background: var(--accent-soft);
    font-weight: 500;
  }

  :global(.dark) .tag-item:hover {
    background: var(--bg-2);
  }

  :global(.dark) .tag-item.active {
    background: #1e3a8a;
  }

  .cat-delete-btn {
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

  .cat-item:hover .cat-delete-btn {
    display: flex;
  }

  .cat-delete-btn:hover {
    color: var(--danger);
    background: var(--danger-soft);
  }

  .cat-delete-hint {
    font-size: 10px;
    color: var(--danger);
    margin-left: auto;
    margin-right: 2px;
    flex-shrink: 0;
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
    box-shadow: 0 0 0 2px var(--accent-soft);
  }

  .cat-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    flex-shrink: 0;
    color: var(--text-3);
  }

  .cat-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .new-cat-form {
    padding: 8px 4px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .new-cat-input {
    width: 100%;
    padding: 5px 8px;
    border: 1px solid var(--border-1);
    border-radius: var(--radius-sm);
    background: var(--bg-0);
    color: var(--text-0);
    font-size: 12px;
    outline: none;
    transition: border-color var(--transition);
  }

  .new-cat-input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .new-cat-actions {
    display: flex;
    gap: 4px;
  }

  .new-cat-btn {
    padding: 3px 10px;
    border: none;
    border-radius: var(--radius-sm);
    font-size: 11px;
    cursor: pointer;
    color: var(--text-2);
    background: var(--bg-2);
    transition: all var(--transition);
  }

  .new-cat-btn.primary {
    background: var(--accent);
    color: white;
  }

  .new-cat-btn.primary:hover { background: var(--accent-hover); }
  .new-cat-btn:not(.primary):hover { background: var(--border-1); }

  .tag-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .tag-delete-btn {
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

  .tag-item:hover .tag-delete-btn {
    display: flex;
  }

  .tag-delete-btn:hover {
    color: var(--danger);
    background: var(--danger-soft);
  }

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
    background: var(--bg-2);
    color: var(--text-1);
  }

  .footer-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .spin {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
