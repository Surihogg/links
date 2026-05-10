<script>
  import { marked } from "marked";
  import * as api from "../api.js";
  let { update_info, release_notes = "", onclose } = $props();
  let downloading = $state(false);
  let progress = $state(0);
  let downloaded = $state(0);
  let total = $state(0);
  let error_msg = $state("");

  async function start_update() {
    downloading = true;
    error_msg = "";
    try {
      await api.setSetting("last-update-notes", release_notes || update_info.body || "");
    } catch (e) {
      console.warn("[update] failed to cache release notes:", e);
    }
    try {
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
      await api.relaunchApp();
    } catch (e) {
      console.error("[update] download/install failed:", e);
      downloading = false;
      error_msg = String(e?.message || e || "未知错误");
    }
  }

  async function open_download_page() {
    await api.openUrl(`https://github.com/Surihogg/links/releases/tag/v${update_info.version}`);
  }

  function on_overlay_click(e) {
    if (e.target === e.currentTarget) onclose?.();
  }

  function handle_keydown(e) {
    if (e.key === "Escape") onclose?.();
  }
</script>

<svelte:window onkeydown={handle_keydown} />
<div class="modal-overlay" onclick={on_overlay_click}>
  <div class="modal" onclick={(e) => e.stopPropagation()}>
    <div class="modal-header">
      <h2 class="modal-title">发现新版本 v{update_info.version}</h2>
      <button class="modal-close" onclick={onclose}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
        </svg>
      </button>
    </div>

    <div class="modal-body">
      <div class="release-notes markdown-body">
        {#if release_notes || update_info.body}
          {@html marked(release_notes || update_info.body)}
        {:else}
          暂无更新说明
        {/if}
      </div>

      {#if downloading}
        <div class="progress-section">
          <div class="progress-bar-track">
            <div class="progress-bar-fill" style="width: {progress}%"></div>
          </div>
          <span class="progress-text">{progress}%</span>
        </div>
      {/if}

      {#if error_msg}
        <div class="update-error">
          <p class="error-title">自动更新失败</p>
          <p class="error-detail">{error_msg}</p>
          <p class="error-hint">你可以前往 GitHub 手动下载安装。如果覆盖安装不成功，请先卸载旧版本再安装新版本。</p>
          <button class="btn btn-secondary btn-sm" onclick={open_download_page}>前往下载</button>
        </div>
      {/if}
    </div>

    <div class="modal-footer">
      {#if !downloading && !error_msg}
        <button class="btn btn-secondary" onclick={onclose}>稍后再说</button>
      {:else if error_msg}
        <button class="btn btn-secondary" onclick={onclose}>关闭</button>
      {/if}
      {#if !downloading && !error_msg}
        <button class="btn btn-primary" onclick={start_update}>
          立即更新
        </button>
      {/if}
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
    max-height: 300px;
    overflow-y: auto;
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

  .update-error {
    margin-top: 16px;
    background: var(--bg-2);
    border-radius: var(--radius-md);
    padding: 16px;
  }

  .error-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--danger);
    margin-bottom: 6px;
  }

  .error-detail {
    font-size: 12px;
    color: var(--text-3);
    margin-bottom: 8px;
    word-break: break-all;
  }

  .error-hint {
    font-size: 12px;
    color: var(--text-2);
    margin-bottom: 12px;
    line-height: 1.5;
  }
</style>
