# UI 组件

7 个 Svelte 5 组件，纯展示 + 回调 props，不直接持有 store。

## 文件一览

| 文件 | 行数 | 职责 |
|---|---|---|
| `Sidebar.svelte` | 520 | 分组树（树形嵌套）+ 标签列表 + 分组/标签 CRUD + 暗色切换 + 导出入口 |
| `LinkForm.svelte` | 323 | 添加/编辑链接弹窗（模态框），含 URL 输入自动抓取 |
| `LinkCard.svelte` | 283 | 单条链接卡片，点击整卡 → `openUrl`，右上角编辑/收藏/删除 |
| `ExportDialog.svelte` | 231 | 导出格式选择 → 调 `exportLinks` → `saveFile`（rfd 原生对话框） |
| `TagInput.svelte` | 161 | 标签输入框 + `autocompleteTags` 模糊建议 |
| `SearchBar.svelte` | 81 | 搜索输入框，250ms 防抖触发 `onsearch` |
| `LinkList.svelte` | 65 | 链接列表渲染（空态、加载态），把 `categories` 扁平化成 name 映射传给 `LinkCard` |

## 组件通信约定

**Props = callback + 数据，不是 EventDispatcher**：

```svelte
let { link, onedit, ondelete, ontoggle_favorite } = $props();
// 用法：onedit?.(link)
```

- 所有事件都是 `on{action}` 前缀的函数 prop（不用 Svelte 4 的 `createEventDispatcher`）。
- 双向绑定（如 `TagInput` 的 `tags`、`SearchBar` 的 `query`）用 `$bindable()`。
- 组件内部状态用 `$state`，派生用 `$derived`，**跨组件共享**才进 `lib/stores/index.js`。

## `LinkForm.svelte` 双抓取路径（易混淆）

1. **自动抓取**：URL 输入框 500ms 防抖后触发 `do_fetch(url)` → `fetchMeta(url)`。用户手动编辑过的字段不覆盖（`user_edited = { title: false, description: false }` 追踪）。
2. **后端异步抓取**：`links_create` 提交后，Rust 侧 `tauri::async_runtime::spawn` 再抓一次。**两条路径都存在**，是有意冗余——前端抓取让用户看到即时反馈，后端抓取兜底（网络抖动、用户不填 URL 等）。

## `Sidebar.svelte` Set 响应式陷阱

`$state(new Set())` 上直接 `.add()` / `.delete()` **不触发响应式**：

```js
// ❌ 错误：expanded.has(id) ? expanded.delete(id) : expanded.add(id);
// ✅ 正确（见 toggle_section）：
function toggle_section(key) {
  const next = new Set(collapsed);
  if (next.has(key)) next.delete(key);
  else next.add(key);
  collapsed = next;  // 重新赋值才触发 re-render
}
```

当前代码中 `toggle_section`（line 17-22）用了新 Set 赋值，`toggle`（line 24-27）是直接 mutate —— 如果分类展开动画失效，去 `toggle` 里改成同样模式。

## `LinkCard.svelte` HTML 转义 + 高亮

`hl(text)` 手动 `esc()`（`&`/`<`/`>`）后再用 `{@html}` 渲染带 `<span>` 的高亮。**必须先转义再插高亮**，否则链接标题里的 `<script>` 会被执行。

## 模态框必须在 `.dark` div 内部

`LinkForm` 和 `ExportDialog` 都是 `position: fixed` 的 overlay。它们在 `App.svelte` 里是**根级** `{#if}` 渲染的，但放在 `<div class={dark_mode ? "dark" : ""}>` **内部**——这样 CSS 变量才从父级继承。改位置会导致暗色下模态框变白。

## 反模式

- **不要** 在组件里 import stores 直接改（`linksStore.load()` 等）——通过 callback props 让父组件协调。唯一例外：`Sidebar` 自己调 `categoriesStore.load()`（已在用）。
- **不要** 用 `createEventDispatcher` —— Svelte 5 模式是 callback props。
- **不要** 在 `$state(link?.url ?? "")` 后期望 `link` 变化会更新——`$state(expr)` 只捕获初始值。这是表单初始化模式，**不是** bug。
- **不要** 直接 mutate `$state(new Set())`——新建 Set 再赋值。
- **不要** 在 a11y 警告上花时间——label without for、span with onclick 等警告不阻止构建，忽略。
