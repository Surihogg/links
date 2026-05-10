<script>
  // Sidebar 顶部品牌头：图标 + Links 文字 + 版本号 + 更新铃 + 设置齿轮。
  // 从 Sidebar.svelte 抽出，CSS 自包含，避免 Sidebar 主组件体积过大。

  import { getVersion } from "@tauri-apps/api/app";

  /**
   * @typedef {object} Props
   * @property {boolean} [hasUpdate] 是否有可用更新（控制更新铃显示）
   * @property {() => void} [onSettings] 点击设置齿轮
   * @property {() => void} [onUpdate] 点击更新铃
   */
  let { hasUpdate = false, onSettings, onUpdate } = $props();

  let version = $state("");
  getVersion().then((v) => (version = v));
</script>

<div class="sidebar-brand">
  <span class="brand-icon">◈</span>
  <span class="brand-text">Links</span>
  {#if version}
    <span class="brand-version">v{version}</span>
  {/if}
  {#if hasUpdate}
    <button
      class="brand-update"
      onclick={onUpdate}
      data-tooltip="有新版本可用"
      aria-label="有新版本可用"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 11-7.778 7.778 5.5 5.5 0 017.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/>
      </svg>
    </button>
  {/if}
  <button class="brand-settings" onclick={onSettings} aria-label="设置">
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="12" cy="12" r="3"/>
      <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/>
    </svg>
  </button>
</div>

<style>
  .sidebar-brand {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 16px 16px 12px;
  }

  .brand-icon {
    font-size: 18px;
    color: var(--accent);
  }

  .brand-text {
    font-size: 18px;
    font-weight: 700;
    color: var(--text-0);
    letter-spacing: -0.5px;
    font-family: "Georgia", "Times New Roman", serif;
  }

  .brand-version {
    font-size: 11px;
    color: var(--text-3);
    font-weight: 400;
    margin-left: 2px;
    align-self: center;
  }

  .brand-settings {
    margin-left: auto;
    width: 24px;
    height: 24px;
    border: none;
    background: none;
    color: var(--text-3);
    cursor: pointer;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition);
  }

  .brand-settings:hover {
    color: var(--text-1);
    background: var(--bg-hover);
  }

  .brand-update {
    position: relative;
    width: 24px;
    height: 24px;
    border: none;
    background: none;
    color: var(--accent);
    cursor: pointer;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition);
    animation: pulse-glow 2s ease-in-out infinite;
  }

  .brand-update:hover {
    background: var(--accent-soft);
  }

  .brand-update[data-tooltip]:hover::after {
    content: attr(data-tooltip);
    position: absolute;
    top: 100%;
    left: 50%;
    transform: translateX(-50%);
    margin-top: 4px;
    padding: 4px 8px;
    background: var(--text-0);
    color: var(--bg-0);
    font-size: 11px;
    border-radius: 4px;
    white-space: nowrap;
    pointer-events: none;
    z-index: 100;
  }

  @keyframes pulse-glow {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }
</style>
