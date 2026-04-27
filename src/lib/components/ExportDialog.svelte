<script>
  import { exportLinks, saveFile } from "../api.js";

  let { onclose } = $props();
  let format = $state("json");
  let exporting = $state(false);

  const formats = [
    { id: "json", name: "JSON", desc: "结构化数据，可重新导入" },
    { id: "html", name: "HTML", desc: "浏览器书签格式，可导入 Chrome/Firefox/Safari" },
    { id: "markdown", name: "Markdown", desc: "纯文本格式，方便阅读" },
    { id: "csv", name: "CSV", desc: "表格格式，可在 Excel 中打开" },
  ];

  async function do_export() {
    exporting = true;
    try {
      const content = await exportLinks({ format });
      let ext = "txt";
      let filename = "links-export";
      if (format === "json") { ext = "json"; }
      else if (format === "html") { ext = "html"; }
      else if (format === "markdown") { ext = "md"; }
      else if (format === "csv") { ext = "csv"; }
      await saveFile(content, `${filename}.${ext}`);
      onclose?.();
    } finally {
      exporting = false;
    }
  }

  function on_overlay_click(e) {
    if (e.target === e.currentTarget) onclose?.();
  }
</script>

<div class="modal-overlay" onclick={on_overlay_click}>
  <div class="modal">
    <div class="modal-header">
      <h2 class="modal-title">导出链接</h2>
      <button class="modal-close" onclick={onclose}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
        </svg>
      </button>
    </div>

    <div class="modal-body">
      <div class="format-list">
        {#each formats as f (f.id)}
          <button
            class="format-option"
            class:active={format === f.id}
            onclick={() => format = f.id}
          >
            <div class="format-radio">
              {#if format === f.id}
                <div class="format-dot"></div>
              {/if}
            </div>
            <div class="format-info">
              <span class="format-name">{f.name}</span>
              <span class="format-desc">{f.desc}</span>
            </div>
            {#if format === f.id}
              <svg class="format-check" width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M3 8l3.5 3.5L13 5"/>
              </svg>
            {/if}
          </button>
        {/each}
      </div>

      <div class="modal-footer">
        <button onclick={onclose} class="btn btn-secondary">取消</button>
        <button onclick={do_export} disabled={exporting} class="btn btn-primary">
          {exporting ? "导出中..." : "导出"}
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .modal {
    max-width: 400px;
  }

  .format-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 16px;
  }

  .format-option {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 10px 12px;
    border: 1px solid var(--border-1);
    border-radius: var(--radius-md);
    background: var(--bg-1);
    color: var(--text-1);
    cursor: pointer;
    transition: all var(--transition);
    text-align: left;
    font-size: 13px;
  }

  .format-option:hover {
    border-color: var(--border-2);
    background: var(--bg-hover);
  }

  .format-option.active {
    border-color: var(--accent);
    background: var(--accent-soft);
    color: var(--accent-text);
  }

  .format-radio {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 1.5px solid var(--border-2);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: border-color var(--transition);
  }

  .format-option.active .format-radio {
    border-color: var(--accent);
  }

  .format-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
  }

  .format-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .format-name {
    font-weight: 500;
    font-size: 13px;
  }

  .format-desc {
    font-size: 11px;
    color: var(--text-3);
  }

  .format-option.active .format-desc {
    color: var(--accent-text);
    opacity: 0.7;
  }

  .format-check {
    color: var(--accent);
    flex-shrink: 0;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
