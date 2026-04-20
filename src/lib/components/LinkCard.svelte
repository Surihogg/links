<script>
  let { link, onedit, ondelete, ontoggle_favorite } = $props();
</script>

<div class="link-card group rounded-lg p-4 transition-colors cursor-default">
  <div class="flex items-start gap-3">
    {#if link.favicon_url}
      <img src={link.favicon_url} alt="" class="w-5 h-5 mt-0.5 rounded shrink-0" onerror={(e) => e.target.style.display = 'none'} />
    {:else}
      <div class="w-5 h-5 mt-0.5 rounded shrink-0 flex items-center justify-center text-xs" style="background:var(--color-border);color:var(--color-text-secondary)">🔗</div>
    {/if}
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2">
        <a href={link.url} target="_blank" rel="noopener" class="font-medium text-sm truncate hover:underline" style="color:var(--color-text)">
          {link.title || link.url}
        </a>
        {#if link.is_favorite}
          <span class="text-yellow-500 text-xs">★</span>
        {/if}
      </div>
      <p class="text-xs truncate mt-0.5" style="color:var(--color-text-secondary)">{link.url}</p>
      {#if link.description}
        <p class="text-xs mt-1.5" style="color:var(--color-text-secondary);display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden">{link.description}</p>
      {/if}
      {#if link.tags.length > 0}
        <div class="flex flex-wrap gap-1 mt-2">
          {#each link.tags as tag}
            <span class="px-1.5 py-0.5 rounded text-xs" style="background:var(--color-tag-bg);color:var(--color-tag-text)">#{tag}</span>
          {/each}
        </div>
      {/if}
    </div>
    <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
      <button onclick={() => ontoggle_favorite?.(link)} class="p-1 rounded text-sm" style="color:{link.is_favorite ? '#eab308' : 'var(--color-text-secondary)'}" title={link.is_favorite ? '取消收藏' : '收藏'}>
        {link.is_favorite ? '★' : '☆'}
      </button>
      <button onclick={() => onedit?.(link)} class="p-1 rounded text-sm" style="color:var(--color-text-secondary)" title="编辑">✎</button>
      <button onclick={() => ondelete?.(link)} class="p-1 rounded text-sm" style="color:var(--color-danger)" title="删除">✕</button>
    </div>
  </div>
</div>

<style>
  .link-card {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
  }
  .link-card:hover {
    background: var(--color-bg-hover);
  }
</style>
