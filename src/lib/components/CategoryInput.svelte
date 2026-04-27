<script>
  import { createCategory } from "../api.js";

  let { categories = [], selectedId = $bindable(null), oncreate } = $props();

  let input = $state("");
  let show_dropdown = $state(false);
  let active_index = $state(-1);
  let input_el = $state(null);
  let creating = $state(false);
  let blur_timeout = null;

  // Locally created categories (not yet in parent's list)
  let createdCategories = $state([]);

  // 将树结构扁平化，附带 depth 和 _path（如 "level1/level2"）
  function flatten_with_depth(cats, depth = 0, parent_path = '') {
    const result = [];
    for (const cat of cats) {
      const path = parent_path ? `${parent_path}/${cat.name}` : cat.name;
      result.push({ ...cat, depth, _path: path });
      if (cat.children?.length > 0) {
        result.push(...flatten_with_depth(cat.children, depth + 1, path));
      }
    }
    return result;
  }

  // Merge prop categories（树结构）with locally created ones，扁平化并带 depth
  let allCategories = $derived.by(() => {
    const flat = flatten_with_depth(categories);
    const createdIds = new Set(flat.map(c => c.id));
    const localFlat = createdCategories
      .filter(c => !createdIds.has(c.id))
      .map(c => ({ ...c, depth: 0, _path: c.name }));
    return [...flat, ...localFlat];
  });

  let selectedCategory = $derived(
    selectedId != null ? allCategories.find(c => c.id === selectedId) ?? null : null
  );

  let filteredCategories = $derived(
    input.trim()
      ? allCategories.filter(c => c.name.toLowerCase().includes(input.trim().toLowerCase()))
      : allCategories
  );

  let exactMatch = $derived(
    input.trim().length > 0 && allCategories.some(c => c.name.toLowerCase() === input.trim().toLowerCase())
  );

  let showCreateOption = $derived(input.trim().length > 0 && !exactMatch && !input.trim().includes('/'));
  let showSlashError = $derived(input.trim().includes('/'));

  let totalItems = $derived(filteredCategories.length + (showCreateOption ? 1 : 0));

  function onfocus_handler() {
    clearTimeout(blur_timeout);
    if (selectedCategory) return;
    show_dropdown = true;
    active_index = -1;
  }

  function oninput_handler(e) {
    input = e.target.value;
    show_dropdown = true;
    active_index = -1;
  }

  function select_category(cat) {
    selectedId = cat.id;
    input = "";
    show_dropdown = false;
    active_index = -1;
  }

  async function create_and_select(name) {
    const trimmed = name.trim();
    if (!trimmed || creating) return;
    creating = true;
    try {
      const cat = await createCategory({ name: trimmed });
      createdCategories = [...createdCategories, cat];
      oncreate?.(cat);
      selectedId = cat.id;
      input = "";
      show_dropdown = false;
    } catch (e) {
      console.error("Failed to create category:", e);
    }
    creating = false;
  }

  function remove_category() {
    selectedId = null;
    input = "";
    show_dropdown = true;
    active_index = -1;
    clearTimeout(blur_timeout);
    setTimeout(() => input_el?.focus(), 50);
  }

  function onkeydown(e) {
    if (selectedCategory) {
      if (e.key === "Backspace" || e.key === "Delete") {
        e.preventDefault();
        remove_category();
      } else if (e.key === "Escape") {
        show_dropdown = false;
      }
      return;
    }

    if (e.key === "ArrowDown") {
      e.preventDefault();
      active_index = Math.min(active_index + 1, totalItems - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      active_index = Math.max(active_index - 1, -1);
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (active_index >= 0) {
        if (active_index < filteredCategories.length) {
          select_category(filteredCategories[active_index]);
        } else if (showCreateOption) {
          create_and_select(input);
        }
      } else if (input.trim() && showCreateOption) {
        create_and_select(input);
      }
    } else if (e.key === "Escape") {
      show_dropdown = false;
    }
  }

  function onblur_handler() {
    blur_timeout = setTimeout(() => {
      show_dropdown = false;
      if (!selectedCategory) input = "";
    }, 150);
  }

  function on_field_click() {
    input_el?.focus();
  }
</script>

<div class="cat-input-wrap">
  <div class="cat-input-field" onclick={on_field_click}>
    {#if selectedCategory}
      <span class="cat-pill">
        {selectedCategory?._path || selectedCategory.name}
        <button type="button" class="cat-remove" onclick={remove_category}>×</button>
      </span>
    {/if}
    <input
      bind:this={input_el}
      type="text"
      value={input}
      readonly={!!selectedCategory}
      oninput={oninput_handler}
      onkeydown={onkeydown}
      onfocus={onfocus_handler}
      onblur={onblur_handler}
      placeholder={selectedCategory ? "" : "搜索或创建分组"}
      class="cat-text-input"
      class:cat-text-readonly={!!selectedCategory}
    />
  </div>

  {#if show_dropdown && !selectedCategory}
    <div class="cat-dropdown">
      {#if showSlashError}
        <div class="cat-error">分组名不能包含 /</div>
      {:else if filteredCategories.length === 0 && !showCreateOption}
        <div class="cat-empty">暂无分组</div>
      {:else}
        {#each filteredCategories as cat, i}
          <button
            type="button"
            class="cat-item"
            class:active={i === active_index}
            style="padding-left: {10 + (cat.depth || 0) * 16}px"
            onclick={() => select_category(cat)}
            onmouseenter={() => active_index = i}
          >
            {cat.name}
          </button>
        {/each}
        {#if showCreateOption}
          <button
            type="button"
            class="cat-item cat-create"
            class:active={active_index === filteredCategories.length}
            onclick={() => create_and_select(input)}
            onmouseenter={() => active_index = filteredCategories.length}
          >
            创建 "{input.trim()}"
          </button>
        {/if}
      {/if}
    </div>
  {/if}
</div>

<style>
  .cat-input-wrap { position: relative; }

  .cat-input-field {
    display: flex;
    flex-wrap: nowrap;
    gap: 4px;
    padding: 6px 8px;
    border: 1px solid var(--border-1);
    border-radius: var(--radius-md);
    background: var(--bg-0);
    min-height: 36px;
    align-items: center;
    cursor: text;
    transition: border-color var(--transition);
  }

  .cat-input-field:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .cat-pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 1px 6px;
    background: var(--cat-soft);
    color: var(--cat-text);
    border-radius: 4px;
    font-size: 12px;
    font-weight: 500;
    white-space: nowrap;
  }

  .cat-remove {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--cat-text);
    font-size: 14px;
    line-height: 1;
    padding: 0;
    opacity: 0.6;
    transition: opacity var(--transition);
  }

  .cat-remove:hover { opacity: 1; }

  .cat-text-input {
    flex: 1;
    min-width: 60px;
    background: none;
    border: none;
    outline: none;
    color: var(--text-0);
    font-size: 13px;
  }

  .cat-text-input::placeholder { color: var(--text-3); }

  .cat-text-readonly {
    cursor: default;
    color: transparent;
    caret-color: transparent;
    min-width: 0;
    flex: 0;
    width: 0;
    padding: 0;
  }

  .cat-dropdown {
    position: absolute;
    z-index: 10;
    width: 100%;
    margin-top: 4px;
    background: var(--bg-0);
    border: 1px solid var(--border-1);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    overflow: hidden;
    max-height: 200px;
    overflow-y: auto;
  }

  .cat-empty {
    padding: 10px;
    text-align: center;
    color: var(--text-3);
    font-size: 12px;
  }

  .cat-error {
    padding: 6px 10px;
    color: var(--danger);
    font-size: 12px;
    text-align: center;
    border-top: 1px solid var(--border-0);
  }

  .cat-item {
    width: 100%;
    text-align: left;
    padding: 6px 10px;
    border: none;
    background: none;
    color: var(--text-1);
    font-size: 12px;
    cursor: pointer;
    display: block;
    transition: background var(--transition);
  }

  .cat-item:hover,
  .cat-item.active { background: var(--accent-soft); }

  .cat-create {
    color: var(--accent);
    border-top: 1px solid var(--border-0);
  }
</style>
