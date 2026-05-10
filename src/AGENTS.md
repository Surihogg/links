# Svelte 前端 (src)

## 概述

Svelte 5 + Vite 构建。三窗口入口（主窗口 / 快速添加 / 全局搜索 Spotlight），共享 `lib/`。
通过 `lib/api.js` 调用 Rust 后端命令；R3-R6 重构后大组件已拆分为子组件 + `lib/utils/` 共享工具。

## 查找指引

| 需求 | 文件 | 说明 |
|------|------|------|
| 新增 API 调用 | `lib/api.js` | 每个后端命令一个 async 函数；invoke 名采用「动词在前」（如 `list_links` / `create_link`）；invoke 第二参数 key 与 Rust snake_case 一致 |
| 链接 / 分组 / 标签状态 | `lib/stores/index.js` | linksStore（分页 + 搜索）/ categoriesStore（树形）/ tagsStore（排序）/ settingsStore |
| 主题状态 | `lib/stores/themeStore.svelte.js` | 一次 `themeStore.init()` 即接管：theme-mode 加载、prefers-color-scheme 监听、跨窗口 theme-changed 同步 |
| 主布局 | `App.svelte` | 顶部内容头 + Sidebar + LinkList + FAB；onMount 完成数据加载、updater、窗口尺寸/位置恢复、跨窗口监听、键盘导航 |
| 链接卡片 | `lib/components/LinkCard.svelte` | 卡片 UI、hint 浮层、失效标记、书签图标；时间用 `formatRelativeTime` / `formatAbsoluteTime`；域名用 `getDomain`；分享格式用 `formatLinkAs` |
| 链接列表 | `lib/components/LinkList.svelte` | 列表容器、空状态、滚动加载、键盘导航辅助 |
| 链接表单（共用） | `lib/components/LinkForm.svelte` | 支持 `mode="modal" \| "standalone"`；导出 `reset` / `focusUrl` / `triggerFetch` / `setSaving` 给父组件；含 IME 守卫 |
| 侧边栏装配 | `lib/components/Sidebar.svelte` | 仅 266 行，装配 Brand + 顶部 nav + CategorySection + TagSection + footer，承担拖拽对象创建与跨节联动 |
| 侧边栏品牌头 | `lib/components/Sidebar/Brand.svelte` | 图标 + 版本 + 更新铃 + 设置齿轮 |
| 侧边栏分组节 | `lib/components/Sidebar/CategorySection.svelte` | 折叠头 / 新建 / 节内搜索 / 未分组项 / 分组列表（拖拽 / 重命名 / 删除两阶段确认 / 子分组创建）；导出 `isEditing(id)` 给父组件查询 |
| 侧边栏标签节 | `lib/components/Sidebar/TagSection.svelte` | 折叠头 / 新建 / 节内搜索 / 无标签项 / 标签列表（重命名 / 删除两阶段确认） |
| 设置弹窗 | `lib/components/SettingsDialog.svelte` | 关闭行为 / 外观 / 通用开关 / 浏览器扩展 / Bookmarklet；4 套快捷键统一用 `ShortcutEditor` 组件 |
| 单条快捷键编辑 | `lib/components/ShortcutEditor.svelte` | props `name / desc / getter / setter`；导出 `handleKeydown(e)` 给父组件转发 |
| 统计视图 | `lib/components/StatsPanel.svelte` | 总数 + 本周新增 + Top 3 列表 |
| 关闭确认 | `lib/components/CloseDialog.svelte` | 「要走了吗？」对话框 |
| 发布说明 | `lib/components/ReleaseNotesDialog.svelte` | 升级首启时弹；marked 渲染 |
| 搜索 / 标签输入 / 分组输入 | `lib/components/{SearchBar,TagInput,CategoryInput}.svelte` | 已就位的独立组件 |
| 导出 / 更新 / 排序 | `lib/components/{ExportDialog,UpdateDialog,SortSelect}.svelte` | 已就位的独立组件 |
| 快速添加窗口 | `pages/QuickAdd.svelte` | 仅 186 行；窗口生命周期（quick-add-shown / deep-link / Esc）；表单本体复用 `<LinkForm mode="standalone">` |
| 全局搜索 | `pages/Spotlight.svelte` | 即时搜索浮层；接入 `themeStore` / `formatRelativeTime` / `getDomain` |
| 工具：时间 | `lib/utils/time.js` | `formatRelativeTime(ts, options)` / `formatAbsoluteTime(ts)` |
| 工具：URL | `lib/utils/url.js` | `getDomain(url, { stripWww, fallback })` |
| 工具：链接分享 | `lib/utils/linkShare.js` | `formatLinkAs(link, "url"\|"markdown"\|"html")` |
| 工具：IME 守卫 | `lib/utils/imeGuard.svelte.js` | `createImeGuard()` 返回 `{ attach, active }` |
| 工具：分类树 | `lib/utils/categoryTree.js` | `findCategoryById` / `flattenCategories` / `isCategoryDescendant` |
| 工具：分组拖拽 | `lib/utils/categoryDragHandler.svelte.js` | `createCategoryDrag({ getCategories, onDropToParent, onDropToRoot, canStartDrag })` |
| 工具：placeholder 循环 | `lib/utils/cyclePlaceholder.js` | `cyclePlaceholder(current)` + `DEFAULT_PLACEHOLDER` / `NAGGED_PLACEHOLDER` |
| 主题/CSS 变量 | `app.css` | CSS 变量设计系统 + 公共样式（`.btn` / `.modal-*` / `.icon-btn` / `.spinner` / `.spinner-sm` / `.nav-item` / `@keyframes spin`） |

