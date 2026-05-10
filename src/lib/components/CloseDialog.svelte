<script>
  // 关闭主窗口确认弹窗（"要走了吗？"）。
  // 当用户没设置 close-behavior 或选了"每次询问"时，关闭按钮触发本对话框。
  //
  // 与一般 .modal-overlay 不同：使用 z-index: 200 高于其它模态，且
  // 自身风格更接近 macOS 系统对话框。

  /**
   * @typedef {object} Props
   * @property {() => void} [onCancel] 点击遮罩关闭对话框
   * @property {() => void} [onMinimize] 选择"最小化到托盘"
   * @property {() => void} [onExit] 选择"退出应用"
   */
  let { onCancel, onMinimize, onExit } = $props();
</script>

<div class="close-overlay" onclick={onCancel}>
  <div class="close-dialog" onclick={(e) => e.stopPropagation()}>
    <p class="close-title">要走了吗？</p>
    <p class="close-desc">选择一下你希望的离开方式~</p>
    <div class="close-actions">
      <button class="btn btn-primary" style="flex:1" onclick={onMinimize}>最小化到托盘</button>
      <button class="btn btn-secondary" style="flex:1" onclick={onExit}>退出应用</button>
    </div>
  </div>
</div>

<style>
  .close-overlay {
    position: fixed;
    inset: 0;
    z-index: 200;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--scrim-bg);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
  }

  .close-dialog {
    background: var(--bg-0);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-xl);
    border: 1px solid var(--border-0);
    padding: 24px;
    min-width: 300px;
  }

  .close-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-0);
    margin-bottom: 4px;
  }

  .close-desc {
    font-size: 13px;
    color: var(--text-2);
    margin-bottom: 20px;
  }

  .close-actions {
    display: flex;
    gap: 8px;
  }
</style>
