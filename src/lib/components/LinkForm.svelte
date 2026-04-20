<script>
  import TagInput from "./TagInput.svelte";
  import { categoriesStore } from "../stores/index.js";

  let { link = null, categories = [], onsave, oncancel } = $props();

  let url = $state(link?.url ?? "");
  let title = $state(link?.title ?? "");
  let description = $state(link?.description ?? "");
  let notes = $state(link?.notes ?? "");
  let category_id = $state(link?.category_id ?? null);
  let tags = $state(link?.tags?.slice() ?? []);
  let saving = $state(false);

  function submit() {
    if (!url.trim()) return;
    saving = true;
    onsave?.({
      id: link?.id,
      url: url.trim(),
      title: title.trim() || undefined,
      description: description.trim() || undefined,
      notes: notes.trim() || undefined,
      category_id,
      tags,
    });
  }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center" style="background:rgba(0,0,0,0.5)">
  <div class="w-full max-w-lg rounded-xl p-6 shadow-2xl" style="background:var(--color-bg)">
    <h2 class="text-lg font-semibold mb-4" style="color:var(--color-text)">{link ? "编辑链接" : "添加链接"}</h2>
    <form onsubmit={(e) => { e.preventDefault(); submit(); }} class="space-y-3">
      <div>
        <label class="block text-xs mb-1 font-medium" style="color:var(--color-text-secondary)">URL *</label>
        <input type="url" bind:value={url} required placeholder="https://..." class="w-full px-3 py-2 rounded-lg text-sm outline-none" style="background:var(--color-bg-secondary);border:1px solid var(--color-border);color:var(--color-text)" />
      </div>
      <div>
        <label class="block text-xs mb-1 font-medium" style="color:var(--color-text-secondary)">标题</label>
        <input type="text" bind:value={title} placeholder="自动抓取" class="w-full px-3 py-2 rounded-lg text-sm outline-none" style="background:var(--color-bg-secondary);border:1px solid var(--color-border);color:var(--color-text)" />
      </div>
      <div>
        <label class="block text-xs mb-1 font-medium" style="color:var(--color-text-secondary)">描述</label>
        <textarea bind:value={description} rows="2" placeholder="自动抓取" class="w-full px-3 py-2 rounded-lg text-sm outline-none resize-none" style="background:var(--color-bg-secondary);border:1px solid var(--color-border);color:var(--color-text)"></textarea>
      </div>
      <div>
        <label class="block text-xs mb-1 font-medium" style="color:var(--color-text-secondary)">备注</label>
        <textarea bind:value={notes} rows="2" placeholder="个人备注..." class="w-full px-3 py-2 rounded-lg text-sm outline-none resize-none" style="background:var(--color-bg-secondary);border:1px solid var(--color-border);color:var(--color-text)"></textarea>
      </div>
      <div>
        <label class="block text-xs mb-1 font-medium" style="color:var(--color-text-secondary)">分类</label>
        <select bind:value={category_id} class="w-full px-3 py-2 rounded-lg text-sm outline-none" style="background:var(--color-bg-secondary);border:1px solid var(--color-border);color:var(--color-text)">
          <option value={null}>未分类</option>
          {#each categories as cat}
            <option value={cat.id}>{cat.name}</option>
          {/each}
        </select>
      </div>
      <div>
        <label class="block text-xs mb-1 font-medium" style="color:var(--color-text-secondary)">标签</label>
        <TagInput bind:tags />
      </div>
      <div class="flex justify-end gap-2 pt-2">
        <button type="button" onclick={oncancel} class="px-4 py-2 rounded-lg text-sm" style="color:var(--color-text-secondary);background:var(--color-bg-secondary)">取消</button>
        <button type="submit" disabled={saving} class="px-4 py-2 rounded-lg text-sm text-white" style="background:var(--color-primary)">{saving ? "保存中..." : "保存"}</button>
      </div>
    </form>
  </div>
</div>
