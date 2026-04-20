<script>
  let { query = "", onsearch } = $props();
</script>

<div class="flex items-center gap-2 px-4 py-2" style="background:var(--color-bg-secondary);border-bottom:1px solid var(--color-border)">
  <div class="flex-1 flex items-center gap-2 rounded-lg px-3 py-2" style="background:var(--color-bg);border:1px solid var(--color-border)">
    <svg class="w-4 h-4 shrink-0" style="color:var(--color-text-secondary)" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
    </svg>
    <input
      type="text"
      placeholder="搜索链接..."
      bind:value={query}
      onkeydown={(e) => e.key === "Enter" && onsearch?.(query)}
      class="flex-1 bg-transparent outline-none text-sm"
      style="color:var(--color-text)"
    />
    {#if query}
      <button onclick={() => { query = ""; onsearch?.(""); }} class="text-xs" style="color:var(--color-text-secondary)">✕</button>
    {/if}
  </div>
</div>
