<script>
  import { categoriesStore } from '../stores/index.js';
  import { getVersion } from '@tauri-apps/api/app';
  let version = $state('');
  getVersion().then(v => version = v);
  let { categories = [], tags = [], selected_id = null, selected_tag = null, onselect, onselect_tag, oncreate, ondelete_cat, onrename_cat, ontag_delete, onrename_tag, oncreate_tag, dark = false, ontoggle_dark, onexport, onimport, onsettings, importing = false, has_update = false, onupdate } = $props();
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
  let tag_placeholder = $state("给我一点输入");
  let cat_placeholder = $state("给我一点输入");

  let editing_cat_id = $state(null);
  let editing_cat_name = $state("");
  let editing_tag_id = $state(null);
  let editing_tag_name = $state("");
  let sub_create_parent_id = $state(null);
  let sub_create_name = $state("");
  let sub_placeholder = $state("给我一点输入");

  // 拖拽状态
  let drag_id = $state(null);
  let drop_target_id = $state(null);
  let ghost_el = $state(null);
  let drag_start_pos = $state({ x: 0, y: 0 });
  let is_dragging = $state(false);
  let pending_drag_id = $state(null);

  // Pointer Events 拖拽实现
  function handle_pointer_down(e, cat) {
    // 只在鼠标左键时启动拖拽
    if (e.button !== 0) return;
    // 如果正在编辑，不启动拖拽
    if (editing_cat_id === cat.id) return;

    // 延迟设置 drag_id，避免点击时立即产生拖拽视觉效果
    pending_drag_id = cat.id;
    drag_start_pos = { x: e.clientX, y: e.clientY };
    is_dragging = false;

    // 添加全局事件监听
    window.addEventListener('pointermove', handle_pointer_move);
    window.addEventListener('pointerup', handle_pointer_up);
    window.addEventListener('pointercancel', handle_pointer_up);
  }

  function handle_pointer_move(e) {
    if (pending_drag_id === null) return;

    const dx = e.clientX - drag_start_pos.x;
    const dy = e.clientY - drag_start_pos.y;

    // 移动超过 5px 才开始拖拽
    if (!is_dragging && (Math.abs(dx) > 5 || Math.abs(dy) > 5)) {
      is_dragging = true;
      drag_id = pending_drag_id;
      create_ghost_element();
    }

    if (is_dragging) {
      e.preventDefault();
      update_ghost_position(e.clientX, e.clientY);
      update_drop_target(e.clientX, e.clientY);
    }
  }

  function handle_pointer_up(e) {
    if (is_dragging && drag_id !== null) {
      // 在释放前再更新一次 drop target，确保坐标准确
      update_drop_target(e.clientX, e.clientY);
      e.preventDefault();
      execute_drop();
    }

    // 清理
    remove_ghost_element();
    window.removeEventListener('pointermove', handle_pointer_move);
    window.removeEventListener('pointerup', handle_pointer_up);
    window.removeEventListener('pointercancel', handle_pointer_up);

    drag_id = null;
    pending_drag_id = null;
    drop_target_id = null;
    is_dragging = false;
  }

  function create_ghost_element() {
    const cat = flat_categories.find(c => c.id === drag_id);
    if (!cat) return;

    ghost_el = document.createElement('div');
    ghost_el.className = 'drag-ghost';
    ghost_el.textContent = cat.name;
    ghost_el.style.cssText = `
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
      box-shadow: 0 4px 12px rgba(0,0,0,0.15);
      white-space: nowrap;
      max-width: 200px;
      overflow: hidden;
      text-overflow: ellipsis;
    `;
    document.body.appendChild(ghost_el);
  }

  function update_ghost_position(x, y) {
    if (ghost_el) {
      ghost_el.style.left = (x + 10) + 'px';
      ghost_el.style.top = (y + 10) + 'px';
    }
  }

  function remove_ghost_element() {
    if (ghost_el) {
      ghost_el.remove();
      ghost_el = null;
    }
  }

  function update_drop_target(x, y) {
    // 隐藏 ghost 以便获取鼠标下的元素
    if (ghost_el) ghost_el.style.display = 'none';

    const elem = document.elementFromPoint(x, y);
    const cat_item = elem?.closest('.cat-item[data-cat-id]');

    if (ghost_el) ghost_el.style.display = '';

    if (cat_item) {
      const target_id = parseInt(cat_item.dataset.catId);
      // 不能拖到自己，也不能拖到后代
      if (target_id !== drag_id && !is_descendant(drag_id, target_id)) {
        drop_target_id = target_id;
        return;
      }
    }

    // 检查是否在"移到根级"区域
    const root_zone = elem?.closest('.root-drop-zone');
    if (root_zone) {
      drop_target_id = 'root';
      return;
    }

    drop_target_id = null;
  }

  function execute_drop() {
    if (drop_target_id === 'root') {
      categoriesStore.update({ id: drag_id, unset_parent: true });
    } else if (drop_target_id !== null && drop_target_id !== drag_id) {
      categoriesStore.update({ id: drag_id, parent_id: drop_target_id });
    }
  }

  // 检查 target_id 是否是 cat_id 的后代（防止循环依赖）
  function is_descendant(cat_id, target_id) {
    if (!cat_id || !target_id) return false;
    function find_in_tree(nodes) {
      for (const node of nodes) {
        if (node.id === cat_id) {
          return check_descendant(node.children || [], target_id);
        }
        if (node.children?.length > 0) {
          const found = find_in_tree(node.children);
          if (found) return found;
        }
      }
      return false;
    }
    function check_descendant(nodes, tid) {
      for (const node of nodes) {
        if (node.id === tid) return true;
        if (node.children?.length > 0) {
          if (check_descendant(node.children, tid)) return true;
        }
      }
      return false;
    }
    return find_in_tree(categories);
  }

  function toggle_section(key) {
    const next = new Set(collapsed);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    collapsed = next;
  }

  function toggle(id) {
    const next = new Set(expanded);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    expanded = next;
  }

  function submit_category() {
    if (!new_name.trim()) {
      cat_placeholder = cat_placeholder === "给我一点输入" ? "你是认真的吗？" : "给我一点输入";
      return;
    }
    const name = new_name.trim();
    if (name.includes('/')) {
      new_name = "";
      cat_placeholder = "分组名不能包含 /";
      return;
    }
    if (flat_categories.some(c => c.name.toLowerCase() === name.toLowerCase())) {
      new_name = "";
      cat_placeholder = "已经有这个分组了";
      return;
    }
    oncreate?.({ name, parent_id: new_parent_id });
    new_name = "";
    cat_placeholder = "给我一点输入";
  }

  function flatten_categories(cats, expanded_set, depth = 0) {
    const result = [];
    for (const cat of cats) {
      result.push({ ...cat, depth });
      // 仅展开的节点才渲染子节点
      if (cat.children?.length > 0 && expanded_set.has(cat.id)) {
        result.push(...flatten_categories(cat.children, expanded_set, depth + 1));
      }
    }
    return result;
  }

  let flat_categories = $derived(flatten_categories(categories, expanded));
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

  function reset_cat_delete() {
    deleting_id = null;
  }

  function reset_tag_delete() {
    deleting_tag_id = null;
  }

  function start_rename_cat(e, cat) {
    e.stopPropagation();
    editing_cat_id = cat.id;
    editing_cat_name = cat.name;
  }

  function confirm_rename_cat(cat) {
    const name = editing_cat_name.trim();
    if (name && name !== cat.name) {
      onrename_cat?.({ id: cat.id, name });
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
    sub_placeholder = "给我一点输入";
    if (!expanded.has(cat.id)) expanded.add(cat.id);
  }

  function submit_sub_create() {
    if (!sub_create_name.trim()) {
      sub_placeholder = sub_placeholder === "给我一点输入" ? "你是认真的吗？" : "给我一点输入";
      return;
    }
    const name = sub_create_name.trim();
    if (name.includes('/')) {
      sub_create_name = "";
      sub_placeholder = "分组名不能包含 /";
      return;
    }
    if (flat_categories.some(c => c.name.toLowerCase() === name.toLowerCase())) {
      sub_create_name = "";
      sub_placeholder = "已经有这个分组了";
      return;
    }
    oncreate?.({ name, parent_id: sub_create_parent_id });
    sub_create_parent_id = null;
    sub_create_name = "";
    sub_placeholder = "给我一点输入";
  }

  function cancel_sub_create() {
    sub_create_parent_id = null;
    sub_create_name = "";
    sub_placeholder = "给我一点输入";
  }

  function start_rename_tag(e, tag) {
    e.stopPropagation();
    editing_tag_id = tag.id;
    editing_tag_name = tag.name;
  }

  function confirm_rename_tag(tag) {
    const name = editing_tag_name.trim();
    if (name && name !== tag.name) {
      onrename_tag?.({ id: tag.id, name });
    }
    editing_tag_id = null;
    editing_tag_name = "";
  }

  function cancel_rename_tag() {
    editing_tag_id = null;
    editing_tag_name = "";
  }

  function submit_tag() {
    if (!new_tag_name.trim()) {
      tag_placeholder = tag_placeholder === "给我一点输入" ? "你是认真的吗？" : "给我一点输入";
      return;
    }
    const name = new_tag_name.trim();
    if (tags.some(t => t.name.toLowerCase() === name.toLowerCase())) {
      new_tag_name = "";
      tag_placeholder = "已经有这个标签了";
      return;
    }
    oncreate_tag?.(name);
    new_tag_name = "";
    tag_placeholder = "给我一点输入";
  }
</script>

<aside class="sidebar">
  <div class="sidebar-brand">
    <span class="brand-icon">◈</span>
    <span class="brand-text">Links</span>
    {#if version}
      <span class="brand-version">v{version}</span>
    {/if}
    {#if has_update}
      <button class="brand-update" onclick={onupdate} data-tooltip="有新版本可用" aria-label="有新版本可用">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 11-7.778 7.778 5.5 5.5 0 017.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/>
        </svg>
      </button>
    {/if}
    <button class="brand-settings" onclick={onsettings}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="3"/>
        <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/>
      </svg>
    </button>
  </div>

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
    {#if show_new}
      <form class="new-cat-form" onsubmit={(e) => { e.preventDefault(); submit_category(); }} onfocusout={(e) => { if (!e.currentTarget.contains(e.relatedTarget)) { show_new = false; new_name = ""; cat_placeholder = "给我一点输入"; } }}>
        <input
          type="text"
          bind:value={new_name}
          placeholder={cat_placeholder}
          class="new-cat-input"
          onkeydown={(e) => { if (e.key === 'Escape') { show_new = false; new_name = ""; cat_placeholder = "给我一点输入"; } }}
          autofocus
        />
      </form>
    {/if}
    {#if flat_categories.length > 10}
    <div class="section-search">
      <input type="text" bind:value={cat_search} placeholder="翻翻你的分组" class="section-search-input" />
    </div>
    {/if}
    <div class="category-list">
      <button
        class="nav-item cat-item"
        class:active={selected_id === 'uncategorized'}
        style="padding-left: 8px"
        onclick={() => onselect?.('uncategorized')}
      >
        <span class="cat-toggle-spacer"></span>
        <span class="cat-icon-area">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="3 2" class="cat-icon icon-static"><path d="M3 7V17C3 18.1046 3.89543 19 5 19H19C20.1046 19 21 18.1046 21 17V9C21 7.89543 20.1046 7 19 7H13L11 5H5C3.89543 5 3 5.89543 3 7Z"/></svg>
        </span>
        <span class="cat-name">未分组</span>
      </button>
      {#if drag_id !== null}
        <div
          class="root-drop-zone"
          class:drop-target={drop_target_id === 'root'}
        >
          移到根级
        </div>
      {/if}
      {#each filtered_categories as cat (cat.id)}
        <button
          class="nav-item cat-item"
          class:active={selected_id === cat.id}
          class:dragging={drag_id === cat.id}
          class:drop-target={drop_target_id === cat.id && drag_id !== cat.id && !is_descendant(drag_id, cat.id)}
          style="padding-left: {8 + cat.depth * 12}px"
          data-cat-id={cat.id}
          onclick={() => { if (editing_cat_id !== cat.id) onselect?.(cat.id); }}
          onmouseleave={() => { reset_cat_delete(); }}
          onpointerdown={(e) => handle_pointer_down(e, cat)}
        >
          {#if cat.children?.length > 0}
            <span class="cat-toggle" onclick={(e) => { e.stopPropagation(); toggle(cat.id); }} onpointerdown={(e) => e.stopPropagation()}>
              <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" style="transform: rotate({expanded.has(cat.id) ? 90 : 0}deg); transition: transform var(--transition);">
                <path d="M3 1l4 4-4 4"/>
              </svg>
            </span>
          {:else}
            <span class="cat-toggle-spacer"></span>
          {/if}
          <span class="cat-icon-area"
            onclick={(e) => handle_delete_cat(e, cat.id)}
            onpointerdown={(e) => e.stopPropagation()}
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="cat-icon icon-folder"><path d="M3 7V17C3 18.1046 3.89543 19 5 19H19C20.1046 19 21 18.1046 21 17V9C21 7.89543 20.1046 7 19 7H13L11 5H5C3.89543 5 3 5.89543 3 7Z"/></svg>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="cat-icon icon-delete"><path d="M3 6h18M8 6V4a2 2 0 012-2h4a2 2 0 012 2v2M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6"/></svg>
          </span>
          {#if editing_cat_id === cat.id}
            <input
              type="text"
              class="rename-input"
              bind:value={editing_cat_name}
              onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); confirm_rename_cat(cat); } if (e.key === 'Escape') cancel_rename_cat(); }}
              onblur={() => confirm_rename_cat(cat)}
              onclick={(e) => e.stopPropagation()}
              autofocus
            />
          {:else if deleting_id === cat.id}
            <span class="cat-delete-hint">再点一下就删除</span>
          {:else}
            <span class="cat-name">{cat.name}</span>
          {/if}
            <span class="cat-action-btn" onclick={(e) => start_rename_cat(e, cat)} onpointerdown={(e) => e.stopPropagation()}>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/>
              </svg>
            </span>
            <span class="cat-action-btn" onclick={(e) => start_sub_create(e, cat)} onpointerdown={(e) => e.stopPropagation()}>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
              </svg>
            </span>
          </button>
          {#if sub_create_parent_id === cat.id}
            <form class="new-cat-form" style="padding-left: {8 + (cat.depth + 1) * 12}px" onsubmit={(e) => { e.preventDefault(); submit_sub_create(); }} onfocusout={(e) => { if (!e.currentTarget.contains(e.relatedTarget)) cancel_sub_create(); }}>
              <input
                type="text"
                bind:value={sub_create_name}
                placeholder={sub_placeholder}
                class="new-cat-input"
                onkeydown={(e) => { if (e.key === 'Escape') cancel_sub_create(); }}
                autofocus
              />
            </form>
          {/if}
        {/each}
    </div>
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
      <form class="new-cat-form" onsubmit={(e) => { e.preventDefault(); submit_tag(); }} onfocusout={(e) => { if (!e.currentTarget.contains(e.relatedTarget)) { show_new_tag = false; new_tag_name = ""; tag_placeholder = "给我一点输入"; } }}>
        <input
          type="text"
          bind:value={new_tag_name}
          placeholder={tag_placeholder}
          class="new-cat-input"
          onkeydown={(e) => { if (e.key === 'Escape') { show_new_tag = false; new_tag_name = ""; tag_placeholder = "给我一点输入"; } }}
          autofocus
        />
      </form>
    {/if}
    {#if tags.length > 10}
    <div class="section-search">
      <input type="text" bind:value={tag_search} placeholder="找找你的标签" class="section-search-input" />
    </div>
    {/if}
    <div class="tag-list">
      <button
        class="nav-item tag-item"
        class:active={selected_tag === '__untagged__'}
        style="padding-left: 12px"
        onclick={() => onselect_tag?.('__untagged__')}
      >
        <span class="tag-icon-area">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="3 2" class="tag-icon icon-static"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>
        </span>
        <span class="cat-name">无标签</span>
      </button>
      {#each filtered_tags as tag (tag.id)}
        <button
          class="nav-item tag-item"
          class:active={selected_tag === tag.name}
          style="padding-left: 12px"
          onclick={() => { if (editing_tag_id !== tag.id) onselect_tag?.(tag.name); }}
          onmouseleave={() => { reset_tag_delete(); }}
        >
          <span class="tag-icon-area"
            onclick={(e) => handle_delete_tag(e, tag.id)}
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="tag-icon icon-tag"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="tag-icon icon-delete"><path d="M3 6h18M8 6V4a2 2 0 012-2h4a2 2 0 012 2v2M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6"/></svg>
          </span>
          {#if editing_tag_id === tag.id}
            <input
              type="text"
              class="rename-input"
              bind:value={editing_tag_name}
              onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); confirm_rename_tag(tag); } if (e.key === 'Escape') cancel_rename_tag(); }}
              onblur={() => confirm_rename_tag(tag)}
              onclick={(e) => e.stopPropagation()}
              autofocus
            />
          {:else if deleting_tag_id === tag.id}
            <span class="cat-delete-hint">再点一下就删除</span>
          {:else}
            <span class="cat-name">{tag.name}</span>
          {/if}
           <span class="tag-action-btn" onclick={(e) => start_rename_tag(e, tag)}>
             <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
               <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/>
             </svg>
           </span>
         </button>
       {/each}
    </div>
    {/if}
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
    font-size: 18px;
    font-weight: 700;
    color: var(--text-0);
    letter-spacing: -0.5px;
    font-family: "Georgia", "Times New Roman", serif;
  }

  .brand-version {
    font-size: 11px;
    color: var(--text-3);
    font-weight: 400;
    margin-left: 2px;
    align-self: center;
  }

  .brand-settings {
    margin-left: auto;
    width: 24px;
    height: 24px;
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

  .brand-settings:hover {
    color: var(--text-1);
    background: var(--bg-hover);
  }

  .brand-update {
    position: relative;
    width: 24px;
    height: 24px;
    border: none;
    background: none;
    color: var(--accent);
    cursor: pointer;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition);
    animation: pulse-glow 2s ease-in-out infinite;
  }

  .brand-update:hover {
    background: var(--accent-soft);
  }

  .brand-update[data-tooltip]:hover::after {
    content: attr(data-tooltip);
    position: absolute;
    top: 100%;
    left: 50%;
    transform: translateX(-50%);
    margin-top: 4px;
    padding: 4px 8px;
    background: var(--text-0);
    color: var(--bg-0);
    font-size: 11px;
    border-radius: 4px;
    white-space: nowrap;
    pointer-events: none;
    z-index: 100;
  }

  @keyframes pulse-glow {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
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
    gap: 2px;
    width: 100%;
    padding: 6px 6px;
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

  .cat-icon-area, .tag-icon-area {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    margin: 0 1px;
    cursor: pointer;
  }

  .cat-icon-area .cat-icon,
  .tag-icon-area .tag-icon {
    margin: 0;
  }

  .icon-static {
    opacity: 0.5;
  }

  .icon-delete {
    display: none;
    color: var(--danger);
  }

  .icon-folder, .icon-tag {
    display: block;
  }

  .nav-item:hover .cat-icon-area .icon-folder,
  .nav-item:hover .tag-icon-area .icon-tag {
    display: none;
  }

  .nav-item:hover .cat-icon-area .icon-delete,
  .nav-item:hover .tag-icon-area .icon-delete {
    display: block;
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
    touch-action: none;
  }

  .cat-item:hover {
    background: var(--bg-2);
  }

  .cat-item.active {
    background: var(--cat-soft);
    font-weight: 500;
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

  :global(.dark) .cat-item:hover {
    background: var(--bg-2);
  }

  :global(.dark) .cat-item.active {
    background: var(--cat-soft);
    color: var(--cat-text);
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
    font-weight: 500;
  }

  :global(.dark) .tag-item:hover {
    background: var(--bg-2);
  }

  :global(.dark) .tag-item.active {
    background: var(--accent-soft);
    color: var(--accent-text);
  }

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

  .cat-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 12px;
    height: 12px;
    flex-shrink: 0;
    color: var(--text-3);
  }

  .cat-toggle-spacer {
    width: 12px;
    flex-shrink: 0;
  }

  .cat-name, .cat-delete-hint {
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

  .tag-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

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

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

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
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    white-space: nowrap;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
