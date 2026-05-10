// 分类树工具函数。
// Sidebar / CategoryInput / App 各处对树进行查找、扁平化的方式略有差异，
// 但底层算法一致，集中到此便于维护。

/**
 * 在分类树中按 id 递归查找节点。
 * 返回原节点引用（带 children），若未找到则返回 null。
 *
 * @param {Array} nodes 顶层分类数组（每个含 children: Array）
 * @param {number} id
 * @returns {object|null}
 */
export function findCategoryById(nodes, id) {
  for (const node of nodes) {
    if (node.id === id) return node;
    if (node.children?.length) {
      const found = findCategoryById(node.children, id);
      if (found) return found;
    }
  }
  return null;
}

/**
 * 把分类树扁平化为一维数组，附带每项的 depth（缩进层级）。
 * 可选地按 expanded 集合过滤（Sidebar 用），或附带 _path 字符串（CategoryInput 用）。
 *
 * @param {Array} cats 顶层分类数组
 * @param {object} [options]
 * @param {Set<number>} [options.expanded] 仅展开节点的子节点会被加入；
 *   未传则全部展开
 * @param {boolean} [options.withPath=false] 是否给每项追加 _path 字段（"父/子"）
 * @returns {Array}
 */
export function flattenCategories(cats, options = {}) {
  const { expanded, withPath = false } = options;
  const result = [];
  function walk(nodes, depth, parentPath) {
    for (const cat of nodes) {
      const path = withPath
        ? parentPath ? `${parentPath}/${cat.name}` : cat.name
        : undefined;
      const flat = withPath
        ? { ...cat, depth, _path: path }
        : { ...cat, depth };
      result.push(flat);
      const shouldRecurse = cat.children?.length > 0 &&
        (expanded === undefined || expanded.has(cat.id));
      if (shouldRecurse) walk(cat.children, depth + 1, path);
    }
  }
  walk(cats, 0, "");
  return result;
}

/**
 * 判断 targetId 是否是 sourceId 的后代节点（含直接 / 间接子级）。
 * 用于拖拽时阻止把父分组拖到自己的子分组造成循环依赖。
 *
 * @param {Array} nodes 顶层分类数组
 * @param {number} sourceId
 * @param {number} targetId
 * @returns {boolean}
 */
export function isCategoryDescendant(nodes, sourceId, targetId) {
  if (!sourceId || !targetId) return false;
  const source = findCategoryById(nodes, sourceId);
  if (!source) return false;
  return containsId(source.children || [], targetId);
}

function containsId(nodes, id) {
  for (const node of nodes) {
    if (node.id === id) return true;
    if (node.children?.length && containsId(node.children, id)) return true;
  }
  return false;
}
