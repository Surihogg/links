<script>
  import * as api from "../api.js";
  let { update_info, onclose } = $props();
  let downloading = $state(false);
  let progress = $state(0);
  let downloaded = $state(0);
  let total = $state(0);

  async function start_update() {
    downloading = true;
    await api.downloadAndInstallUpdate(update_info, (event) => {
      switch (event.event) {
        case "Started":
          total = event.data.contentLength || 0;
          break;
        case "Progress":
          downloaded += event.data.chunkLength;
          progress = total > 0 ? Math.round(downloaded / total * 100) : 0;
          break;
        case "Finished":
          progress = 100;
          break;
      }
    });
    // 下载安装完成后重启应用
    await api.relaunchApp();
  }

  function on_overlay_click(e) {
    if (e.target === e.currentTarget && !downloading) onclose?.();
  }

  function handle_keydown(e) {
    if (e.key === "Escape" && !downloading) onclose?.();
  }
</script>

<svelte:window onkeydown={handle_keydown} />
<div class="modal-overlay" onclick={on_overlay_click}>
  <div class="modal" onclick={(e) => e.stopPropagation()}>
    <div class="modal-header">
      <h2 class="modal-title">发现新版本 v{update_info.version}</h2>
      {#if !downloading}
        <button class="modal-close" onclick={onclose}>
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
            <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
          </svg>
        </button>
      {/if}
    </div>

    <div class="modal-body">
      <div class="release-notes">
        {update_info.body || "暂无更新说明"}
      </div>

      {#if downloading}
        <div class="progress-section">
          <div class="progress-bar-track">
            <div class="progress-bar-fill" style="width: {progress}%"></div>
          </div>
          <span class="progress-text">{progress}%</span>
        </div>
      {/if}
    </div>

    <div class="modal-footer">
      {#if !downloading}
        <button class="btn btn-secondary" onclick={onclose}>稍后再说</button>
      {/if}
      <button class="btn btn-primary" onclick={start_update} disabled={downloading}>
        {#if downloading}
          下载中... {progress}%
        {:else}
          立即更新
        {/if}
      </button>
    </div>
  </div>
</div>

<style>
  .modal {
    max-width: 420px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
  }

  .modal-body {
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 20px 20px;
    flex-shrink: 0;
  }

  .release-notes {
    background: var(--bg-1);
    border-radius: var(--radius-md);
    padding: 16px;
    font-size: 13px;
    color: var(--text-1);
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 300px;
    overflow-y: auto;
    line-height: 1.6;
  }

  .progress-section {
    margin-top: 16px;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .progress-bar-track {
    flex: 1;
    height: 4px;
    background: var(--bg-2);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 2px;
    transition: width 200ms ease;
  }

  .progress-text {
    font-size: 12px;
    color: var(--text-3);
    font-weight: 500;
    min-width: 36px;
    text-align: right;
  }
</style>
