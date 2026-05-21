// TagInput 标签拖拽排序（Pointer Events 实现）。
//
// 拖拽 tag-pill 到另一个 tag-pill 上方/下方 → 列表内换位。
// 5px 死区避免点击误判；自定义 ghost 元素跟随光标。
//
// 用法（Svelte 5）：
//
// ```js
// const drag = createTagDrag({
//   getTags: () => tags,
//   onReorder: (newTags) => { tags = newTags; onchange?.(tags); },
// });
// // 在 tag-pill 上：onpointerdown={(e) => drag.start(e, i)}
// // 在模板中读取 drag.dragIndex / drag.dropIndex 决定样式
// ```

/**
 * @param {object} opts
 * @param {() => string[]} opts.getTags 获取当前标签数组
 * @param {(newTags: string[]) => void} opts.onReorder 拖拽完成后的回调
 */
export function createTagDrag(opts) {
  const { getTags, onReorder } = opts;

  let dragIndex = $state(-1);
  let dropIndex = $state(-1);
  let isDragging = $state(false);
  let pendingIndex = -1;
  let dragStartPos = { x: 0, y: 0 };
  let ghostEl = null;

  function createGhost(tag) {
    ghostEl = document.createElement("span");
    ghostEl.className = "tag-drag-ghost";
    ghostEl.textContent = tag;
    document.body.appendChild(ghostEl);
  }

  function moveGhost(x, y) {
    if (!ghostEl) return;
    ghostEl.style.left = x + 10 + "px";
    ghostEl.style.top = y + 10 + "px";
  }

  function removeGhost() {
    ghostEl?.remove();
    ghostEl = null;
  }

  function updateDropTarget(x, y) {
    // 临时隐藏 ghost 才能拿到鼠标下的真实 DOM
    if (ghostEl) ghostEl.style.display = "none";
    const elem = document.elementFromPoint(x, y);
    if (ghostEl) ghostEl.style.display = "";

    const pill = elem?.closest(".tag-pill[data-tag-index]");
    if (pill) {
      const targetIdx = parseInt(pill.dataset.tagIndex, 10);
      if (targetIdx !== dragIndex) {
        dropIndex = targetIdx;
        return;
      }
    }
    dropIndex = -1;
  }

  function onMove(e) {
    if (pendingIndex === -1) return;
    const dx = e.clientX - dragStartPos.x;
    const dy = e.clientY - dragStartPos.y;
    if (!isDragging && (Math.abs(dx) > 5 || Math.abs(dy) > 5)) {
      isDragging = true;
      dragIndex = pendingIndex;
      const tags = getTags();
      if (dragIndex < tags.length) {
        createGhost(tags[dragIndex]);
      }
    }
    if (isDragging) {
      e.preventDefault();
      moveGhost(e.clientX, e.clientY);
      updateDropTarget(e.clientX, e.clientY);
    }
  }

  function onUp(e) {
    if (isDragging && dragIndex >= 0 && dropIndex >= 0 && dragIndex !== dropIndex) {
      // 执行换位
      const tags = [...getTags()];
      const [moved] = tags.splice(dragIndex, 1);
      tags.splice(dropIndex, 0, moved);
      onReorder(tags);
    }
    cleanup();
  }

  function cleanup() {
    removeGhost();
    window.removeEventListener("pointermove", onMove);
    window.removeEventListener("pointerup", onUp);
    window.removeEventListener("pointercancel", onUp);
    dragIndex = -1;
    pendingIndex = -1;
    dropIndex = -1;
    isDragging = false;
  }

  /** 在 tag-pill 的 pointerdown 处调用 */
  function start(e, index) {
    if (e.button !== 0) return; // 仅左键
    pendingIndex = index;
    dragStartPos = { x: e.clientX, y: e.clientY };
    isDragging = false;
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    window.addEventListener("pointercancel", onUp);
  }

  return {
    /** 当前正在拖动的标签索引（响应式） */
    get dragIndex() { return dragIndex; },
    /** 当前命中的放置目标索引（响应式） */
    get dropIndex() { return dropIndex; },
    /** 是否正在拖拽中（响应式） */
    get isDragging() { return isDragging; },
    /** 在 tag-pill 上 onpointerdown */
    start,
  };
}
