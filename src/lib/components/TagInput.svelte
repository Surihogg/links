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

<div class="relative">
  <div class="flex flex-wrap gap-1 p-2 rounded-lg min-h-[38px] items-center" style="border:1px solid var(--color-border);background:var(--color-bg)">
    {#each tags as tag, i}
      <span class="flex items-center gap-1 px-1.5 py-0.5 rounded text-xs" style="background:var(--color-tag-bg);color:var(--color-tag-text)">
        #{tag}
        <button onclick={() => remove_tag(i)} class="hover:opacity-70">✕</button>
      </span>
    {/each}
    <input
      type="text"
      value={input}
      oninput={oninput}
      onkeydown={onkeydown}
      onblur={() => setTimeout(() => show_suggestions = false, 150)}
      placeholder={tags.length === 0 ? "输入标签，回车添加..." : ""}
      class="flex-1 min-w-[80px] bg-transparent outline-none text-sm"
      style="color:var(--color-text)"
    />
  </div>
  {#if show_suggestions}
    <div class="absolute z-10 mt-1 w-full rounded-lg shadow-lg overflow-hidden" style="background:var(--color-bg);border:1px solid var(--color-border)">
      {#each suggestions as suggestion, i}
        <button
          class="w-full text-left px-3 py-1.5 text-sm"
          style="background:{i === active_index ? 'var(--color-bg-hover)' : 'transparent'};color:var(--color-text)"
          onclick={() => add_tag(suggestion.name)}
          onmouseenter={() => active_index = i}
        >
          #{suggestion.name}
        </button>
      {/each}
    </div>
  {/if}
</div>
