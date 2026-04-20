<script>
  import LinkCard from "./LinkCard.svelte";

  let { links = [], loading = false, onedit, ondelete, ontoggle_favorite } = $props();
</script>

<div class="flex-1 overflow-y-auto p-4 space-y-2">
  {#if loading}
    <div class="text-center py-12" style="color:var(--color-text-secondary)">
      <div class="animate-spin inline-block w-6 h-6 border-2 rounded-full" style="border-color:var(--color-border);border-top-color:var(--color-primary)"></div>
      <p class="mt-2 text-sm">加载中...</p>
    </div>
  {:else if links.length === 0}
    <div class="text-center py-12" style="color:var(--color-text-secondary)">
      <p class="text-lg mb-1">📭</p>
      <p class="text-sm">暂无链接</p>
      <p class="text-xs mt-1">点击下方按钮添加第一个链接</p>
    </div>
  {:else}
    {#each links as link (link.id)}
      <LinkCard {link} onedit={onedit} ondelete={ondelete} ontoggle_favorite={ontoggle_favorite} />
    {/each}
  {/if}
</div>
