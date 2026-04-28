<script>
  let { query = $bindable(""), onsearch, filter_chip = null, onremovefilter } = $props();
  let timer = $state(null);
  let input_el;

  export function focus() {
    input_el?.focus();
  }

  function oninput() {
    clearTimeout(timer);
    timer = setTimeout(() => onsearch?.(query), 250);
  }

  function clear() {
    query = "";
    clearTimeout(timer);
    onsearch?.("");
  }

  function onkeydown(e) {
    if (e.key === "Backspace" && query === "" && filter_chip) {
      e.preventDefault();
      onremovefilter?.();
    }
  }
</script>

<div class="search-wrap">
  <svg class="search-icon" width="15" height="15" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
    <circle cx="7" cy="7" r="4.5"/>
    <line x1="10.2" y1="10.2" x2="14" y2="14"/>
  </svg>
  {#if filter_chip}
    <span class="filter-chip" class:filter-chip--category={filter_chip.type === 'category'} class:filter-chip--favorite={filter_chip.type === 'favorite'}>
      {filter_chip.label}
      <button class="chip-remove" onclick={onremovefilter}>
        <svg width="10" height="10" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <line x1="4" y1="4" x2="10" y2="10"/><line x1="10" y1="4" x2="4" y2="10"/>
        </svg>
      </button>
    </span>
  {/if}
  <input
    type="text"
    placeholder="找找看~"
    bind:value={query}
    bind:this={input_el}
    {oninput}
    {onkeydown}
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

  .filter-chip {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 1px 6px 1px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 500;
    white-space: nowrap;
    flex-shrink: 0;
    background: var(--accent-soft);
    color: var(--accent-text);
  }

  .filter-chip--category {
    background: var(--cat-soft);
    color: var(--cat-text);
  }

  .filter-chip--favorite {
    background: var(--star-soft);
    color: var(--star-text);
  }

  .chip-remove {
    background: none;
    border: none;
    cursor: pointer;
    color: inherit;
    padding: 1px;
    display: flex;
    border-radius: 3px;
    opacity: 0.6;
    transition: all var(--transition);
  }

  .chip-remove:hover {
    opacity: 1;
  }
</style>
