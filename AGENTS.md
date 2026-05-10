# Links 项目知识库

**最后刷新:** 2026-05-10（R1-R6 重构后同步）
**对应提交:** 7d9b5c8
**对应分支:** refactor/test
**应用版本:** v1.3.0（重构未发布新 tag）

## 概述

跨平台本地链接管理桌面应用。Tauri 2.x (Rust) 后端 + Svelte 5 前端 + SQLite 本地存储。单应用架构，非 monorepo。

## 项目结构

```
links/
├── src-tauri/                  # Rust 后端（独立 Cargo 项目）
│   └── src/db/                 # 数据库层 R2 已拆为子模块（mod/error/models/migration/row_mapping/repository/search/export/tests）
├── src/
│   ├── lib/components/         # UI 组件
│   │   └── Sidebar/            # R4+R6 抽出：Brand / CategorySection / TagSection
│   ├── lib/stores/             # 状态管理：list/category/tag store + R3 themeStore
│   ├── lib/utils/              # R3+R4 抽出的共享工具：time / url / linkShare / imeGuard / categoryTree / categoryDragHandler / cyclePlaceholder
│   └── pages/                  # 多窗口入口的页面：QuickAdd / Spotlight
├── .github/workflows/          # CI/CD（macOS + Windows 自动构建）
├── browser-extension/          # Chrome 扩展源码
├── index.html                  # 主窗口入口
├── quick-add.html              # 快速添加窗口入口
├── spotlight.html              # 全局搜索浮层入口
├── REFACTOR_E2E_CHECKLIST.md   # R1-R6 端到端测试清单
└── E2E_TEST_CHECKLIST.md       # 全功能测试清单
```

## 查找指引

| 需求 | 位置 | 备注 |
|------|------|------|
| 添加/修改后端命令 | `src-tauri/src/commands.rs` | 所有 `#[tauri::command]` 集中于此（R2 后命令名统一为「动词在前」） |
| 数据模型 / DTO | `src-tauri/src/db/models.rs` | `Link` / `Category` / `Tag` / `*Payload` |
| 数据库迁移 | `src-tauri/src/db/migration.rs` | R5 引入 PRAGMA user_version 显式版本管理 |
| 链接 / 分组 / 标签 CRUD | `src-tauri/src/db/repository.rs` | impl Db 散布于多个文件，repository 是其中一份 |
| 搜索与统计 | `src-tauri/src/db/search.rs` | FTS5 + 标签 + 分类 + LIKE 四路 UNION，FTS 失败回退 |
| 导出（json/md/csv/html） | `src-tauri/src/db/export.rs` | HTML 走 Netscape Bookmark 格式 |
| 行映射 / 共享 SQL 工具 | `src-tauri/src/db/row_mapping.rs` | `LINK_COLUMNS` 列序、`row_to_link/category/tag`、`load_tags_for_links` 批量加载、`build_category_tree` |
| 错误类型 | `src-tauri/src/db/error.rs` | `AppError` 枚举（Database/Io/Fetch/Json/General） |
| URL 抓取 | `src-tauri/src/fetcher.rs` | `build_http_client` 共享 HTTP 客户端 + Windows 系统代理 |
| 浏览器扩展 HTTP 服务 | `src-tauri/src/http_server.rs` | 本地 HTTP 服务（端口 + 令牌鉴权） |
| 配置读写 | `src-tauri/src/config.rs` | `Config(Mutex<HashMap>)` 封装 config.json |
| 应用启动流程 | `src-tauri/src/lib.rs` | 插件注册、DB 初始化、托盘、快捷键、deep-link |
| 前端 API 层 | `src/lib/api.js` | 所有 invoke 调用的 JS 绑定（命名"动词在前"） |
| 主题状态 | `src/lib/stores/themeStore.svelte.js` | 跨窗口主题同步（R3 抽出） |
| 链接 / 分组 / 标签 store | `src/lib/stores/index.js` | linksStore / categoriesStore / tagsStore |
| 共享工具 | `src/lib/utils/` | time / url / linkShare / imeGuard / categoryTree / categoryDragHandler / cyclePlaceholder |
| 主布局 | `src/App.svelte` | 顶部内容头 + Sidebar + LinkList + FAB + 各类模态 |
| 链接表单 | `src/lib/components/LinkForm.svelte` | 支持 `mode="modal" \| "standalone"`，QuickAdd 复用其 standalone 模式 |
| 侧边栏组合 | `src/lib/components/Sidebar.svelte` | 仅装配子组件 |
| 侧边栏子节 | `src/lib/components/Sidebar/` | Brand / CategorySection / TagSection |
| 单条快捷键编辑 | `src/lib/components/ShortcutEditor.svelte` | 替代 SettingsDialog 中 4 套样板 |
| 统计面板 | `src/lib/components/StatsPanel.svelte` | 总数 / 本周新增 / Top 3 |
| 关闭确认 | `src/lib/components/CloseDialog.svelte` | 「要走了吗？」对话框 |
| 发布说明 | `src/lib/components/ReleaseNotesDialog.svelte` | 升级后弹窗，含 marked 渲染 |
| 窗口配置 | `src-tauri/tauri.conf.json` | 主窗口 + quick-add + spotlight 三窗口 |
| 构建配置 | `vite.config.js` | 三入口 rollup（main + quick-add + spotlight） |
| CI 流程 | `.github/workflows/release.yml` | macOS + Windows 双平台自动构建 |

