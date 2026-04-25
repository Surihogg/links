# Svelte 前端 (src)

## 概述

Svelte 5 + TailwindCSS 4 前端。Vite 构建，双窗口入口（主窗口 + 快速添加）。通过 `api.js` 调用 Rust 后端。

## 查找指引

| 需求 | 文件 | 说明 |
|------|------|------|
| 新增 API 调用 | `lib/api.js` | 每个后端命令一个 async 函数，invoke 参数名与 Rust snake_case 一致 |
| 状态管理 | `lib/stores/index.js` | 三个自定义 store：links（分页）、categories（树形）、tags（排序） |
| 主布局 | `App.svelte` | 侧边栏 + 内容区 + FAB + 关闭覆盖层 |
| 链接卡片 | `lib/components/LinkCard.svelte` | 卡片 UI、hint 浮层、失效标记、书签图标 |
| 链接表单 | `lib/components/LinkForm.svelte` | 新增/编辑表单、元数据刷新、去重检测 |
| 侧边栏 | `lib/components/Sidebar.svelte` | 分组树 + 标签列表 + "未分组"/"没标签" 固定筛选 |
| 搜索 | `lib/components/SearchBar.svelte` | FTS5 搜索栏 |
| 标签输入 | `lib/components/TagInput.svelte` | 多标签输入 + 自动补全 |
| 导出 | `lib/components/ExportDialog.svelte` | JSON/MD/CSV 格式选择 |
| 设置 | `lib/components/SettingsDialog.svelte` | 关闭行为、暗色模式等配置 |
| 快速添加 | `pages/QuickAdd.svelte` | 独立窗口的精简添加表单 |
| 主题/CSS | `app.css` | CSS 变量设计系统（明/暗主题） |

## 约定

- **API 调用**：必须通过 `api.js`，禁止组件直接 `invoke()`
- **Store 使用**：组件调用 store 方法（`linksStore.load()`），store 内部调用 `api.js`
- **CSS 变量**：使用 `app.css` 定义的 `--bg-*` / `--text-*` / `--accent-*` 等变量，禁止硬编码颜色
- **全局样式**：`.btn` / `.modal` 等重复样式应提取到 `app.css`，不在组件内重复定义
- **双窗口**：`main.js` 挂载 `App.svelte`，`quick-add.js` 挂载 `QuickAdd.svelte`，共享 `lib/`
- **TypeScript**：`tsconfig.json` 开启严格模式，`allowJs: true` + `checkJs: true`

## 反模式

- **禁止**用 `localStorage` 存储配置，统一用 `getSetting` / `setSetting`（持久化到 config.json）
- 其余反模式见根目录 AGENTS.md（invoke 禁直调、颜色禁硬编码、样式禁重复定义）

## 备注

- `lib/utils/` 目录为空预留
- `stores/index.js` 的 `linksStore` 支持分页加载（`loadMore`）和搜索（`search`）
- `categoriesStore.update` 返回更新后的分类但不刷新列表，依赖组件自行处理
- QuickAdd 窗口：`tauri.conf.json` 配置为 `alwaysOnTop`、`resizable: false`、`skipTaskbar: true`
- 主窗口 `titleBarStyle: "Overlay"`，macOS 下自定义标题栏拖拽区
- 暗色模式通过 CSS `prefers-color-scheme` + `config.json` 双重控制
