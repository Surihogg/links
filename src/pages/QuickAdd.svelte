<script>
  import TagInput from "../lib/components/TagInput.svelte";
  import * as api from "../lib/api.js";
  import { categoriesStore } from "../lib/stores/index.js";

  let url = $state("");
  let title = $state("");
  let tags = $state([]);
  let saving = $state(false);
  let categories = $state([]);
  let category_id = $state(null);

  $effect(() => {
    categoriesStore.load().then(() => {
      categories = $state.snapshot(categoriesStore).subscribe ? [] : [];
    });
    api.listCategories().then(c => categories = c);
  });

  async function submit() {
    if (!url.trim()) return;
    saving = true;
    try {
      await api.createLink({
        url: url.trim(),
        title: title.trim() || undefined,
        tags,
        category_id,
      });
      url = "";
      title = "";
      tags = [];
      category_id = null;
    } finally {
      saving = false;
    }
  }
</script>

<div class="p-4" style="background:var(--color-bg);min-height:100vh">
  <h2 class="text-sm font-semibold mb-3" style="color:var(--color-text)">快速添加链接</h2>
  <form onsubmit={(e) => { e.preventDefault(); submit(); }} class="space-y-2">
    <input type="url" bind:value={url} required placeholder="粘贴链接..." autofocus class="w-full px-3 py-2 rounded-lg text-sm outline-none" style="background:var(--color-bg-secondary);border:1px solid var(--color-border);color:var(--color-text)" />
    <input type="text" bind:value={title} placeholder="标题（可选）" class="w-full px-3 py-2 rounded-lg text-sm outline-none" style="background:var(--color-bg-secondary);border:1px solid var(--color-border);color:var(--color-text)" />
    <TagInput bind:tags />
    <select bind:value={category_id} class="w-full px-3 py-2 rounded-lg text-sm outline-none" style="background:var(--color-bg-secondary);border:1px solid var(--color-border);color:var(--color-text)">
      <option value="">无分组</option>
      {#each categories as cat}
        <option value={cat.id}>{cat.name}</option>
      {/each}
    </select>
    <button type="submit" disabled={saving || !url.trim()} class="w-full py-2 rounded-lg text-sm text-white font-medium" style="background:var(--color-primary)">
      {saving ? "保存中..." : "保存"}
    </button>
  </form>
</div>