## 代码地图（关键符号）

| 符号 | 类型 | 位置 | 角色 |
|------|------|------|------|
| `Db` | struct | `src-tauri/src/db/mod.rs` | SQLite 连接封装（`Mutex<Connection>`） |
| `AppError` | enum | `src-tauri/src/db/error.rs` | 统一错误类型 |
| `Link` / `Category` / `Tag` | struct | `src-tauri/src/db/models.rs` | 数据模型（pub use 自 db 根） |
| `Migration` | struct | `src-tauri/src/db/migration.rs` | `{ version, apply }` 元组，PRAGMA user_version 驱动 |
| `LINK_COLUMNS` | const | `src-tauri/src/db/row_mapping.rs` | links 表标准 SELECT 列序 |
| `load_tags_for_links` | fn | `src-tauri/src/db/row_mapping.rs` | 批量加载标签消除 N+1 |
| `build_category_tree` | fn | `src-tauri/src/db/row_mapping.rs` | 扁平 → 树形结构构建 |
| `build_http_client` | fn | `src-tauri/src/fetcher.rs` | reqwest builder + Windows 系统代理（fetch / link 检查共用） |
| `update_shortcut` | fn | `src-tauri/src/commands.rs` | 4 套快捷键 setter 的统一 helper |
| `app_data_dir` | fn | `src-tauri/src/commands.rs` | 数据目录 helper（替代散落的 `expect`） |
| `Config` | struct | `src-tauri/src/config.rs` | 配置管理（Mutex<HashMap>） |
| `themeStore` | store | `src/lib/stores/themeStore.svelte.js` | 主题状态 + 跨窗口同步 |
| `linksStore` / `categoriesStore` / `tagsStore` / `settingsStore` | store | `src/lib/stores/index.js` | 各类状态 |
| `createCategoryDrag` | factory | `src/lib/utils/categoryDragHandler.svelte.js` | Sidebar 分组拖拽（Pointer Events） |
| `createImeGuard` | factory | `src/lib/utils/imeGuard.svelte.js` | IME 输入法守卫（防止组词期间误提交） |
| `findCategoryById` / `flattenCategories` / `isCategoryDescendant` | fn | `src/lib/utils/categoryTree.js` | 分类树工具 |
| `cyclePlaceholder` | fn | `src/lib/utils/cyclePlaceholder.js` | 输入框俏皮 placeholder 状态机 |
| `formatRelativeTime` / `formatAbsoluteTime` | fn | `src/lib/utils/time.js` | 时间格式化（LinkCard / Spotlight 共用） |
| `getDomain` | fn | `src/lib/utils/url.js` | URL 域名提取 |
| `formatLinkAs` | fn | `src/lib/utils/linkShare.js` | 链接复制为 url / markdown / html |
| `Spotlight` | page | `src/pages/Spotlight.svelte` | 全局搜索浮层 |
| `QuickAdd` | page | `src/pages/QuickAdd.svelte` | 快速添加窗口（复用 LinkForm standalone 模式） |
| `ShortcutEditor` | component | `src/lib/components/ShortcutEditor.svelte` | 单条快捷键展示 / 录制 / 保存 |
| `StatsPanel` / `CloseDialog` / `ReleaseNotesDialog` | component | `src/lib/components/` | App.svelte 抽出的内联模态 |
| `Sidebar/Brand` / `Sidebar/CategorySection` / `Sidebar/TagSection` | component | `src/lib/components/Sidebar/` | Sidebar 拆出的子节 |

