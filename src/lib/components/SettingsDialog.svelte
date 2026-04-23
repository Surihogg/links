<script>
  import { onMount } from "svelte";
  import * as api from "../api.js";
  let { onclose } = $props();
  let close_behavior = $state(null);
  let loaded = $state(false);

  let shortcut_raw = $state(null);
  let shortcut_loaded = $state(false);
  let recording = $state(false);
  let recorded_shortcut = $state(null);
  let shortcut_error = $state(false);

  const isMac = /mac/i.test(navigator.userAgentData?.platform ?? navigator.platform);

  function formatShortcut(raw) {
    if (!raw) return "";
    const parts = raw.split("+");
    const map = isMac
      ? { super: "\u2318", cmdorctrl: "\u2318", control: "\u2325", alt: "\u2325", shift: "\u21E7" }
      : { super: "Win", cmdorctrl: "Ctrl", control: "Ctrl", alt: "Alt", shift: "Shift" };
    const separator = isMac ? " " : "+";
    return parts.map((p) => {
      const lower = p.toLowerCase();
      return map[lower] ?? p.toUpperCase();
    }).join(separator);
  }

  function buildShortcutFromEvent(e) {
    const modifiers = [];
    if (e.metaKey || e.ctrlKey) modifiers.push("CmdOrCtrl");
    if (e.shiftKey) modifiers.push("Shift");
    if (e.altKey) modifiers.push("Alt");
    const ignore = ["Meta", "Control", "Shift", "Alt"];
    if (ignore.includes(e.key)) return null;
    if (modifiers.length === 0) return null;
    const key = e.key.length === 1 ? e.key.toUpperCase() : e.key;
    return [...modifiers, key].join("+");
  }

  function on_record_keydown(e) {
    e.preventDefault();
    e.stopPropagation();
    const result = buildShortcutFromEvent(e);
    if (result) recorded_shortcut = result;
  }

  function start_recording() {
    recorded_shortcut = null;
    shortcut_error = false;
    recording = true;
  }

  function cancel_recording() {
    recorded_shortcut = null;
    recording = false;
    shortcut_error = false;
  }

  async function save_shortcut() {
    if (!recorded_shortcut) return;
    shortcut_error = false;
    try {
      const result = await api.setShortcut(recorded_shortcut);
      shortcut_raw = result;
      recording = false;
      recorded_shortcut = null;
    } catch {
      shortcut_error = true;
      recorded_shortcut = null;
    }
  }

  onMount(async () => {
    const val = await api.getSetting("close-behavior");
    close_behavior = val || "ask";

    try {
      shortcut_raw = await api.getShortcut();
    } catch {
      shortcut_raw = null;
    }
    shortcut_loaded = true;
    loaded = true;
  });

  const behaviors = [
    { id: "ask", name: "每次询问", desc: "关闭时弹出对话框选择" },
    { id: "tray", name: "最小化到托盘", desc: "关闭窗口后继续在后台运行" },
    { id: "exit", name: "直接退出", desc: "关闭窗口时直接退出应用" },
  ];

  async function save() {
    await api.setSetting("close-behavior", close_behavior);
    onclose?.();
  }

  function on_overlay_click(e) {
    if (e.target === e.currentTarget) onclose?.();
  }

  function handle_window_keydown(e) {
    if (recording) on_record_keydown(e);
  }
</script>

