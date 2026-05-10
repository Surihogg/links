<script>
  // 单条快捷键的展示 + 录制 + 保存。
  // 替代 SettingsDialog 中 4 套（quick-add / main / spotlight / hide）逐字重复
  // 的状态、handler、模板（约 200+ 行重复代码）。
  //
  // 调用方提供 getter / setter；本组件不直接依赖具体的 api 函数名，便于复用。

  /**
   * @typedef {object} Props
   * @property {string} name 快捷键的功能名（"快速添加" / "主窗口" / ...）
   * @property {string} desc 描述文案
   * @property {() => Promise<string|null>} getter 加载当前快捷键
   * @property {(value: string) => Promise<string>} setter 保存新快捷键，返回规范化后的字符串
   */

  let { name, desc, getter, setter } = $props();

  let raw = $state(null);
  let loaded = $state(false);
  let recording = $state(false);
  let recorded = $state(null);
  let error = $state(false);

  const isMac = /mac/i.test(navigator.userAgentData?.platform ?? navigator.platform);

  function format(rawValue) {
    if (!rawValue) return "";
    const parts = rawValue.split("+");
    const map = isMac
      ? { super: "\u2318", cmdorctrl: "\u2318", control: "\u2325", alt: "\u2325", shift: "\u21E7" }
      : { super: "Win", cmdorctrl: "Ctrl", control: "Ctrl", alt: "Alt", shift: "Shift" };
    const sep = isMac ? " " : "+";
    return parts.map((p) => map[p.toLowerCase()] ?? p.toUpperCase()).join(sep);
  }

  function buildFromEvent(e) {
    const mods = [];
    if (e.metaKey || e.ctrlKey) mods.push("CmdOrCtrl");
    if (e.shiftKey) mods.push("Shift");
    if (e.altKey) mods.push("Alt");
    const ignore = ["Meta", "Control", "Shift", "Alt"];
    if (ignore.includes(e.key)) return null;
    if (mods.length === 0) return null;
    const key = e.key.length === 1 ? e.key.toUpperCase() : e.key;
    return [...mods, key].join("+");
  }

  /** 父组件转发 keydown 时调用此方法消费事件。返回 true 表示已处理。 */
  export function handleKeydown(e) {
    if (!recording) return false;
    e.preventDefault();
    e.stopPropagation();
    const r = buildFromEvent(e);
    if (r) recorded = r;
    return true;
  }

  function startRecording() {
    recorded = null;
    error = false;
    recording = true;
  }

  function cancelRecording() {
    recorded = null;
    recording = false;
    error = false;
  }

  async function save() {
    if (!recorded) return;
    error = false;
    try {
      raw = await setter(recorded);
      recording = false;
      recorded = null;
    } catch {
      error = true;
      recorded = null;
    }
  }

  $effect(() => {
    (async () => {
      try {
        raw = await getter();
      } catch {
        raw = null;
      }
      loaded = true;
    })();
  });
</script>

<div class="shortcut-row">
  <div class="shortcut-info">
    <span class="format-name">{name}</span>
    <span class="format-desc">{desc}</span>
  </div>
  {#if !loaded}
    <span class="format-desc">加载中…</span>
  {:else if !recording}
    <div class="shortcut-display">{format(raw) || "未设置"}</div>
    <button class="btn btn-secondary btn-sm" onclick={startRecording}>修改</button>
  {:else}
    <div class="shortcut-display recording-area">
      {#if recorded}
        {format(recorded)}
      {:else}
        请按下新的快捷键...
      {/if}
    </div>
    {#if error}
      <span class="shortcut-error">快捷键设置失败，请重试</span>
    {/if}
    <div class="shortcut-actions">
      <button class="btn btn-secondary btn-sm" onclick={cancelRecording}>取消</button>
      <button class="btn btn-primary btn-sm" disabled={!recorded} onclick={save}>保存</button>
    </div>
  {/if}
</div>

<style>
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

  .format-name {
    font-weight: 500;
    font-size: 13px;
  }

  .format-desc {
    font-size: 11px;
    color: var(--text-3);
  }
</style>