## 约定

- **后端命令注册**：新增 `#[tauri::command]` 后必须在 `lib.rs` 的 `invoke_handler` 宏中注册
- **后端命令命名**：「动词在前」（`list_links` / `create_link` / `update_link` / `delete_link` 等），R2 重构后已统一
- **前端 API**：每个后端命令在 `api.js` 中有对应 async 函数，invoke 第二参数 key 与 Rust snake_case 一致
- **数据库迁移**：R5 起使用 `PRAGMA user_version` + `Migration { version, apply }` 元组数组，**不允许**修改已发布过的迁移函数；新增迁移按版本号递增追加到 `MIGRATIONS` 末尾
- **N+1 防御**：list / search / export 路径加载链接列表时**必须**用 `load_tags_for_links` 批量加载标签，禁止在循环中调用 `load_tags_for_link` 单条版本
- **CSS 变量**：主题色定义在 `app.css`，组件应使用 CSS 变量而非硬编码颜色；遮罩用 `var(--scrim-bg)`，拖拽阴影用 `var(--shadow-drag)`
- **公共样式**：`.btn` / `.modal-overlay` / `.modal` / `.icon-btn` / `.spinner` / `.spinner-sm` / `.nav-item` / `@keyframes spin` 已提到 `app.css`，组件不应再各自定义
- **三窗口架构**：主窗口（index.html / App.svelte）、快速添加（quick-add.html / pages/QuickAdd.svelte）、全局搜索（spotlight.html / pages/Spotlight.svelte）三个独立入口共享 `lib/`
- **主题状态**：通过 `themeStore.init()` 一次性初始化；切换用 `themeStore.setMode(next)` 自动跨窗口同步
- **IME 守卫**：表单组件用 `createImeGuard()` 防止中文/日文输入法组词期间 Enter 误触发提交
- **浏览器扩展**：通过 deep-link 协议（links://）和本地 HTTP 服务与主应用通信
- **自动更新**：`tauri-plugin-updater`，启动检查（受 `auto-check-update` 配置控制）+ 设置页手动检查

## 反模式（本项目）

- **禁止**在组件中硬编码颜色值，必须使用 `app.css` 中定义的 CSS 变量
- **禁止**在多处重复定义 `.btn` / `.modal` / `.spinner` / `.nav-item` 等已全局化的样式
- **禁止**直接在组件中调用 `invoke()`，必须通过 `api.js` 封装
- **禁止**使用 `as any` 或 `@ts-ignore` 抑制类型错误（项目目前 0 处，是已建立的纪律）
- **禁止**在 `create_link` 中同步抓取元数据，必须异步（`tauri::async_runtime::spawn`）
- **禁止**跨 await 持有 `Mutex<Connection>` 锁；导入等长任务**禁止**持锁做文件 IO（R1 修复了 `import_bookmarks` 的此类问题）
- **禁止**在 list/search/export 路径用 `load_tags_for_link` 循环代替 `load_tags_for_links` 批量加载
- **禁止**修改已发布过的 Migration 函数（迁移历史不可变）

