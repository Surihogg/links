<script>
  import LinkCard from "./LinkCard.svelte";

  let { links = [], categories = [], loading = false, highlight = "", onedit, ondelete, ontoggle_favorite } = $props();

  let cat_map = $derived(() => {
    const map = {};
    function walk(list) {
      for (const c of list) {
        map[c.id] = c.name;
        if (c.children) walk(c.children);
      }
    }
    walk(categories);
    return map;
  });

</script>

<div class="link-list">
  {#if loading}
    <div class="empty-state">
      <div class="spinner"></div>
      <p class="empty-text">加载中...</p>
    </div>
  {:else if links.length === 0}
    <div class="empty-state">
      <div class="empty-icon">
        <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71"/>
          <path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71"/>
        </svg>
      </div>
      <p class="empty-text">暂无链接</p>
      <p class="empty-hint">点击右下角按钮添加第一个链接</p>
    </div>
  {:else}
    {#each links as link (link.id)}
      <LinkCard {link} {highlight} category_name={link.category_id ? cat_map()[link.category_id] : null} onedit={onedit} ondelete={ondelete} ontoggle_favorite={ontoggle_favorite} />
    {/each}
  {/if}
</div>

<style>
  .link-list {
    flex: 1;
    overflow-y: auto;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 64px 24px;
    gap: 8px;
  }

  .empty-icon { color: var(--text-3); margin-bottom: 4px; }

  .empty-text { font-size: 14px; color: var(--text-2); font-weight: 500; }
  .empty-hint { font-size: 12px; color: var(--text-3); }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--border-1);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    margin-bottom: 8px;
  }

  @keyframes spin { to { transform: rotate(360deg); } }
</style>
