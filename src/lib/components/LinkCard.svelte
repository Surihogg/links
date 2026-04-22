<script>
  import { openUrl } from "../api.js";

  let { link, highlight = "", category_name = null, onedit, ondelete, ontoggle_favorite } = $props();
  let show_confirm = $state(false);

  let domain = $derived.by(() => {
    try { return new URL(link.url).hostname.replace('www.', ''); } catch { return ''; }
  });

  function esc(s) {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

  function hl(text) {
    if (!highlight || !text) return esc(text);
    const safe = esc(text);
    const escaped = highlight.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    return safe.replace(new RegExp(`(${escaped})`, "gi"), '<span style="background:#fef08a;border-radius:2px;padding:0 2px;font-weight:600">$1</span>');
  }

  function card_click() {
    openUrl(link.url);
  }

  function toggle_fav(e) {
    e.stopPropagation();
    ontoggle_favorite?.(link);
  }

  function delete_link(e) {
    e.stopPropagation();
    show_confirm = true;
  }

  function confirm_delete() {
    show_confirm = false;
    ondelete?.(link);
  }

  function edit_link(e) {
    e.stopPropagation();
    e.preventDefault();
    onedit?.(link);
  }
</script>

<div class="link-card">
  <div class="card-main">
    <div class="card-content" onclick={card_click}>
      <div class="card-top">
        <div class="card-title-row">
          {#if link.favicon_url}
            <img src={link.favicon_url} alt="" class="favicon" onerror={(e) => e.target.style.display = 'none'} />
          {:else}
            <div class="favicon-ph">🔗</div>
          {/if}
          <span class="card-title">{@html hl(link.title || link.url)}</span>
        </div>
      </div>

      <div class="card-meta">
        <span class="card-domain">{domain}</span>
      </div>

      {#if link.description}
        <p class="card-desc">{@html hl(link.description)}</p>
      {/if}

      {#if category_name || link.tags.length > 0}
        <div class="card-tags">
          {#if category_name}
            <span class="cat-chip">{category_name}</span>
          {/if}
          {#each link.tags.slice(0, 5) as tag}
            <span class="tag-chip">{@html hl(tag)}</span>
          {/each}
          {#if link.tags.length > 5}
            <span class="tag-more">+{link.tags.length - 5}</span>
          {/if}
        </div>
      {/if}
    </div>

    <div class="card-actions">
      <button class="action-btn" class:active-fav={link.is_favorite} onclick={toggle_fav} title={link.is_favorite ? '取消特别关注' : '特别关注'}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill={link.is_favorite ? 'currentColor' : 'none'} stroke="currentColor" stroke-width="1.5">
          <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01z"/>
        </svg>
      </button>
      <button class="action-btn" onclick={edit_link} title="编辑">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M17 3l4 4L7 21H3v-4z"/>
        </svg>
      </button>
      <button class="action-btn danger" onclick={delete_link} title="删除">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M3 6h18M8 6V4a2 2 0 012-2h4a2 2 0 012 2v2M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6"/>
        </svg>
      </button>
    </div>
  </div>

  {#if show_confirm}
    <div class="confirm-overlay" onclick={() => show_confirm = false}>
      <div class="confirm-box" onclick={(e) => e.stopPropagation()}>
        <p class="confirm-text">确定要删除这个链接吗？</p>
        <div class="confirm-actions">
          <button class="confirm-btn cancel" onclick={() => show_confirm = false}>取消</button>
          <button class="confirm-btn delete" onclick={confirm_delete}>删除</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .link-card {
    padding: 0 24px;
    transition: background var(--transition);
  }

  .link-card:hover { background: var(--bg-hover); }

  .link-card:first-child .card-main { border-top: none; }

  .card-main {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 16px 0;
    border-top: 1px solid var(--border-0);
  }

  .link-card:hover .card-main { border-top-color: transparent; }

  .card-content {
    flex: 1;
    min-width: 0;
    cursor: pointer;
  }

  .card-top {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .card-title-row {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    flex: 1;
    overflow: hidden;
  }

  .favicon, .favicon-ph {
    width: 16px;
    height: 16px;
    border-radius: 3px;
    flex-shrink: 0;
  }

  .favicon-ph {
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 10px;
    background: var(--bg-2);
  }

  .card-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-0);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .card-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 2px;
    padding-left: 22px;
  }

  .card-domain { font-size: 11px; color: var(--text-3); }

  .card-desc {
    font-size: 12px;
    color: var(--text-2);
    line-height: 1.5;
    margin-top: 4px;
    padding-left: 22px;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }


  .card-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 6px;
    padding-left: 22px;
  }

  .cat-chip {
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 11px;
    background: #f0fdf4;
    color: #15803d;
    font-weight: 500;
  }

  :global(.dark) .cat-chip {
    background: #14532d;
    color: #86efac;
  }

  .tag-chip {
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 11px;
    background: var(--accent-soft);
    color: var(--accent-text);
    font-weight: 500;
  }

  .tag-more { font-size: 11px; color: var(--text-3); padding: 1px 4px; }

  .card-actions {
    display: flex;
    gap: 2px;
    flex-shrink: 0;
    padding-top: 2px;
  }

  .card-actions .action-btn {
    opacity: 0;
    transition: opacity var(--transition);
  }

  .link-card:hover .card-actions .action-btn { opacity: 1; }
  .card-actions .action-btn.active-fav { opacity: 1; color: #f59e0b; }

  .action-btn {
    width: 28px;
    height: 28px;
    border: none;
    background: none;
    border-radius: var(--radius-sm);
    color: var(--text-3);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition);
    text-decoration: none;
  }

  .action-btn:hover { background: var(--bg-2); color: var(--text-1); }
  .action-btn.danger:hover { background: var(--danger-soft); color: var(--danger); }

  .confirm-overlay {
    position: fixed;
    inset: 0;
    z-index: 60;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
  }

  .confirm-box {
    background: var(--bg-0);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-xl);
    border: 1px solid var(--border-0);
    padding: 20px;
    min-width: 280px;
  }

  .confirm-text {
    font-size: 13px;
    color: var(--text-0);
    margin-bottom: 16px;
  }

  .confirm-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .confirm-btn {
    padding: 6px 14px;
    border: none;
    border-radius: var(--radius-md);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--transition);
  }

  .confirm-btn.cancel {
    background: var(--bg-2);
    color: var(--text-2);
  }

  .confirm-btn.cancel:hover {
    background: var(--border-1);
    color: var(--text-1);
  }

  .confirm-btn.delete {
    background: var(--danger);
    color: white;
  }

  .confirm-btn.delete:hover {
    opacity: 0.9;
  }
</style>
