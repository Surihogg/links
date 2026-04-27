<script>
  import { openUrl, getSetting } from "../api.js";
  import { copyToClipboard } from "../api.js";

  let { link, highlight = "", category_name = null, onedit, ondelete, ontoggle_favorite, onremovecategory, onremovetag } = $props();
  let show_confirm = $state(false);
  let show_share_menu = $state(false);
  let copy_success = $state(false);

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

  async function card_click() {
    openUrl(link.url);
    if ((await getSetting("auto-minimize-on-open")) === "true") {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      await getCurrentWindow().hide();
    }
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

  function toggle_share() {
    show_share_menu = !show_share_menu;
  }

  function close_share() {
    show_share_menu = false;
  }

  async function copy_as(format) {
    const title = link.title || link.url;
    let content = "";
    function escMarkdown(text) {
      return text.replace(/[\[\]]/g, '\\$&').replace(/[*_`~]/g, '\\$&');
    }
    function escHtml(text) {
      return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
    }
    if (format === 'url') {
      content = link.url;
    } else if (format === 'markdown') {
      content = `[${escMarkdown(title)}](${link.url})`;
    } else {
      content = `<a href="${escHtml(link.url)}">${escHtml(title)}</a>`;
    }
    await copyToClipboard(content);
    show_share_menu = false;
    copy_success = true;
    setTimeout(() => copy_success = false, 1500);
  }

  function check_title_overflow(e) {
    const el = e.target;
    title_truncated = el.scrollWidth > el.clientWidth;
  }

  function check_desc_overflow(e) {
    const el = e.target;
    desc_truncated = el.scrollWidth > el.clientWidth;
  }
</script>

<div class="link-card" data-link-id={link.id}>
  {#if link.is_favorite}
    <div class="bookmark-corner" data-tooltip="取消标记" onclick={toggle_fav}>
      <svg width="10" height="14" viewBox="0 0 10 14" fill="var(--star)" stroke="none">
        <path d="M1 0H9V14L5 10L1 14Z"/>
      </svg>
    </div>
  {/if}
  <div class="card-main">
    <div class="card-content" onclick={card_click}>
      <div class="card-top">
          {#if link.is_broken}
            <span class="broken-badge" data-tooltip="链接可能已失效">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/>
                <line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>
              </svg>
            </span>
          {/if}
        <div class="card-title-row" data-tooltip={title_truncated ? (link.title || link.url) : undefined}>
          {#if link.favicon_url}
            <img src={link.favicon_url} alt="" class="favicon" referrerpolicy="no-referrer" onerror={(e) => e.target.style.display = 'none'} />
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
        <div class="card-desc-row" data-tooltip={desc_truncated ? link.description : undefined}>
          <p class="card-desc" onmouseenter={check_desc_overflow}>{@html hl(link.description)}</p>
        </div>
      {/if}

      {#if category_name || link.tags.length > 0 || link.notes}
        <div class="card-tags">
          {#if category_name}
            <span class="cat-chip">
              <span class="chip-icon-area"
                onclick={(e) => { e.stopPropagation(); onremovecategory?.(link); }}
              >
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="chip-icon icon-folder"><path d="M3 7V17C3 18.1046 3.89543 19 5 19H19C20.1046 19 21 18.1046 21 17V9C21 7.89543 20.1046 7 19 7H13L11 5H5C3.89543 5 3 5.89543 3 7Z"/></svg>
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="chip-icon icon-delete"><path d="M3 6h18M8 6V4a2 2 0 012-2h4a2 2 0 012 2v2M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6"/></svg>
              </span>
              {category_name}
            </span>
          {/if}
          {#each link.tags.slice(0, 5) as tag}
            <span class="tag-chip">
              <span class="chip-icon-area"
                onclick={(e) => { e.stopPropagation(); onremovetag?.(link, tag); }}
              >
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="chip-icon icon-tag"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="chip-icon icon-delete"><path d="M3 6h18M8 6V4a2 2 0 012-2h4a2 2 0 012 2v2M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6"/></svg>
              </span>
              {@html hl(tag)}
            </span>
          {/each}
          {#if link.tags.length > 5}
            <span class="tag-more">+{link.tags.length - 5}</span>
          {/if}
          {#if link.notes}
            <span class="note-chip">
              <svg class="note-icon" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
              <span class="note-text">{@html hl(link.notes)}</span>
            </span>
          {/if}
        </div>
      {/if}
    </div>

    <div class="card-actions">
      <button class="action-btn" onclick={toggle_fav}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"/>
        </svg>
      </button>
      <button class="action-btn" onclick={edit_link}>
        <svg width="14" height="14" viewBox="0 0 1024 1024" fill="currentColor">
          <path d="M867.22 413.07c-9.68 0-19.36-3.63-26.82-10.92-15.19-14.82-15.49-39.14-0.68-54.32 46.84-48.02 45.89-125.18-2.12-172.02-23.27-22.7-54.13-34.93-86.46-34.56-32.49 0.4-62.87 13.43-85.56 36.69-14.83 15.19-39.15 15.47-54.32 0.68-15.19-14.81-15.49-39.13-0.68-54.32C687 45.94 812.9 44.4 891.24 120.82c78.33 76.42 79.89 202.32 3.47 280.66-7.52 7.71-17.51 11.59-27.49 11.59z"/>
          <path d="M819.09 462.01c-9.68 0-19.36-3.63-26.82-10.92L563.13 227.55c-15.19-14.82-15.49-39.14-0.68-54.32 14.82-15.2 39.15-15.47 54.32-0.68L845.92 396.1c15.19 14.82 15.49 39.14 0.68 54.32-7.54 7.72-17.52 11.59-27.51 11.59z"/>
          <path d="M164.51 674.68c-9.68 0-19.36-3.63-26.82-10.92-15.19-14.82-15.49-39.14-0.68-54.32l473.74-485.6c14.82-15.2 39.15-15.47 54.33-0.67 15.18 14.82 15.48 39.14 0.67 54.33L192.01 663.09c-7.53 7.72-17.52 11.59-27.5 11.59z"/>
          <path d="M111.34 958.62c-2.31 0-4.65-0.21-7.01-0.64-20.86-3.85-34.66-23.88-30.81-44.74l51.7-280.46c3.85-20.86 23.86-34.7 44.74-30.81 20.86 3.85 34.66 23.88 30.81 44.74l-51.7 280.46c-3.41 18.5-19.56 31.45-37.73 31.45z"/>
          <path d="M393.86 898.44c-9.68 0-19.36-3.63-26.82-10.92-15.19-14.82-15.49-39.14-0.68-54.32L840.1 347.6c14.82-15.19 39.14-15.49 54.32-0.68 15.19 14.82 15.49 39.13 0.68 54.32l-473.74 485.6c-7.53 7.72-17.51 11.6-27.5 11.6z"/>
          <path d="M111.3 958.66c-17.79 0-33.76-12.42-37.56-30.52-4.36-20.76 8.93-41.13 29.7-45.49l279.1-58.62c20.8-4.35 41.13 8.93 45.49 29.7 4.36 20.76-8.93 41.13-29.7 45.49l-279.1 58.62c-2.66 0.55-5.31 0.82-7.93 0.82z"/>
          <path d="M912.71 959.5H592.59c-21.21 0-38.41-17.2-38.41-38.41 0-21.21 17.2-38.41 38.41-38.41h320.12c21.21 0 38.41 17.2 38.41 38.41 0 21.21-17.2 38.41-38.41 38.41z"/>
        </svg>
      </button>
      <button class="action-btn" onclick={toggle_share}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
          <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
        </svg>
      </button>
      <button class="action-btn danger" onclick={delete_link}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M3 6h18M8 6V4a2 2 0 012-2h4a2 2 0 012 2v2M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6"/>
        </svg>
      </button>
    </div>
  </div>

  {#if show_share_menu}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="share-overlay" onclick={close_share}></div>
    <div class="share-menu">
      <button class="share-option" onclick={() => copy_as('url')}>复制链接</button>
      <button class="share-option" onclick={() => copy_as('markdown')}>复制 Markdown</button>
      <button class="share-option" onclick={() => copy_as('html')}>复制 HTML</button>
    </div>
  {/if}

  {#if copy_success}
    <span class="copy-toast">已复制!</span>
  {/if}

  {#if show_confirm}
    <div class="confirm-overlay" onclick={() => show_confirm = false}>
      <div class="confirm-box" onclick={(e) => e.stopPropagation()}>
        <p class="confirm-text">确定要和这个链接说再见吗？</p>
        <div class="confirm-actions">
          <button class="btn btn-secondary btn-sm" onclick={() => show_confirm = false}>算了</button>
          <button class="btn btn-danger btn-sm" onclick={confirm_delete}>没错</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .link-card {
    position: relative;
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

  .bookmark-corner {
    position: absolute;
    top: 0;
    right: 24px;
    z-index: 5;
    cursor: pointer;
    display: flex;
    align-items: flex-start;
    padding: 0 3px 2px;
    border-radius: 0 0 3px 3px;
    transition: opacity var(--transition);
  }

  .bookmark-corner[data-tooltip]:hover::after {
    content: attr(data-tooltip);
    position: absolute;
    right: 0;
    top: 100%;
    z-index: 50;
    white-space: nowrap;
    padding: 4px 8px;
    background: var(--bg-0);
    border: 1px solid var(--border-1);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-md);
    font-size: 11px;
    font-weight: 400;
    color: var(--text-2);
    pointer-events: none;
  }

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
  .card-desc-row[data-tooltip]:hover::after {
    content: attr(data-tooltip);
    position: absolute;
    left: 0;
    top: 100%;
    z-index: 50;
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

  .card-desc-row {
    position: relative;
    min-width: 0;
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
    margin-top: 2px;
    padding-left: 22px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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
    background: var(--cat-soft);
    color: var(--cat-text);
    font-weight: 500;
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

  .tag-more {
    font-size: 11px;
    color: var(--text-3);
    padding: 1px 4px;
  }

  .note-chip {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 11px;
    background: var(--bg-2);
    color: var(--text-2);
    max-width: 200px;
    overflow: hidden;
  }

  .note-icon {
    flex-shrink: 0;
    opacity: 0.5;
  }

  .note-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .chip-icon-area {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }

  .chip-icon.icon-folder,
  .chip-icon.icon-tag {
    display: inline;
  }

  .chip-icon.icon-delete {
    display: none;
    color: var(--danger);
  }

  .cat-chip:hover .icon-folder {
    display: none;
  }
  .cat-chip:hover .icon-delete {
    display: inline;
  }

  .tag-chip:hover .icon-tag {
    display: none;
  }
  .tag-chip:hover .icon-delete {
    display: inline;
  }

  .card-actions {
    display: flex;
    gap: 2px;
    flex-shrink: 0;
    align-self: center;
  }

  .card-actions .action-btn {
    opacity: 0;
    transition: opacity var(--transition);
  }

  .link-card:hover .card-actions .action-btn { opacity: 1; }

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

  .action-btn:hover { background: var(--border-1); color: var(--text-1); }
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

  /* Share dropdown styles */
  .share-overlay {
    position: fixed;
    inset: 0;
    z-index: 49;
  }

  .share-menu {
    position: absolute;
    right: 24px;
    top: 0;
    z-index: 50;
    background: var(--bg-0);
    border: 1px solid var(--border-1);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    padding: 4px;
    display: flex;
    flex-direction: column;
    min-width: 140px;
  }

  .share-option {
    background: none;
    border: none;
    color: var(--text-1);
    font-size: 12px;
    font-weight: 500;
    text-align: left;
    padding: 7px 12px;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition);
  }

  .share-option:hover {
    background: var(--bg-2);
    color: var(--text-0);
  }

  .copy-toast {
    position: absolute;
    right: 24px;
    top: 0;
    z-index: 50;
    background: var(--bg-0);
    border: 1px solid var(--border-1);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    padding: 6px 12px;
    font-size: 12px;
    color: var(--success);
    pointer-events: none;
  }

  .broken-badge {
    position: relative;
    display: inline-flex;
    align-items: center;
    color: var(--star);
    flex-shrink: 0;
  }
  :global(.dark) .broken-badge { color: var(--star); }
  .broken-badge[data-tooltip]:hover::after {
    content: attr(data-tooltip);
    position: absolute;
    left: 0;
    top: calc(100% + 4px);
    z-index: 50;
    white-space: nowrap;
    padding: 4px 8px;
    background: var(--bg-0);
    border: 1px solid var(--border-1);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-md);
    font-size: 11px;
    font-weight: 400;
    color: var(--text-2);
    pointer-events: none;
  }
</style>
