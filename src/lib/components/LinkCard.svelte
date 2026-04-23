<script>
  import { openUrl } from "../api.js";

  let { link, highlight = "", category_name = null, onedit, ondelete, ontoggle_favorite } = $props();
  let show_confirm = $state(false);

  let show_full_url = $state(false);
  let title_truncated = $state(false);
  let desc_truncated = $state(false);

  let domain = $derived.by(() => {
    try { return new URL(link.url).hostname.replace('www.', ''); } catch { return ''; }
  });

  let url_matches_search = $derived.by(() => {
    if (!highlight) return false;
    try { return link.url.toLowerCase().includes(highlight.toLowerCase()); } catch { return false; }
  });

  let display_full_url = $derived(show_full_url || url_matches_search);

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

  function check_title_overflow(e) {
    const el = e.target;
    title_truncated = el.scrollWidth > el.clientWidth;
  }

  function check_desc_overflow(e) {
    const el = e.target;
    desc_truncated = el.scrollHeight > el.clientHeight;
  }
</script>

<div class="link-card">
  <div class="card-main">
    <div class="card-content" onclick={card_click}>
      <div class="card-top">
        <div class="card-title-row" data-tooltip={title_truncated ? (link.title || link.url) : undefined}>
          {#if link.favicon_url}
            <img src={link.favicon_url} alt="" class="favicon" onerror={(e) => e.target.style.display = 'none'} />
          {:else}
            <div class="favicon-ph">🔗</div>
          {/if}
          <div class="card-title" onmouseenter={check_title_overflow}>{@html hl(link.title || link.url)}</div>
        </div>
      </div>

      <div class="card-meta">
        {#if display_full_url}
          <span class="card-url-full" onmouseout={() => { if (!url_matches_search) show_full_url = false; }}>{@html hl(link.url)}</span>
        {:else}
          <span class="card-domain" onmouseover={() => show_full_url = true}>{domain}</span>
        {/if}
      </div>

      {#if link.description}
        <div class="card-desc-wrap" data-tooltip={desc_truncated ? link.description : undefined}>
          <p class="card-desc" onmouseenter={check_desc_overflow}>{@html hl(link.description)}</p>
        </div>
      {/if}

      {#if category_name || link.tags.length > 0 || link.notes}
        <div class="card-tags">
          {#if category_name}
            <span class="cat-chip">
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7V17C3 18.1046 3.89543 19 5 19H19C20.1046 19 21 18.1046 21 17V9C21 7.89543 20.1046 7 19 7H13L11 5H5C3.89543 5 3 5.89543 3 7Z"/></svg>
              {category_name}
            </span>
          {/if}
          {#each link.tags.slice(0, 5) as tag}
            <span class="tag-chip">
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>
              {@html hl(tag)}
            </span>
          {/each}
          {#if link.tags.length > 5}
            <span class="tag-more">+{link.tags.length - 5}</span>
          {/if}
          {#if link.notes}
            <span class="note-chip">
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
              {@html hl(link.notes)}
            </span>
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
        <p class="confirm-text">确定要和这个链接说再见吗？</p>
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
    position: relative;
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
    position: relative;
  }

  .card-title-row[data-tooltip]:hover::after,
  .card-desc-wrap[data-tooltip]:hover::after {
    content: attr(data-tooltip);
    position: absolute;
    left: 0;
    top: 100%;
    z-index: 30;
    max-width: 360px;
    padding: 6px 10px;
    background: var(--bg-0);
    border: 1px solid var(--border-1);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    font-size: 12px;
    font-weight: 400;
    color: var(--text-1);
    line-height: 1.5;
    white-space: normal;
    word-break: break-all;
    pointer-events: none;
  }

  .card-title-row {
    position: relative;
  }

  .card-desc-wrap {
    position: relative;
  }

  .card-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 2px;
    padding-left: 22px;
    position: relative;
    min-height: 16px;
  }

  .card-domain { font-size: 11px; color: var(--text-3); cursor: default; }

  .card-url-full {
    font-size: 11px;
    color: var(--text-3);
    word-break: break-all;
    line-height: 1.4;
  }

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
    display: inline-flex;
    align-items: center;
    gap: 3px;
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
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 11px;
    background: var(--accent-soft);
    color: var(--accent-text);
    font-weight: 500;
  }

  .tag-more { font-size: 11px; color: var(--text-3); padding: 1px 4px; }

  .note-chip {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 11px;
    background: #fef2f2;
    color: #dc2626;
    font-weight: 500;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  :global(.dark) .note-chip {
    background: #3b1c1c;
    color: #f87171;
  }

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
