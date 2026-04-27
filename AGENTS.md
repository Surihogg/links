# Links 项目知识库

**生成时间:** 2026-04-26
**提交:** 3ede9cd
**分支:** main

## 概述

跨平台本地链接管理桌面应用。Tauri 2.x (Rust) 后端 + Svelte 5 前端 + SQLite 本地存储。单应用架构，非 monorepo。

## 项目结构

```
links/
├── src-tauri/        # Rust 后端（独立 Cargo 项目）
├── src/              # Svelte 前端（Vite 构建）
├── .github/          # CI/CD（仅 Windows Release）
└── index.html        # 主窗口入口
```

## 查找指引

| 需求 | 位置 | 备注 |
|------|------|------|
| 添加/修改后端命令 | `src-tauri/src/commands.rs` | 所有 `#[tauri::command]` 集中于此 |
| 数据库操作 | `src-tauri/src/db.rs` | 模型、CRUD、搜索、导出逻辑 |
| URL 抓取逻辑 | `src-tauri/src/fetcher.rs` | 元数据解析、Windows 代理 |
| 配置读写 | `src-tauri/src/config.rs` | config.json 的 HashMap 封装 |
| 应用启动流程 | `src-tauri/src/lib.rs` | 插件注册、DB 初始化、托盘、快捷键 |
| 前端 API 层 | `src/lib/api.js` | 所有 invoke 调用的 JS 绑定 |
| 状态管理 | `src/lib/stores/index.js` | links / categories / tags 三个 store |
| UI 组件 | `src/lib/components/` | 8 个 Svelte 组件 |
| 窗口配置 | `src-tauri/tauri.conf.json` | 主窗口 + quick-add 窗口 |
| 构建配置 | `vite.config.js` | 双入口 rollup（main + quick-add） |
| CI 流程 | `.github/workflows/release.yml` | Windows 自动构建 |

## 代码地图

| 符号 | 类型 | 位置 | 角色 |
|------|------|------|------|
| `Db` | struct | `src-tauri/src/db.rs` | SQLite 连接封装（Mutex<Connection>） |
| `Config` | struct | `src-tauri/src/config.rs` | 配置管理（Mutex<HashMap>） |
| `PageMeta` | struct | `src-tauri/src/fetcher.rs` | 抓取结果模型 |
| `AppError` | enum | `src-tauri/src/db.rs` | 统一错误类型（Database/IO/Fetch/Json/General） |
| `Link` | struct | `src-tauri/src/db.rs` | 链接数据模型 |
| `Category` | struct | `src-tauri/src/db.rs` | 分组模型（含 children 树） |
| `linksStore` | store | `src/lib/stores/index.js` | 链接状态（分页、加载、搜索） |
| `categoriesStore` | store | `src/lib/stores/index.js` | 分组状态（树形） |
| `tagsStore` | store | `src/lib/stores/index.js` | 标签状态（排序） |

## 约定

- **后端命令注册**：新增 `#[tauri::command]` 后必须在 `lib.rs` 的 `invoke_handler` 宏中注册
- **前端 API**：每个后端命令在 `api.js` 中有对应 async 函数，参数名与 Rust 保持 snake_case 一致
- **数据库迁移**：当前使用 `CREATE TABLE IF NOT EXISTS` + `ALTER TABLE ADD COLUMN`（无版本管理），见 TODO.md 中的技术债务
- **CSS 变量**：主题色定义在 `app.css`，组件应使用 CSS 变量而非硬编码颜色
- **双窗口架构**：主窗口（index.html）和快速添加窗口（quick-add.html）是两个独立入口

## 反模式（本项目）

- **禁止**在组件中硬编码颜色值，必须使用 `app.css` 中定义的 CSS 变量
- **禁止**在多处重复定义 `.btn` / `.modal` 等全局样式，应提取到 `app.css`
- **禁止**直接在组件中调用 `invoke()`，必须通过 `api.js` 封装
- **禁止**使用 `as any` 或 `@ts-ignore` 抑制类型错误
- **禁止**在 `links_create` 中同步抓取元数据，必须异步（`tauri::async_runtime::spawn`）

## 独特风格

- 中文 UI 文案和提示语风格：有温度、带一点俏皮（如"添加你的想法吧~"）
- 分组删除交互：第一次点击显示"再点一下"提示，鼠标移出恢复原状
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

# 检查
cd src-tauri && cargo check  # Rust 语法检查
cd src-tauri && cargo test   # Rust 单元测试（db.rs, fetcher.rs, normalize.rs）
```

## 备注

- Vite 开发服务器端口固定 1420，被占用会报错（不会自动换端口）
- `src/lib/utils/` 目录为空，预留未使用
- `normalize.rs` 标记为 `#[allow(dead_code)]`，URL 标准化功能未接入去重流程
- Windows 构建 CI 仅构建 Windows 包，macOS/Linux 需本地或添加 CI Runner
- Release 为 Draft 模式，需手动确认发布
- `db.rs` 含 30+ 个 Rust 单元测试（`cargo test` 可运行），前端暂无测试


## 变更与提交约束（新增）
- 改动提交约束：所有改动不得直接提交到远程仓库。应在本地创建独立分支，完成测试后再合并提交；提交应通过测试并获得人工测试确认后再进行，确保不破坏主线功能。
- 代码注释与提交信息规范：代码注释使用中文，提交信息统一使用中文，格式示例：feat: 添加新功能 或 fix: 修复问题；描述部分应包含改动原因（why）及影响范围，避免空描述。
- TODO 工作区开发日志归档：若任务来自 TODO 工作区，完成后将相关条目写入 TODO.md 的开发日志（在工作区 TODO.md 内的 ## 开发日志 小节），按时间戳追加归档，格式为 YYYY-MM-DD HH:MM:SS。
