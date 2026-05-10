<script>
  // 统计视图：链接总数、本周新增、最常访问 Top 3。
  // 从 App.svelte 抽出，模板与样式自包含；侧边栏点击"统计"时展示。
  // 数据由父组件通过 stats prop 注入；为空时显示加载占位。

  /**
   * @typedef {object} TopLink
   * @property {number} id
   * @property {string} title
   * @property {string} url
   * @property {number} click_count
   *
   * @typedef {object} Stats
   * @property {number} total
   * @property {number} this_week
   * @property {Array<TopLink>} top
   *
   * @typedef {object} Props
   * @property {Stats|null} [stats]
   */
  let { stats = null } = $props();
</script>

<div class="stats-panel">
  {#if stats}
    <div class="stats-overview">
      <div class="stat-card">
        <span class="stat-value">{stats.total}</span>
        <span class="stat-label">收藏总数</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">+{stats.this_week}</span>
        <span class="stat-label">本周新增</span>
      </div>
    </div>
    {#if stats.top.length > 0}
      <div class="stats-top-section">
        <h3 class="stats-section-title">最常访问</h3>
        <div class="stats-top-list">
          {#each stats.top as link, i}
            <div class="stats-top-row">
              <span class="stats-rank">{i + 1}</span>
              <div class="stats-top-info">
                <span class="stats-top-title">{link.title}</span>
                <span class="stats-top-url">{link.url}</span>
              </div>
              <span class="stats-top-count">
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>
                {link.click_count} 次
              </span>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {:else}
    <div class="stats-loading">加载中...</div>
  {/if}
</div>

<style>
  .stats-panel {
    flex: 1;
    padding: 24px;
    overflow-y: auto;
  }

  .stats-overview {
    display: flex;
    gap: 16px;
    margin-bottom: 28px;
  }

  .stat-card {
    flex: 1;
    background: var(--bg-1);
    border: 1px solid var(--border-0);
    border-radius: var(--radius-lg);
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .stat-value {
    font-size: 28px;
    font-weight: 700;
    color: var(--text-0);
    letter-spacing: -0.5px;
  }

  .stat-label {
    font-size: 12px;
    color: var(--text-3);
  }

  .stats-section-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-1);
    margin-bottom: 12px;
  }

  .stats-top-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stats-top-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: var(--bg-1);
    border: 1px solid var(--border-0);
    border-radius: var(--radius-md);
    transition: background var(--transition);
  }

  .stats-top-row:hover {
    background: var(--bg-hover);
  }

  .stats-rank {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    background: var(--accent-soft);
    color: var(--accent);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    font-weight: 700;
    flex-shrink: 0;
  }

  .stats-top-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stats-top-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-0);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .stats-top-url {
    font-size: 11px;
    color: var(--text-3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .stats-top-count {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-2);
    font-weight: 500;
    flex-shrink: 0;
  }

  .stats-loading {
    color: var(--text-3);
    font-size: 13px;
    padding: 40px 0;
    text-align: center;
  }
</style>
