<script>
  import { autocompleteTags } from "../api.js";

  let { tags = $bindable([]), onchange } = $props();
  let input = $state("");
  let suggestions = $state([]);
  let show_suggestions = $state(false);
  let active_index = $state(-1);

  async function oninput(e) {
    input = e.target.value;
    if (input.length > 0) {
      suggestions = await autocompleteTags(input);
      show_suggestions = suggestions.length > 0;
      active_index = -1;
    } else {
      show_suggestions = false;
    }
  }

  function add_tag(name) {
    const t = name.trim();
    if (t && !tags.includes(t)) {
      tags = [...tags, t];
      onchange?.(tags);
    }
    input = "";
    show_suggestions = false;
  }

  function remove_tag(index) {
    tags = tags.filter((_, i) => i !== index);
    onchange?.(tags);
  }

  function onkeydown(e) {
    if (e.key === "Enter" || e.key === ",") {
      e.preventDefault();
      if (active_index >= 0 && suggestions[active_index]) {
        add_tag(suggestions[active_index].name);
      } else if (input.trim()) {
        add_tag(input);
      }
    } else if (e.key === "Backspace" && !input && tags.length > 0) {
      remove_tag(tags.length - 1);
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      active_index = Math.min(active_index + 1, suggestions.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      active_index = Math.max(active_index - 1, -1);
    } else if (e.key === "Escape") {
      show_suggestions = false;
    }
  }
</script>

<div class="tag-input-wrap">
  <div class="tag-input-field">
    {#each tags as tag, i}
      <span class="tag-pill">
        {tag}
        <button class="tag-remove" onclick={() => remove_tag(i)}>×</button>
      </span>
    {/each}
    <input
      type="text"
      value={input}
      oninput={oninput}
      onkeydown={onkeydown}
      onblur={() => setTimeout(() => show_suggestions = false, 150)}
      placeholder={tags.length === 0 ? "快到碗里来！" : ""}
      class="tag-text-input"
    />
  </div>
  {#if show_suggestions}
    <div class="tag-suggestions">
      {#each suggestions as suggestion, i}
        <button
          class="suggestion-item"
          class:active={i === active_index}
          onclick={() => add_tag(suggestion.name)}
          onmouseenter={() => active_index = i}
        >
          {suggestion.name}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .tag-input-wrap { position: relative; }

  .tag-input-field {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding: 6px 8px;
    border: 1px solid var(--border-1);
    border-radius: var(--radius-md);
    background: var(--bg-0);
    min-height: 36px;
    align-items: center;
    transition: border-color var(--transition);
  }

  .tag-input-field:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .tag-pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 1px 6px;
    background: var(--accent-soft);
    color: var(--accent-text);
    border-radius: 4px;
    font-size: 12px;
    font-weight: 500;
  }

  .tag-remove {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--accent-text);
    font-size: 14px;
    line-height: 1;
    padding: 0;
    opacity: 0.6;
    transition: opacity var(--transition);
  }

  .tag-remove:hover { opacity: 1; }

  .tag-text-input {
    flex: 1;
    min-width: 80px;
    background: none;
    border: none;
    outline: none;
    color: var(--text-0);
    font-size: 13px;
  }

  .tag-text-input::placeholder { color: var(--text-3); }

  .tag-suggestions {
    position: absolute;
    z-index: 10;
    width: 100%;
    margin-top: 4px;
    background: var(--bg-0);
    border: 1px solid var(--border-1);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    overflow: hidden;
  }

  .suggestion-item {
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

  .suggestion-item:hover,
  .suggestion-item.active { background: var(--accent-soft); }
</style>
