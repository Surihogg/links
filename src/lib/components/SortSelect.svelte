<script>
  let { value = $bindable(""), options = [], onchange } = $props();

  let open = $state(false);
  let active_index = $state(-1);
  let blur_timeout = null;

  let normalizedValue = $derived(value ?? "");

  let selectedLabel = $derived(
    options.find(o => o.value === normalizedValue)?.label || ""
  );

  function toggle() {
    open = !open;
    if (open) {
      active_index = options.findIndex(o => o.value === normalizedValue);
    }
  }

  function select(option) {
    value = option.value;
    onchange?.(option.value);
    open = false;
    active_index = -1;
  }

  function onkeydown(e) {
    if (!open) {
      if (e.key === "Enter" || e.key === "ArrowDown" || e.key === " ") {
        e.preventDefault();
        open = true;
        active_index = options.findIndex(o => o.value === normalizedValue);
      }
      return;
    }

    if (e.key === "ArrowDown") {
      e.preventDefault();
      active_index = Math.min(active_index + 1, options.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      active_index = Math.max(active_index - 1, 0);
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (active_index >= 0) {
        select(options[active_index]);
      }
    } else if (e.key === "Escape") {
      e.preventDefault();
      open = false;
      active_index = -1;
    }
  }

  function onblur_handler() {
    blur_timeout = setTimeout(() => {
      open = false;
      active_index = -1;
    }, 150);
  }

  function onfocus_handler() {
    clearTimeout(blur_timeout);
  }
</script>

<div class="sort-select-wrap">
  <button
    type="button"
    class="sort-select-trigger"
    class:open
    onclick={toggle}
    onkeydown={onkeydown}
    onblur={onblur_handler}
    onfocus={onfocus_handler}
  >
    <span class="sort-select-label">{selectedLabel}</span>
    <svg class="sort-select-arrow" width="10" height="6" viewBox="0 0 10 6" fill="none" stroke="currentColor" stroke-width="1.5">
      <path d="M1 1l4 4 4-4"/>
    </svg>
  </button>

  {#if open}
    <div class="sort-select-dropdown">
      {#each options as option, i}
        <button
          type="button"
          class="sort-select-item"
          class:active={i === active_index}
          class:selected={option.value === normalizedValue}
          onclick={() => select(option)}
          onmouseenter={() => active_index = i}
        >
          {option.label}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .sort-select-wrap {
    position: relative;
    flex-shrink: 0;
  }

  .sort-select-trigger {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 8px;
    border: 1px solid var(--border-1);
    border-radius: var(--radius-md);
    background: var(--bg-0);
    min-height: 36px;
    color: var(--text-2);
    font-size: 12px;
    cursor: pointer;
    transition: border-color var(--transition);
    outline: none;
  }

  .sort-select-trigger:hover {
    border-color: var(--border-2);
  }

  .sort-select-trigger:focus,
  .sort-select-trigger.open {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .sort-select-label {
    white-space: nowrap;
  }

  .sort-select-arrow {
    color: var(--text-3);
    flex-shrink: 0;
    transition: transform var(--transition);
  }

  .sort-select-trigger.open .sort-select-arrow {
    transform: rotate(180deg);
  }

  .sort-select-dropdown {
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

  .sort-select-item {
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

  .sort-select-item:hover,
  .sort-select-item.active {
    background: var(--accent-soft);
  }

  .sort-select-item.selected {
    color: var(--accent);
    font-weight: 500;
  }
</style>
