<script>
  import { categoriesStore } from "../stores/index.js";

  let { categories = [], selected_id = null, onselect, oncreate } = $props();
  let expanded = $state(new Set());
  let show_new = $state(false);
  let new_name = $state("");
  let new_parent_id = $state(null);

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
</script>

<aside class="w-56 shrink-0 flex flex-col overflow-y-auto" style="background:var(--color-bg-secondary);border-right:1px solid var(--color-border)">
  <div class="p-3 space-y-0.5">
    <button
      class="w-full text-left px-3 py-2 rounded-lg text-sm"
      style="background:{selected_id === null ? 'var(--color-bg-hover)' : 'transparent'};color:var(--color-text)"
      onclick={() => onselect?.(null)}
    >
      全部链接
    </button>
    <button
      class="w-full text-left px-3 py-2 rounded-lg text-sm"
      style="background:{selected_id === 'favorite' ? 'var(--color-bg-hover)' : 'transparent'};color:var(--color-text)"
      onclick={() => onselect?.('favorite')}
    >
      ★ 收藏
    </button>
  </div>

  <div class="px-3 pt-1 pb-1">
    <span class="text-xs font-medium" style="color:var(--color-text-secondary)">分类</span>
  </div>

  <div class="flex-1 px-3 space-y-0.5">
    {#each categories as cat (cat.id)}
      <div>
        <button
          class="w-full text-left px-3 py-1.5 rounded-lg text-sm flex items-center gap-1"
          style="background:{selected_id === cat.id ? 'var(--color-bg-hover)' : 'transparent'};color:var(--color-text)"
          onclick={() => onselect?.(cat.id)}
        >
          {#if cat.children.length > 0}
            <span class="text-xs w-3" onclick={(e) => { e.stopPropagation(); toggle(cat.id); }}>
              {expanded.has(cat.id) ? '▼' : '▶'}
            </span>
          {:else}
            <span class="w-3"></span>
          {/if}
          📁 {cat.name}
        </button>
        {#if expanded.has(cat.id) && cat.children.length > 0}
          <div class="ml-4 space-y-0.5">
            {#each cat.children as child (child.id)}
              <button
                class="w-full text-left px-3 py-1.5 rounded-lg text-sm"
                style="background:{selected_id === child.id ? 'var(--color-bg-hover)' : 'transparent'};color:var(--color-text)"
                onclick={() => onselect?.(child.id)}
              >
                📁 {child.name}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </div>

  <div class="p-3">
    {#if show_new}
      <form onsubmit={(e) => { e.preventDefault(); submit_category(); }} class="space-y-2">
        <input type="text" bind:value={new_name} placeholder="分类名称" class="w-full px-2 py-1.5 rounded text-xs outline-none" style="background:var(--color-bg);border:1px solid var(--color-border);color:var(--color-text)" />
        <div class="flex gap-1">
          <button type="submit" class="px-2 py-1 rounded text-xs text-white" style="background:var(--color-primary)">确定</button>
          <button type="button" onclick={() => show_new = false} class="px-2 py-1 rounded text-xs" style="color:var(--color-text-secondary)">取消</button>
        </div>
      </form>
    {:else}
      <button onclick={() => show_new = true} class="w-full px-3 py-1.5 rounded-lg text-xs text-left" style="color:var(--color-text-secondary)">
        + 新建分类
      </button>
    {/if}
  </div>
</aside>
