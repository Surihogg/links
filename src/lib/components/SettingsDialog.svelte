<script>
  import { onMount } from "svelte";
  import * as api from "../api.js";
  import { settingsStore } from "../stores/index.js";
  import ShortcutEditor from "./ShortcutEditor.svelte";

  let { onclose, onthemechange, oncheckupdate } = $props();
  let close_behavior = $state(null);
  let loaded = $state(false);
  // 外观（亮/暗/跟随系统）
  let appearance = $state("system");

  // 检查更新状态
  let checking_update = $state(false);
  let check_status = $state("检查是否有新版本可用");

  // 4 套快捷键：用通用 ShortcutEditor 组件，元数据驱动
  /** @type {Array<{key: string, name: string, desc: string, getter: () => Promise<string|null>, setter: (v: string) => Promise<string>}>} */
  const shortcuts = [
    { key: "quick", name: "快速添加", desc: "唤起快速添加窗口",
      getter: api.getShortcut, setter: api.setShortcut },
    { key: "main", name: "主窗口", desc: "唤起主窗口",
      getter: api.getMainShortcut, setter: api.setMainShortcut },
    { key: "spotlight", name: "全局搜索", desc: "唤起 Spotlight 搜索窗口",
      getter: api.getSpotlightShortcut, setter: api.setSpotlightShortcut },
    { key: "hide", name: "隐藏窗口", desc: "隐藏主程序窗口",
      getter: api.getHideShortcut, setter: api.setHideShortcut },
  ];
  /** ShortcutEditor 组件实例引用，用于把 window keydown 转发给正在录制的那个 */
  const shortcutRefs = $state({});

  let autostart_enabled = $state(false);
  let autostart_loaded = $state(false);
  let auto_minimize = $state(false);
  // 启动时自动检查更新
  let auto_check_update = $state(true);
  let check_link_reachability = $state(true);
  let bookmarklet_copied = $state(false);
  let bookmarklet_code = $state("");
  let ext_step = $state(0);

  onMount(async () => {
    const val = await api.getSetting("close-behavior");
    close_behavior = val || "ask";

    // Load autostart state
    try {
      autostart_enabled = await api.isAutostartEnabled();
    } catch {
      autostart_enabled = false;
    }
    autostart_loaded = true;
    // Load auto-minimize-on-open setting
    try {
      auto_minimize = (await api.getSetting("auto-minimize-on-open")) === "true";
    } catch {
      auto_minimize = false;
    }
    // 启动时自动检查更新
    try {
      auto_check_update = (await api.getSetting("auto-check-update")) !== "false";
    } catch {
      auto_check_update = true;
    }
    // 链接可达性检查
    try {
      check_link_reachability = (await api.getSetting("check-link-reachability")) !== "false";
      settingsStore.update(s => ({ ...s, check_link_reachability: check_link_reachability }));
    } catch {
      check_link_reachability = true;
    }
    loaded = true;
    try {
      const t = await api.getSetting("theme-mode");
      appearance = t || "system";
    } catch {
      appearance = "system";
    }
    // 拉取本地 HTTP 服务端口与 token，构造 Bookmarklet
    // 优先 fetch 本地服务（无频率限制），失败时降级到 links:// 自定义协议
    try {
      const info = await api.getLocalServerInfo();
      bookmarklet_code = build_bookmarklet(info.port, info.token);
    } catch {
      bookmarklet_code = build_bookmarklet(0, "");
    }
  });

  function build_bookmarklet(port, token) {
    const has_local = port > 0 && token.length > 0;
    const endpoint = has_local
      ? `http://127.0.0.1:${port}/add`
      : "";
    const tok = token;
    // HTTP 优先策略：应用运行时走 fetch（不触发 OS 协议，避免 macOS 激活主窗口），
    // fetch 失败时回退到 iframe 深链接并延迟提示用户；无 HTTP 端点时仅走深链接（冷启动唤起）。
    // 后端 PendingDeepLink 用 take() 只消费一次，不会重复触发
    return (
      "javascript:void(function(){" +
      "var u=encodeURIComponent(location.href)," +
      "t=encodeURIComponent(document.title)," +
      "ts=Date.now();" +
      "function dl(){try{var i=document.createElement('iframe');i.hidden=1;" +
      "i.src='links://add?url='+u+'&title='+t+'&_t='+ts;" +
      "document.body.appendChild(i);" +
      "setTimeout(function(){i.remove()},2e3);}catch(e){}}" +
      // 页面内 toast 提示：CSP 拦截或应用未运行时告知用户
      "function tip(m){var d=document.createElement('div');d.textContent=m;" +
      "d.style.cssText='position:fixed;top:16px;right:16px;z-index:2147483647;padding:12px 20px;" +
      "background:#1e1e2e;color:#cdd6f4;border-radius:8px;font:14px/1.5 system-ui,sans-serif;" +
      "box-shadow:0 4px 16px rgba(0,0,0,.3);max-width:340px;cursor:pointer;transition:opacity .3s';" +
      "d.onclick=function(){d.remove()};" +
      "document.body.appendChild(d);" +
      "setTimeout(function(){d.style.opacity='0';setTimeout(function(){d.remove()},300)},5e3)}" +
      "var ep=" + JSON.stringify(endpoint) + ",tk=" + JSON.stringify(tok) + ";" +
      "if(ep){" +
      "fetch(ep+'?url='+u+'&title='+t+'&t='+tk+'&_='+ts)" +
      ".then(function(r){if(!r.ok)throw new Error()})" +
      ".catch(function(){dl();setTimeout(function(){" +
      "tip('Links：这个网站可能把咱拦截了T.T，试试快捷键添加链接吧~')" +
      "},500)});" +
      "}else{" +
      "dl();" +
      "setTimeout(function(){" +
      "tip('Links：最好开启Links喔~')" +
      "},1e3);" +
      "}" +
      "}())"
    );
  }

  const behaviors = [
    { id: "ask", name: "每次询问", desc: "关闭时弹出对话框选择" },
    { id: "tray", name: "最小化到托盘", desc: "关闭窗口后继续在后台运行" },
    { id: "exit", name: "直接退出", desc: "关闭窗口时直接退出应用" },
  ];

  // Appearance options (theme mode)
  const appearances = [
    { id: "light", name: "亮色", desc: "始终使用亮色主题" },
    { id: "dark", name: "暗色", desc: "始终使用暗色主题" },
    { id: "system", name: "跟随系统", desc: "自动匹配系统外观设置" },
  ];

  async function on_theme_select(mode) {
    appearance = mode;
    await api.setSetting("theme-mode", mode);
    onthemechange?.(mode);
  }

  async function save() {
    await api.setSetting("close-behavior", close_behavior);
    onclose?.();
  }

  async function toggleAutostart() {
    if (!autostart_loaded) return;
    try {
      if (autostart_enabled) {
        await api.disableAutostart();
        autostart_enabled = false;
      } else {
        await api.enableAutostart();
        autostart_enabled = true;
      }
      // Persist the autostart state to config.json for consistency
      await api.setSetting("autostart-enabled", autostart_enabled);
    } catch {
      // swallow errors; user can retry
    }
  }

  async function toggleAutoMinimize() {
    auto_minimize = !auto_minimize;
    await api.setSetting("auto-minimize-on-open", String(auto_minimize));
  }

  async function toggleAutoCheckUpdate() {
    auto_check_update = !auto_check_update;
    await api.setSetting("auto-check-update", String(auto_check_update));
  }

  async function toggleCheckLinkReachability() {
    check_link_reachability = !check_link_reachability;
    await api.setSetting("check-link-reachability", String(check_link_reachability));
    settingsStore.update(s => ({ ...s, check_link_reachability: check_link_reachability }));
  }

  async function copy_bookmarklet() {
    try {
      await navigator.clipboard.writeText(bookmarklet_code);
      bookmarklet_copied = true;
      setTimeout(() => bookmarklet_copied = false, 2000);
    } catch {
      // 剪贴板失败时静默处理
    }
  }

  async function do_check_update() {
    checking_update = true;
    check_status = "正在检查...";
    try {
      const update = await api.checkUpdate();
      if (update) {
        if (oncheckupdate) await oncheckupdate(update);
        check_status = "发现新版本！";
      } else {
        check_status = "已是最新版本 ✨";
      }
    } catch (e) {
      const msg = e?.message || String(e);
      if (msg.includes("404") || msg.includes("Not Found")) {
        check_status = "暂无可用更新（未检测到发布版本）";
      } else if (msg.includes("network") || msg.includes("Failed to fetch") || msg.includes("fetch") || msg.includes("超时")) {
        check_status = "网络连接失败，请检查网络后重试";
      } else {
        check_status = "检查失败，请稍后重试";
      }
    }
    checking_update = false;
  }

  async function open_data_dir() {
    try {
      await api.openDataDir();
    } catch {
      // silently fail
    }
  }

  function on_overlay_click(e) {
    if (e.target === e.currentTarget) onclose?.();
  }

  function handle_window_keydown(e) {
    if (e.key === "Escape") {
      onclose?.();
      return;
    }
    // 转发给所有 ShortcutEditor，组件内部按 recording 状态自行决定是否消费
    for (const s of shortcuts) {
      if (shortcutRefs[s.key]?.handleKeydown(e)) break;
    }
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

        <div class="section-label" style="margin-top: 16px;">快捷键</div>
        <div class="shortcut-section">
          {#each shortcuts as s (s.key)}
            <ShortcutEditor
              bind:this={shortcutRefs[s.key]}
              name={s.name}
              desc={s.desc}
              getter={s.getter}
              setter={s.setter}
            />
          {/each}
        </div>

        <div class="section-label" style="margin-top: 20px;">外观</div>
        {#if loaded}
          <div class="format-list">
            {#each appearances as a}
              <button
                class="format-option"
                class:active={appearance === a.id}
                onclick={() => on_theme_select(a.id)}
              >
                <div class="format-radio">
                  {#if appearance === a.id}
                    <div class="format-dot"></div>
                  {/if}
                </div>
                <div class="format-info">
                  <span class="format-name">{a.name}</span>
                  <span class="format-desc">{a.desc}</span>
                </div>
                {#if appearance === a.id}
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

        <div class="section-label" style="margin-top: 20px;">通用</div>
        {#if autostart_loaded}
          <div class="format-option autostart-row" style="justify-content: space-between; align-items: center;">
            <div class="autostart-info" style="display:flex; flex-direction:column; gap:2px; min-width:0;">
              <span class="format-name">开机自启动</span>
              <span class="format-desc">系统启动时自动运行 Links</span>
            </div>
            <button class="toggle" class:active={autostart_enabled} onclick={toggleAutostart}></button>
          </div>
          <div class="format-option autostart-row" style="justify-content: space-between; align-items: center; margin-top: 6px;">
            <div style="display:flex; flex-direction:column; gap:2px; min-width:0;">
              <span class="format-name">打开链接后最小化</span>
              <span class="format-desc">点击链接跳转时自动最小化到托盘</span>
            </div>
            <button class="toggle" class:active={auto_minimize} onclick={toggleAutoMinimize}></button>
          </div>
          <div class="format-option autostart-row" style="justify-content: space-between; align-items: center; margin-top: 6px;">
            <div style="display:flex; flex-direction:column; gap:2px; min-width:0;">
              <span class="format-name">启动时自动检查更新</span>
              <span class="format-desc">每次打开应用时自动检测是否有新版本</span>
            </div>
            <button class="toggle" class:active={auto_check_update} onclick={toggleAutoCheckUpdate}></button>
          </div>
          <div class="format-option autostart-row" style="justify-content: space-between; align-items: center; margin-top: 6px;">
            <div style="display:flex; flex-direction:column; gap:2px; min-width:0;">
              <span class="format-name">链接可达性检查</span>
              <span class="format-desc">添加链接时检测网址是否可正常访问</span>
            </div>
            <button class="toggle" class:active={check_link_reachability} onclick={toggleCheckLinkReachability}></button>
          </div>
          <div class="format-option autostart-row" style="justify-content: space-between; align-items: center; margin-top: 6px;">
            <div style="display:flex; flex-direction:column; gap:2px; min-width:0;">
              <span class="format-name">手动检查更新</span>
              <span class="format-desc">{check_status}</span>
            </div>
            <button class="btn btn-secondary btn-sm" onclick={do_check_update} disabled={checking_update}>
              {checking_update ? '检查中...' : '检查'}
            </button>
          </div>
          <div class="format-option autostart-row" style="justify-content: space-between; align-items: center; margin-top: 6px;">
            <div style="display:flex; flex-direction:column; gap:2px; min-width:0;">
              <span class="format-name">数据文件夹</span>
              <span class="format-desc">在文件管理器中打开数据库和配置文件所在目录</span>
            </div>
            <button class="btn btn-secondary btn-sm" onclick={open_data_dir}>打开</button>
          </div>
        {:else}
          <div class="format-loading">加载中...</div>
        {/if}

        <div class="section-label" style="margin-top: 20px;">浏览器扩展</div>
        <div class="ext-section">
          <div class="ext-desc">
            安装浏览器扩展后，在任意网页点击扩展图标即可一键收藏到 Links，比书签更方便 ✨
          </div>
          <div class="ext-steps">
            {#if ext_step === 0}
              <div class="ext-step">
                <span class="ext-step-num">1</span>
                <span>从 <button type="button" onclick={() => api.openUrl('https://github.com/Surihogg/links/releases')} class="ext-link">GitHub Releases</button> 下载浏览器扩展 zip 包</span>
              </div>
              <div class="ext-step">
                <span class="ext-step-num">2</span>
                <span>解压 zip 到任意文件夹</span>
              </div>
              <div class="ext-step">
                <span class="ext-step-num">3</span>
                <span>打开 <code>chrome://extensions</code>，开启右上角「开发者模式」</span>
              </div>
              <div class="ext-step">
                <span class="ext-step-num">4</span>
                <span>点击「加载已解压的扩展程序」，选择解压后的文件夹</span>
              </div>
            {:else}
              <div class="ext-step">
                <span class="ext-step-num">✓</span>
                <span>扩展已安装，在任意网页点击工具栏的 Links 图标即可收藏</span>
              </div>
            {/if}
          </div>
          <button class="btn btn-secondary btn-sm" onclick={() => ext_step = ext_step === 0 ? 1 : 0}>
            {ext_step === 0 ? '我已安装，隐藏步骤' : '查看安装步骤'}
          </button>
        </div>

        <div class="section-label" style="margin-top: 20px;">浏览器收藏</div>
        <div class="bookmarklet-section">
          <div class="bookmarklet-desc">
            将下方按钮拖拽到浏览器书签栏，即可在任何网页一键收藏到 Links ✨
          </div>
          <div class="bookmarklet-row">
            <a
              class="bookmarklet-btn"
              class:disabled={!bookmarklet_code}
              href={bookmarklet_code || "#"}
              title={bookmarklet_code ? "拖拽我到书签栏" : "正在准备..."}
              onclick={(e) => { e.preventDefault(); }}
            >
              扔给Links
            </a>
            <button class="btn btn-secondary btn-sm" onclick={copy_bookmarklet} disabled={!bookmarklet_code}>
              {bookmarklet_copied ? '已复制 ✓' : '复制代码'}
            </button>
          </div>
          <div class="bookmarklet-tip">
            💡 在浏览器书签栏里点击“扔给Links”会跳转到快速添加；Links未运行时会自动唤起应用<p>
            （但由于浏览器有安全限制，限定了一定时间内无法连续触发应用冷启动，有概率出现点击书签后没反应，为了更好的体验，建议您启动应用后保持常驻）
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button onclick={onclose} class="btn btn-secondary">取消</button>
        <button onclick={save} class="btn btn-primary">保存</button>
      </div>
    </div>
</div>

<style>
  .modal {
    max-width: 400px;
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

  .format-loading {
    font-size: 13px;
    color: var(--text-3);
    padding: 16px 0;
    text-align: center;
  }

  /* shortcut-section: 容器；具体 row 样式在 ShortcutEditor 内 */
  .shortcut-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-bottom: 16px;
  }

  .bookmarklet-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-bottom: 16px;
  }

  .bookmarklet-desc {
    font-size: 12px;
    color: var(--text-3);
    line-height: 1.5;
  }

  .bookmarklet-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .bookmarklet-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    border: 1px solid var(--accent);
    border-radius: var(--radius-md);
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 13px;
    font-weight: 500;
    text-decoration: none;
    cursor: grab;
    transition: all var(--transition);
    white-space: nowrap;
  }

  .bookmarklet-btn:hover {
    background: var(--accent);
    color: white;
  }

  .bookmarklet-btn:active {
    cursor: grabbing;
  }

  .bookmarklet-btn.disabled {
    opacity: 0.5;
    cursor: wait;
    pointer-events: none;
  }

  .bookmarklet-tip {
    font-size: 11px;
    color: var(--text-3);
    line-height: 1.4;
  }

  .ext-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .ext-desc {
    font-size: 12px;
    color: var(--text-2);
    line-height: 1.6;
  }

  .ext-steps {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .ext-step {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    font-size: 12px;
    color: var(--text-1);
    line-height: 1.6;
  }

  .ext-step-num {
    flex-shrink: 0;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 11px;
    font-weight: 600;
  }

  .ext-link {
    color: var(--accent);
    font-size: inherit;
    font-family: inherit;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    text-decoration: underline;
  }

  .ext-link:hover {
    opacity: 0.8;
  }
</style>