<svelte:window onkeydown={handle_window_keydown} />
<div class="modal-overlay" onclick={on_overlay_click}>
  <div class="modal">
    <div class="modal-header">
      <h2 class="modal-title">设置</h2>
      <button class="modal-close" onclick={onclose}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
        </svg>
      </button>
    </div>

    <div class="modal-body">
      <div class="section-label">关闭行为</div>
      {#if loaded}
        <div class="format-list">
          {#each behaviors as b (b.id)}
            <button
              class="format-option"
              class:active={close_behavior === b.id}
              onclick={() => close_behavior = b.id}
            >
              <div class="format-radio">
                {#if close_behavior === b.id}
                  <div class="format-dot"></div>
                {/if}
              </div>
              <div class="format-info">
                <span class="format-name">{b.name}</span>
                <span class="format-desc">{b.desc}</span>
              </div>
              {#if close_behavior === b.id}
                <svg class="format-check" width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M3 8l3.5 3.5L13 5"/>
                </svg>
              {/if}
            </button>
          {/each}
        </div>
      {:else}
        <div class="format-loading">加载中...</div>
      {/if}

      <div class="section-label" style="margin-top: 20px;">快捷键</div>
      {#if shortcut_loaded}
        <div class="shortcut-section">
          <div class="shortcut-row">
            <div class="shortcut-info">
              <span class="format-name">全局快捷键</span>
              <span class="format-desc">用于唤起快速添加窗口</span>
            </div>
            {#if !recording}
              <div class="shortcut-display">{formatShortcut(shortcut_raw) || "未设置"}</div>
              <button class="btn btn-secondary btn-sm" onclick={start_recording}>修改</button>
            {:else}
              <div class="shortcut-display recording-area">
                {#if recorded_shortcut}
                  {formatShortcut(recorded_shortcut)}
                {:else}
                  请按下新的快捷键...
                {/if}
              </div>
              {#if shortcut_error}
                <span class="shortcut-error">快捷键设置失败，请重试</span>
              {/if}
              <div class="shortcut-actions">
                <button class="btn btn-secondary btn-sm" onclick={cancel_recording}>取消</button>
                <button class="btn btn-primary btn-sm" disabled={!recorded_shortcut} onclick={save_shortcut}>保存</button>
              </div>
            {/if}
          </div>
        </div>
      {:else}
        <div class="format-loading">加载中...</div>
      {/if}

      <div class="modal-footer">
        <button onclick={onclose} class="btn btn-secondary">取消</button>
        <button onclick={save} class="btn btn-primary">保存</button>
      </div>
    </div>
  </div>
</div>

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
  }

  .modal {
    width: 100%;
    max-width: 400px;
    background: var(--bg-0);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-xl);
    border: 1px solid var(--border-0);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px 12px;
  }

  .modal-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-0);
  }

  .modal-close {
    width: 28px;
    height: 28px;
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

  .modal-close:hover {
    background: var(--bg-2);
    color: var(--text-1);
  }

  .modal-body {
    padding: 0 20px 20px;
  }

  .section-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-3);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 8px;
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

  .btn {
    padding: 7px 16px;
    border: none;
    border-radius: var(--radius-md);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--transition);
  }

  .btn-primary {
    background: var(--accent);
    color: white;
  }

  .btn-primary:hover {
    background: var(--accent-hover);
  }

  .btn-secondary {
    background: var(--bg-2);
    color: var(--text-2);
  }

  .btn-secondary:hover {
    background: var(--border-1);
    color: var(--text-1);
  }

  .format-loading {
    font-size: 13px;
    color: var(--text-3);
    padding: 16px 0;
    text-align: center;
  }

  .shortcut-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 16px;
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border: 1px solid var(--border-1);
    border-radius: var(--radius-md);
    background: var(--bg-1);
    flex-wrap: wrap;
  }

  .shortcut-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .shortcut-display {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-1);
    padding: 4px 10px;
    border: 1px solid var(--border-1);
    border-radius: var(--radius-sm);
    background: var(--bg-2);
    white-space: nowrap;
    letter-spacing: 0.5px;
    font-family: var(--font);
  }

  .shortcut-display.recording-area {
    border-color: var(--accent);
    background: var(--accent-soft);
    color: var(--accent-text);
    min-width: 140px;
    text-align: center;
  }

  .shortcut-actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }

  .shortcut-error {
    font-size: 11px;
    color: var(--danger);
    width: 100%;
  }

  .btn-sm {
    padding: 4px 12px;
    font-size: 12px;
  }
</style>
