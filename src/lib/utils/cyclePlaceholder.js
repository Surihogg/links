// 输入框俏皮 placeholder：用户在空输入下尝试提交时，提示语在
// "给我一点输入" ↔ "你是认真的吗？" 间循环。
//
// 此前 Sidebar 中分组创建 / 子分组创建 / 标签创建 三处都直接内联
// `state === "给我一点输入" ? "你是认真的吗？" : "给我一点输入"`，
// 字面量四散；抽到此处统一来源，方便文案调整。

export const DEFAULT_PLACEHOLDER = "给我一点输入";
export const NAGGED_PLACEHOLDER = "你是认真的吗？";

/**
 * 在两个 placeholder 之间循环切换。
 * @param {string} current 当前 placeholder 值
 * @returns {string} 切换后的值
 */
export function cyclePlaceholder(current) {
  return current === DEFAULT_PLACEHOLDER ? NAGGED_PLACEHOLDER : DEFAULT_PLACEHOLDER;
}
