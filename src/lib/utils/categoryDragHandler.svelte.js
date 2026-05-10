// Sidebar 分组拖拽逻辑（Pointer Events 实现）。
//
// 把分组拖到另一个分组 → 改为子分组；拖到 root-drop-zone → 升为根级。
// 5px 死区避免点击误判；自定义 ghost 元素由 :global(.drag-ghost) 样式承载。
//
// 用法（Svelte 5）：
//
// ```js
// const drag = createCategoryDrag({
//   getCategories: () => categories,
//   onDropToParent: (id, parentId) => categoriesStore.update({ id, parent_id: parentId }),
//   onDropToRoot: (id) => categoriesStore.update({ id, unset_parent: true }),
// });
// // 在 cat-item 上：onpointerdown={(e) => drag.start(e, cat)}
// // 在模板中读取 drag.dragId / drag.dropTargetId 决定样式
// // 在判断 is_descendant 时调 drag.isDescendant(cat_id, target_id)
// ```

import { findCategoryById, isCategoryDescendant } from "./categoryTree.js";

/**
 * @param {object} opts
 * @param {() => Array} opts.getCategories 获取当前的分组树（含 children）
 * @param {(id: number, parentId: number) => void} opts.onDropToParent 拖到另一分组
 * @param {(id: number) => void} opts.onDropToRoot 拖到 root-drop-zone
 * @param {() => boolean} [opts.canStartDrag] 额外的启动条件（如未在编辑某项）
 */
export function createCategoryDrag(opts) {
  const { getCategories, onDropToParent, onDropToRoot, canStartDrag } = opts;

  let dragId = $state(null);
  let dropTargetId = $state(null);
  let isDragging = $state(false);
  let pendingDragId = null;
  let dragStartPos = { x: 0, y: 0 };
  let ghostEl = null;

  function createGhost() {
    const cat = findCategoryById(getCategories(), dragId);
    if (!cat) return;
    ghostEl = document.createElement("div");
    // 全部样式由 :global(.drag-ghost) 定义
    ghostEl.className = "drag-ghost";
    ghostEl.textContent = cat.name;
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

  /** 判断 targetId 是否是 sourceId 的后代 → 防止把父分组拖到自己的子分组 */
  function isDescendant(sourceId, targetId) {
    return isCategoryDescendant(getCategories(), sourceId, targetId);
  }

  function updateDropTarget(x, y) {
    // 临时隐藏 ghost 才能拿到鼠标下的真实 DOM
    if (ghostEl) ghostEl.style.display = "none";
    const elem = document.elementFromPoint(x, y);
    if (ghostEl) ghostEl.style.display = "";

    const catItem = elem?.closest(".cat-item[data-cat-id]");
    if (catItem) {
      const targetId = parseInt(catItem.dataset.catId, 10);
      // 不能拖到自己也不能拖到自己的后代
      if (targetId !== dragId && !isDescendant(dragId, targetId)) {
        dropTargetId = targetId;
        return;
      }
    }
    if (elem?.closest(".root-drop-zone")) {
      dropTargetId = "root";
      return;
    }
    dropTargetId = null;
  }

  function onMove(e) {
    if (pendingDragId === null) return;
    const dx = e.clientX - dragStartPos.x;
    const dy = e.clientY - dragStartPos.y;
    if (!isDragging && (Math.abs(dx) > 5 || Math.abs(dy) > 5)) {
      isDragging = true;
      dragId = pendingDragId;
      createGhost();
    }
    if (isDragging) {
      e.preventDefault();
      moveGhost(e.clientX, e.clientY);
      updateDropTarget(e.clientX, e.clientY);
    }
  }

  function onUp(e) {
    if (isDragging && dragId !== null) {
      // 释放前再次校正坐标，避免快速移动导致 hover 状态滞后
      updateDropTarget(e.clientX, e.clientY);
      e.preventDefault();
      if (dropTargetId === "root") {
        onDropToRoot(dragId);
      } else if (dropTargetId !== null && dropTargetId !== dragId) {
        onDropToParent(dragId, dropTargetId);
      }
    }
    cleanup();
  }

  function cleanup() {
    removeGhost();
    window.removeEventListener("pointermove", onMove);
    window.removeEventListener("pointerup", onUp);
    window.removeEventListener("pointercancel", onUp);
    dragId = null;
    pendingDragId = null;
    dropTargetId = null;
    isDragging = false;
  }

  /** 在 cat-item 的 pointerdown 处调用 */
  function start(e, cat) {
    if (e.button !== 0) return; // 仅左键
    if (canStartDrag && !canStartDrag(cat)) return;
    pendingDragId = cat.id;
    dragStartPos = { x: e.clientX, y: e.clientY };
    isDragging = false;
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    window.addEventListener("pointercancel", onUp);
  }

  return {
    /** 当前正在拖动的分组 ID（响应式） */
    get dragId() { return dragId; },
    /** 当前命中的 drop target：分组 id / 字符串 'root' / null（响应式） */
    get dropTargetId() { return dropTargetId; },
    /** 模板中判断 .drop-target 是否应高亮 */
    isDescendant,
    /** 在 cat-item 上 onpointerdown */
    start,
  };
}