## 约定

- **API 调用**：必须通过 `lib/api.js`，禁止组件直接 `invoke()`；所有 invoke 名采用「动词在前」
- **Store 使用**：组件调用 store 方法（如 `linksStore.load()`），store 内部调用 `api.js`
- **主题切换**：唯一入口 `themeStore.setMode(next)`；其它窗口通过 `theme-changed` 事件自动同步，禁止再写一份 `apply_theme`
- **CSS 变量**：使用 `app.css` 定义的 `--bg-*` / `--text-*` / `--accent-*` / `--scrim-bg` / `--shadow-drag` 等变量，禁止硬编码颜色或 rgba
- **公共样式**：`.btn` / `.modal-overlay` / `.modal` / `.icon-btn` / `.spinner` / `.spinner-sm` / `.nav-item` / `@keyframes spin` 已提到 `app.css`，组件不应再各自定义
- **三窗口**：`main.js` → `App.svelte`；`quick-add.js` → `pages/QuickAdd.svelte`；`spotlight.js` → `pages/Spotlight.svelte`，三者共享 `lib/`
- **TypeScript**：`tsconfig.json` 当前 strict + checkJs 开启，但 `include` 不含 `.js`（事实未生效）；项目 0 处 `as any` / `@ts-ignore`，靠人工纪律保证；R 系列重构未引入全量 TS 化（评估后认为对独立开发者性价比有限）
- **`.svelte.js` / `.svelte.ts` 后缀**：在工具模块中需要 `$state` / `$derived` 时必须用此后缀；本项目已用：`themeStore.svelte.js` / `imeGuard.svelte.js` / `categoryDragHandler.svelte.js`
- **IME 守卫**：表单组件用 `createImeGuard()` 防止中文/日文输入法组词期间 Enter 误触发提交；模板内通过 `ime_guard.active` 在提交前判断
- **LinkForm 复用**：QuickAdd 通过 `mode="standalone"` 复用 LinkForm；新表单优先复用，不要新写一份
- **快捷键编辑**：在 SettingsDialog 这种"多个相同录制条目"的场景，必须用 `ShortcutEditor` 组件，不要再写第二份录制逻辑
- **Sidebar 拖拽**：分组拖拽逻辑封装在 `createCategoryDrag(...)` 中；Sidebar 主组件通过 `bind:this={categorySectionRef}` 拿到子组件引用，注入 `canStartDrag` 询问 `isEditing`

## 反模式

- **禁止**用 `localStorage` 存储配置，统一用 `getSetting` / `setSetting`（持久化到 config.json）
- **禁止**在多个组件中各写一份 `apply_theme` / `format_last_opened` / `extract_domain` / IME 守卫等已抽到 `lib/utils/` 的逻辑
- **禁止**在 SettingsDialog 中重新内联快捷键录制状态机；用 `ShortcutEditor`
- **禁止**在 QuickAdd 中重新维护一份完整的链接表单字段；用 `<LinkForm mode="standalone">`
- **禁止**在组件 `<style>` 中重新定义 `.btn` / `.modal-*` / `.spinner*` / `.nav-item` 等已全局化的样式
- **禁止**用硬编码 `rgba(0, 0, 0, 0.4)` / `box-shadow: 0 4px 12px rgba(0,0,0,0.15)` 等数值；使用 `var(--scrim-bg)` / `var(--shadow-drag)` / `var(--shadow-md)` 等变量
- 其余反模式见根目录 AGENTS.md（invoke 禁直调、`as any` 禁用等）

## 备注

- 前端总行数约 7925；最大文件：`App.svelte` 959 / `LinkCard.svelte` 716 / `SettingsDialog.svelte` 699 / `Sidebar/CategorySection.svelte` 639 / `Spotlight.svelte` 546 / `Sidebar/TagSection.svelte` 450 / `LinkForm.svelte` 447
- `Sidebar.svelte` 主组件仅 266 行（拆分前 1213）；CategorySection / TagSection 内部各保留一份 `.section-header` / `.rename-input` / `.new-cat-form` 样式（约 50 行重复）——这是有意权衡，保证子组件独立可用
- `App.svelte` 不再 `import { marked }`；marked 依赖跟随 ReleaseNotesDialog 转移
- `themeStore.init()` 多次调用安全（内部 `initialized` 标记）；三窗口可各自调用，不会重复注册监听
- `linksStore` 支持分页加载（`loadMore`）和搜索（`search`）；`categoriesStore.update` 返回更新后的分类但不刷新列表，依赖组件自行处理
- 主窗口 `titleBarStyle: "Overlay"`，macOS 下自定义标题栏拖拽区
- QuickAdd / Spotlight 窗口：`alwaysOnTop` + `resizable: false` + `skipTaskbar: true`
- 暗色模式：`themeStore` 给 `<html>` 加 `.dark` class；CSS 变量在 `app.css` 切换；`prefers-color-scheme` 仅在 `theme-mode === "system"` 时生效
- 启动闪屏：`index.html` 含 `#splash`；主窗口 onMount 末尾 `splash.classList.add("fade-out")` 后移除
- QuickAdd / Spotlight 通过 `<html class="theme-ready">` 解除初始遮蔽，避免主题首屏闪白
- Sidebar 拖拽 ghost 元素附着到 `<body>`，样式由 `:global(.drag-ghost)` 在 Sidebar 父组件定义；CategorySection 内只赋 className，不再内联 cssText
- E2E 测试清单：`REFACTOR_E2E_CHECKLIST.md` 仅覆盖 R1-R6 改动；`E2E_TEST_CHECKLIST.md` 是全功能清单
