<script>
  import { exportLinks } from "../api.js";

  let { onclose } = $props();
  let format = $state("json");
  let exporting = $state(false);

  async function do_export() {
    exporting = true;
    try {
      const content = await exportLinks({ format });
      let mime = "text/plain";
      let ext = "txt";
      if (format === "json") { mime = "application/json"; ext = "json"; }
      else if (format === "markdown") { mime = "text/markdown"; ext = "md"; }
      else if (format === "csv") { mime = "text/csv"; ext = "csv"; }
      const blob = new Blob([content], { type: mime });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `links-export.${ext}`;
      a.click();
      URL.revokeObjectURL(url);
      onclose?.();
    } finally {
      exporting = false;
    }
  }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center" style="background:rgba(0,0,0,0.5)">
  <div class="w-full max-w-sm rounded-xl p-6 shadow-2xl" style="background:var(--color-bg)">
    <h2 class="text-lg font-semibold mb-4" style="color:var(--color-text)">导出链接</h2>
    <div class="space-y-2 mb-4">
      {#each ["json", "markdown", "csv"] as f}
        <label class="flex items-center gap-2 px-3 py-2 rounded-lg cursor-pointer" style="background:{format === f ? 'var(--color-bg-hover)' : 'transparent'};border:1px solid var(--color-border)">
          <input type="radio" name="format" value={f} bind:group={format} />
          <span class="text-sm" style="color:var(--color-text)">
            {f === "json" ? "JSON" : f === "markdown" ? "Markdown" : "CSV"}
          </span>
        </label>
      {/each}
    </div>
    <div class="flex justify-end gap-2">
      <button onclick={onclose} class="px-4 py-2 rounded-lg text-sm" style="color:var(--color-text-secondary);background:var(--color-bg-secondary)">取消</button>
      <button onclick={do_export} disabled={exporting} class="px-4 py-2 rounded-lg text-sm text-white" style="background:var(--color-primary)">{exporting ? "导出中..." : "导出"}</button>
    </div>
  </div>
</div>