## 独特风格

- 中文 UI 文案和提示语风格：有温度、带一点俏皮（如"添加你的想法吧~"、"给我一点输入" ↔ "你是认真的吗？" 的循环）
- 分组/标签删除交互：第一次点击显示"再点一下就删除"提示，鼠标移出恢复原状
- 长标题/描述：hover 时用 hint 浮层展示全文，而非截断
- URL 展示：域名常态显示，hover 原地展开完整 URL
- 快速添加窗口：独立窗口，始终置顶，无任务栏图标，不可调整大小

## 命令

```bash
# 开发
npm run tauri dev          # Rust + 前端热重载
npm run dev                # 仅前端（浏览器预览）

# 构建
npm run build              # 仅前端
npm run tauri build        # 完整打包

# 检查（src-tauri 目录下）
cargo check                # Rust 语法检查
cargo test                 # Rust 单元测试（95 个，覆盖 db 各层、fetcher、normalize、http_server、deep_link）
```

## 备注

- Vite 开发服务器端口固定 1420，被占用会报错（不会自动换端口）
- `normalize.rs` 仍标记为 `#[allow(dead_code)]`，URL 标准化功能仍未接入去重流程（R5 评估后决定保留为未来工具，避免擅改用户原始数据）
- macOS + Windows 双平台 CI 自动构建，Release 为 Draft 模式，需手动确认发布
- 后端最大文件：`commands.rs` 1094 行 / `db/tests.rs` 822 行（测试）/ `fetcher.rs` 566 行；其它生产代码单文件均 ≤ 455 行
- 前端最大文件：`App.svelte` 959 / `LinkCard.svelte` 716 / `SettingsDialog.svelte` 699 / `Sidebar/CategorySection.svelte` 639；Sidebar 主组件仅 266 行
- 关键 crate：`tauri-plugin-updater` / `tauri-plugin-deep-link` / `tauri-plugin-single-instance` / `tiny_http`（浏览器扩展本地 HTTP 服务）/ `marked`（Release Notes Markdown 渲染）

## 变更与提交约束

- **改动提交约束**：所有改动不得直接提交到远程仓库。应在本地创建独立分支，完成测试后再合并提交；提交应通过测试并获得人工测试确认后再进行，确保不破坏主线功能。
- **代码注释与提交信息规范**：代码注释使用中文，提交信息统一使用中文，格式示例：`feat: 添加新功能` 或 `fix: 修复问题`；描述部分应包含改动原因（why）及影响范围，避免空描述。
- **TODO 工作区开发日志归档**：若任务来自 TODO 工作区，完成后将相关条目写入 `TODO.md` 的开发日志（在工作区 TODO.md 内的 `## 开发日志` 小节），按时间戳追加归档，格式为 `YYYY-MM-DD HH:MM:SS`。
- **Release 文档刷新**：每当发布新版本（打 tag `vX.Y.Z`），必须同步刷新 `AGENTS.md`、`README.md`、`USER_GUIDE.md` 三个文档。更新内容包括版本号引用、新功能描述、工作流变更、弃用/迁移说明、资源链接等，确保文档与代码保持一致。
- **重构刷新**：大规模重构（如 R1-R6）即使未打 tag，也应在合并前刷新本文件、`src-tauri/AGENTS.md`、`src/AGENTS.md` 三份知识库文档；REA​DME / USER_GUIDE 仅在涉及对外可见变化时刷新。
- **版本号**：分别在 `tauri.conf.json` 的 `version`、`Cargo.toml` 与 `Cargo.lock` 的 `package.version`、`package.json` 的 `version`（仅前端构建版本，非应用版本，当前未同步）。
