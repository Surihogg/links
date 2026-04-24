<script>
  let { query = $bindable(""), onsearch } = $props();
  let timer = $state(null);

  function oninput() {
    clearTimeout(timer);
    timer = setTimeout(() => onsearch?.(query), 250);
  }

  function clear() {
    query = "";
    clearTimeout(timer);
    onsearch?.("");
  }
</script>

<div class="search-wrap">
  <svg class="search-icon" width="15" height="15" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
    <circle cx="7" cy="7" r="4.5"/>
    <line x1="10.2" y1="10.2" x2="14" y2="14"/>
  </svg>
  <input
    type="text"
    placeholder="找找看~"
    bind:value={query}
    {oninput}
    class="search-input"
  />
  {#if query}
    <button class="search-clear" onclick={clear}>
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
        <line x1="4" y1="4" x2="10" y2="10"/><line x1="10" y1="4" x2="4" y2="10"/>
      </svg>
    </button>
  {/if}
</div>

<style>
  .search-wrap {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--bg-1);
    border: 1px solid var(--border-1);
    border-radius: var(--radius-md);
    padding: 0 10px;
    height: 34px;
    transition: all var(--transition);
    width: 240px;
  }

  .search-wrap:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
    background: var(--bg-0);
  }

  .search-icon {
    color: var(--text-3);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    color: var(--text-0);
    font-size: 13px;
  }

  .search-input::placeholder {
    color: var(--text-3);
  }

  .search-clear {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-3);
    padding: 2px;
    display: flex;
    border-radius: 4px;
    transition: all var(--transition);
  }

  .search-clear:hover {
    color: var(--text-1);
    background: var(--bg-2);
  }
</style>
