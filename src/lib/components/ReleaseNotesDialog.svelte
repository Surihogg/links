<script>
  // 发布说明对话框：升级到新版本后首次启动展示。
  // 内容来自 GitHub Releases API（fetchReleaseNotes）或 last-update-notes 配置兜底，
  // 由父组件预先解析为 markdown 字符串后传入。

  import { marked } from "marked";

  /**
   * @typedef {object} Props
   * @property {string} [version] 当前版本号（不带 v 前缀）
   * @property {string} [notes] markdown 正文；为空时显示占位文案
   * @property {() => void} [onClose] 关闭回调
   */
  let { version = "", notes = "", onClose } = $props();
</script>

<div class="modal-overlay" onclick={onClose}>
  <div class="modal" onclick={(e) => e.stopPropagation()}>
    <div class="modal-header">
      <h3 class="modal-title">当前版本：v{version}</h3>
      <button class="modal-close" onclick={onClose} aria-label="关闭">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
        </svg>
      </button>
    </div>
    <div class="modal-body">
      <p class="release-notes-intro">版本更新内容：</p>
      <div class="release-notes-content markdown-body">
        {#if notes}
          {@html marked(notes)}
        {:else}
          暂无更新说明
        {/if}
      </div>
    </div>
    <div class="modal-footer">
      <button class="btn btn-primary" onclick={onClose}>知道了</button>
    </div>
  </div>
</div>

<style>
  .release-notes-intro {
    font-size: 13px;
    color: var(--text-2);
    margin-bottom: 12px;
  }

  .release-notes-content {
    background: var(--bg-1);
    border-radius: var(--radius-md);
    padding: 16px;
    max-height: 300px;
    overflow-y: auto;
  }

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
</style>
