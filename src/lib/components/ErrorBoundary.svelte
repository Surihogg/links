<script>
  import { onMount } from "svelte";
  import {
    dismissErrorNotice,
    errorNotice,
    initErrorMonitoring,
    reportFrontendError,
  } from "../monitoring/errorMonitor.svelte.js";

  let { component: Component } = $props();

  onMount(() => initErrorMonitoring());
</script>

<svelte:boundary onerror={(error) => reportFrontendError(error, "svelte.boundary", false)}>
  <Component />

  {#snippet failed(_error, reset)}
    <div class="error-fallback" role="alert">
      <div class="error-card">
        <div class="error-icon">☁</div>
        <h1>页面暂时打了个盹</h1>
        <p>你的链接数据还在。可以重新加载页面，错误详情已经记到日志里啦。</p>
        <div class="error-actions">
          <button type="button" class="btn btn-secondary" onclick={reset}>再试一次</button>
          <button type="button" class="btn btn-primary" onclick={() => window.location.reload()}>重新加载</button>
        </div>
      </div>
    </div>
  {/snippet}
</svelte:boundary>

{#if errorNotice.message}
  <div class="error-toast" role="status">
    <span>{errorNotice.message}</span>
    <button type="button" class="error-toast-close" onclick={dismissErrorNotice} aria-label="关闭提示">×</button>
  </div>
{/if}

<style>
  .error-fallback {
    min-height: 100vh;
    display: grid;
    place-items: center;
    padding: 24px;
    background: var(--bg-0);
    color: var(--text-0);
  }

  .error-card {
    max-width: 400px;
    text-align: center;
  }

  .error-icon {
    font-size: 36px;
    color: var(--accent);
  }

  .error-card h1 {
    margin: 12px 0 8px;
    font-size: 20px;
  }

  .error-card p {
    margin: 0;
    color: var(--text-2);
    font-size: 13px;
    line-height: 1.7;
  }

  .error-actions {
    display: flex;
    justify-content: center;
    gap: 8px;
    margin-top: 20px;
  }

  .error-toast {
    position: fixed;
    z-index: 10000;
    right: 20px;
    bottom: 20px;
    display: flex;
    align-items: center;
    gap: 12px;
    max-width: min(420px, calc(100vw - 40px));
    padding: 12px 14px;
    border: 1px solid var(--border-1);
    border-radius: var(--radius-md);
    background: var(--bg-1);
    color: var(--text-1);
    box-shadow: var(--shadow-md);
    font-size: 12px;
    line-height: 1.5;
  }

  .error-toast-close {
    flex: none;
    border: none;
    background: transparent;
    color: var(--text-2);
    font-size: 20px;
    line-height: 1;
    cursor: pointer;
  }
</style>
