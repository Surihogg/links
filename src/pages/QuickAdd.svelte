<script>
  // 快速添加窗口（独立 Tauri 窗口，全局快捷键唤起）。
  // 表单本体复用 LinkForm 的 standalone 模式，QuickAdd 只负责窗口生命周期：
  // 1. 主题加载（themeStore.init）
  // 2. quick-add-shown 事件后 reset 表单 + 拉取 deep link
  // 3. 创建链接后 emit("links-changed") 并 hide 窗口
  // 4. Esc 关窗、IME 守卫由 LinkForm 内部处理

  import { onMount } from "svelte";
  import LinkForm from "../lib/components/LinkForm.svelte";
  import {
    createLink,
    listCategories,
    getSetting,
    popPendingDeepLink,
  } from "../lib/api.js";
  import { waitForBackendReady } from "../lib/ready.js";
  import { themeStore } from "../lib/stores/themeStore.svelte.js";
  import { emit, listen } from "@tauri-apps/api/event";

  let categories = $state([]);
  let message = $state("");
  /** @type {LinkForm | undefined} */
  let form;

  /** 用 deep-link 数据预填表单 */
  async function fill_from_pending() {
    try {
      const pending = await popPendingDeepLink();
      if (pending?.url) {
        form?.reset({ url: pending.url, title: pending.title || "" });
        // 主动触发去重检测 + 元数据抓取（reset 不会派发 input 事件）
        form?.triggerFetch();
      }
    } catch {
      // 无 pending 则忽略
    }
  }

  async function close_window() {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await emit("links-changed");
    await getCurrentWindow().hide();
    // 窗口隐藏后再清空表单，避免下次 show 时视觉残留（show → emit 异步到达有延迟）
    form?.reset();
  }

  async function on_save(payload) {
    message = "";
    try {
      await createLink({
        url: payload.url,
        title: payload.title,
        description: payload.description,
        notes: payload.notes,
        category_id: payload.category_id,
        tags: payload.tags,
        favicon_url: payload.favicon_url,
        og_image_url: payload.og_image_url,
      });
      // 保存成功，通知主程序刷新，关闭窗口
      await emit("links-changed");
      await close_window();
    } catch {
      message = "保存失败 ✗";
      form?.setSaving(false);
    }
  }

  onMount(async () => {
    await waitForBackendReady();
    await themeStore.init();

    listCategories().then((c) => (categories = c));

    const unlistenShown = await listen("quick-add-shown", async () => {
      message = "";
      form?.reset();
      await fill_from_pending();
      form?.focusUrl();
    });

    // 冷启动兜底：前端就绪后主动拉一次 deep link
    await fill_from_pending();

    const handle_keydown = (e) => {
      if (e.key === "Escape") {
        e.preventDefault();
        close_window();
      }
    };
    window.addEventListener("keydown", handle_keydown);
    return () => {
      window.removeEventListener("keydown", handle_keydown);
      unlistenShown();
    };
  });
</script>

<div class="quick-add">
  <div class="modal-header" data-tauri-drag-region>
    <h2 class="modal-title">添加链接</h2>
    <button class="modal-close" onclick={close_window} aria-label="关闭">
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
        <line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/>
      </svg>
    </button>
  </div>

  <div class="quick-add-body">
    <LinkForm
      bind:this={form}
      mode="standalone"
      {categories}
      onsave={on_save}
      oncancel={close_window}
    />
    {#if message}
      <div class="message-bar">{message}</div>
    {/if}
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    height: 100vh;
    width: 100vw;
  }

  .quick-add {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background: var(--bg-0);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px 12px;
    flex-shrink: 0;
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

  .quick-add-body {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    padding: 0 20px 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  /* standalone 模式下让 LinkForm 撑满 quick-add-body：
     表单整体填满 → footer 自然贴底；
     描述/备注两个 textarea 占据中间剩余空间，避免下方留白也避免出现滚动条。
     注意：LinkForm 是共享组件（modal 模式也用），所以这里只通过 :global 选择器
     在 standalone 模式（QuickAdd 专用）下覆写布局。 */
  .quick-add-body :global(.form-body.standalone) {
    flex: 1;
    min-height: 0;
    /* 行：url / title+category / tags / desc / notes / footer
       desc 与 notes 用 1fr 平分剩余空间 */
    grid-template-rows: auto auto auto 1fr 1fr auto;
  }

  .quick-add-body :global(.form-body.standalone .desc-field),
  .quick-add-body :global(.form-body.standalone .notes-field) {
    min-height: 0;
  }

  .quick-add-body :global(.form-body.standalone .desc-field .field-textarea),
  .quick-add-body :global(.form-body.standalone .notes-field .field-textarea) {
    flex: 1;
    min-height: 0;
    height: 100%;
    resize: none;
  }

  .message-bar {
    font-size: 12px;
    color: var(--danger);
    padding: 6px 0;
  }
</style>
