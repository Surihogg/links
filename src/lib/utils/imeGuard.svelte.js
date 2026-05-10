// IME 输入法守卫：在用户用中文/日文等输入法组词期间，应屏蔽 Enter 等键的副作用。
// 监听 document 的 compositionstart / compositionend，并在 end 后保留一个尾巴
// 防抖期（默认 200ms），兼容某些输入法在 end 后仍会派发 Enter 的情况。
//
// 此前 LinkForm / CategoryInput / TagInput / QuickAdd 各写一份；现在统一抽到此处。
// 文件后缀 .svelte.js 让 $state 在工具模块中可用。

/**
 * 创建一个 IME 守卫对象。
 *
 * 用法（Svelte 5）：
 *
 * ```js
 * const guard = createImeGuard();
 * $effect(() => guard.attach());      // 自动在组件销毁时清理
 * if (guard.active) return;            // 在 IME 组词期间跳过
 * ```
 *
 * @param {object} [options]
 * @param {number} [options.tailMs=200] composition end 后仍视为 active 的尾巴时长（毫秒）
 */
export function createImeGuard(options = {}) {
  const { tailMs = 200 } = options;
  let activeState = $state(false);
  let timer = null;

  function attach() {
    const onStart = () => {
      activeState = true;
      clearTimeout(timer);
    };
    const onEnd = () => {
      activeState = true;
      clearTimeout(timer);
      timer = setTimeout(() => {
        activeState = false;
      }, tailMs);
    };
    document.addEventListener("compositionstart", onStart, true);
    document.addEventListener("compositionend", onEnd, true);
    return () => {
      clearTimeout(timer);
      document.removeEventListener("compositionstart", onStart, true);
      document.removeEventListener("compositionend", onEnd, true);
    };
  }

  return {
    attach,
    /** 当前是否处于 IME 组词期内（含尾巴防抖期）。是响应式 getter。 */
    get active() {
      return activeState;
    },
  };
}
