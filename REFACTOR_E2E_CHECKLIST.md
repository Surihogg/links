# R1-R6 重构端到端测试清单

> 仅覆盖 R1-R6 六轮重构所触及的代码路径，不重复 `E2E_TEST_CHECKLIST.md` 的全功能回归。
> 每条用例都标注了对应的重构来源（R1/R2/...），方便定位回归。
>
> **建议执行顺序**：P0 → P1 → P2，发现回归先看对应 R 编号的 commit。
> **测试前置**：备份 `links.db` 与 `config.json`，避免污染日常数据。

**优先级**：P0 = 改动核心路径，必测 / P1 = 重要分支 / P2 = 边界与体验

**重构 commit 映射**：

| 标签 | Commit | 主题 |
|---|---|---|
| R1 | `f6dcdf4` | fetcher 优先级 bug + import 持锁 IO + 4 套快捷键 helper + ensure_tags / get_or_create_category 单例化 + Windows 代理 builder 共用 + app_data_dir helper |
| R2 | `070fbf6` | db.rs 拆 9 文件 + 39 个 invoke 重命名（动词在前） |
| R3 | `70fccf4` | themeStore / imeGuard / time / url / linkShare 工具抽取 + ShortcutEditor + LinkForm/QuickAdd 合并 + app.css 公共样式 + 8 处硬编码颜色清理 |
| R4 | `7967a87` | App.svelte 抽 StatsPanel/CloseDialog/ReleaseNotesDialog + Sidebar/Brand 抽出 + categoryDragHandler / categoryTree / cyclePlaceholder 工具抽取 |
| R5 | `5f1d093` | PRAGMA user_version 迁移框架 + load_tags_for_links 批量加载（消除 N+1） |
| R6 | `11ab48d` | Sidebar 拆 CategorySection / TagSection + .nav-item 提到 app.css |

---

## 一、后端 invoke 命名重构（R2，**前端绕过 api.js 直调风险点**）

R2 重命名了 16 个 Tauri 命令（动词在前）。前端 `api.js` 已同步更新，但若有外部脚本/浏览器扩展硬编码命令名将失效。

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| RENAME-001 | 浏览器扩展端到端冒烟 | 通过浏览器扩展按钮添加一条链接 | 主程序的快速添加窗口被唤起，URL/title 已预填 | P0 | R2 |
| RENAME-002 | Bookmarklet 端到端 | 在浏览器书签栏点击「扔给 Links」 | 主程序唤起 quick-add，URL 与标题正确 | P0 | R2 |
| RENAME-003 | 39 命令冒烟（前端） | 任选 5 个核心动作：搜索 / 添加 / 编辑 / 删除 / 导出 | 全部正常无 invoke 报错 | P0 | R2 |

---

## 二、后端 bug 修复（R1）

### 2.1 `import_bookmarks` 持锁 IO 修复

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| IMPORT-001 | HTML 书签导入（小批量） | 设置→导入 → 选择含 10 条链接的 Chrome 书签 HTML | 全部成功导入；分组树正确（多层嵌套保留） | P0 | R1 |
| IMPORT-002 | HTML 书签导入（中等批量） | 导入含 100+ 条链接、3 层嵌套的书签 HTML | 全部成功；导入期间**主窗口仍可滚动/搜索/编辑其他链接**（关键：R1 修复了持锁 IO 的阻塞问题） | P0 | R1 |
| IMPORT-003 | JSON 导入（新格式） | 导入 `{ links: [...], categories: [...] }` | 分组与链接关系正确还原；id 映射不冲突 | P1 | R1 |
| IMPORT-004 | JSON 导入（旧格式裸数组） | 导入仅 `[{...}, {...}]` 链接数组 | 链接成功导入，无分组关联 | P1 | R1 |
| IMPORT-005 | 导入去重 | 导入含重复 URL 的书签文件 | 重复链接只导入一次 | P1 | R1 |
| IMPORT-006 | 导入按钮 spinner | 导入大文件时观察侧边栏底部导入按钮 | 显示旋转 spinner，导入完成后恢复 | P2 | R1 |

### 2.2 `fetcher.rs:213` Windows 代理优先级修复

仅 Windows 平台需要测试。

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| PROXY-001 | Windows 代理元数据抓取 | Windows 系统设置代理（如 127.0.0.1:7890）→ 添加链接 | 元数据通过代理抓取成功 | P0 | R1（Windows） |
| PROXY-002 | Windows 代理 bypass 内网 | 设置 ProxyOverride 含 `*.internal.local` → 添加内网链接 | 直连不走代理；日志显示 "bypassing proxy" | P1 | R1（Windows） |
| PROXY-003 | Windows 代理地址解析 | 注册表 ProxyServer 含特殊字符（如 socks=...）的多协议格式 | 解析出 IP:port 而不被注册表关键字干扰（修复了 R1 之前的优先级 bug） | P1 | R1（Windows） |

### 2.3 共享 HTTP client（R1 把 `fetch_metadata` 与 `check_link_status` 的代理逻辑合并）

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| HTTP-001 | 链接可达性检查（开关开启） | 设置→开启「链接可达性检查」→ 添加一个 404 URL | 链接卡片显示失效告警图标 | P1 | R1 |
| HTTP-002 | 链接可达性检查（开关关闭） | 设置→关闭可达性检查 → 添加 404 URL | 不显示失效图标 | P2 | R1 |
| HTTP-003 | check_duplicate 命令 | 编辑链接 → URL 输入框输入与他人重复的 URL | 显示「已有相同链接」警告 | P1 | R1 |

### 2.4 4 套快捷键 helper 合并（R1 把 4 套 setter 整合）

参见下面 §四「ShortcutEditor」。

---

## 三、DB 迁移框架与 N+1（R5）

### 3.1 PRAGMA user_version 迁移

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| MIGRATE-001 | 全新库启动 | 删除 `links.db` / `links.db-wal` / `links.db-shm`，启动应用 | 应用启动正常；新库 `PRAGMA user_version = 1`；所有表与触发器都建好 | P0 | R5 |
| MIGRATE-002 | 老库（无 user_version）兼容升级 | 用 R5 之前的版本（任意旧 commit）创建数据库并填几条链接，切到 R5+ 版本启动 | 启动正常；数据完整无丢失；user_version 升至 1；新增 link_tags / fts 表与触发器都补齐（若旧库缺）| P0 | R5 |
| MIGRATE-003 | 重复启动 | 重复关闭并启动应用多次 | 启动均正常；不会因迁移幂等性问题报错 | P1 | R5 |
| MIGRATE-004 | 日志检查 | 启动时查看 `links.log` | 出现 `[migrate] current user_version = X`；如有迁移则有 `applying vN` / `vN done` | P2 | R5 |

> 验证 user_version 命令（macOS）：
> ```bash
> sqlite3 ~/Library/Application\ Support/com.links.desktop/links.db "PRAGMA user_version;"
> ```
> Windows：`sqlite3 %APPDATA%\com.links.desktop\links.db "PRAGMA user_version;"`

### 3.2 N+1 批量加载（性能改进）

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| PERF-001 | 大列表分页 | 库内有 200+ 链接，每条带 3-5 个标签，滚动列表加载更多 | 每页加载明显流畅，标签正确显示（**关键**：原 N+1 → 现 1 次 IN 查询） | P0 | R5 |
| PERF-002 | 搜索结果标签 | 搜索一个常见关键词，返回 30+ 条 | 所有结果的标签 chip 都正确显示 | P0 | R5 |
| PERF-003 | 搜索 LIKE 回退路径 | 搜索含特殊字符（如 `?` `*`）的内容 | 结果标签也正确（FTS 失败回退路径同样接入了批量） | P1 | R5 |
| PERF-004 | 统计 Top 3 标签 | 侧边栏点击「统计」 | Top 3 链接的标签正确显示 | P1 | R5 |
| PERF-005 | 全量导出含标签 | 设置→导出 → 选 JSON 或 markdown | 所有链接的标签字段都正确包含 | P1 | R5 |
| PERF-006 | 边界：导出空库 | 删除所有链接后导出 | 不报错；返回空数据 | P2 | R5 |
| PERF-007 | 边界：链接无标签 | 含未关联任何标签的链接出现在列表中 | 标签字段为空数组，UI 不报错 | P2 | R5 |

---

## 四、ShortcutEditor 重构（R3 + R1）

R3 把 SettingsDialog 中 4 套快捷键样板合并为单组件 + 元数据驱动；R1 把后端的 4 套 setter 也合并。

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| SHORTCUT-001 | 快速添加快捷键（默认） | 设置→快捷键，按默认 `Cmd/Ctrl+Shift+L`（mac）或对应组合 | 弹出 quick-add 窗口 | P0 | R1+R3 |
| SHORTCUT-002 | 主窗口快捷键 | 默认 `Cmd/Ctrl+Shift+J` | 唤起主窗口 | P0 | R1+R3 |
| SHORTCUT-003 | Spotlight 快捷键 | 默认 `Cmd/Ctrl+Shift+K` | 弹出 Spotlight 浮层 | P0 | R1+R3 |
| SHORTCUT-004 | 隐藏快捷键 | 默认 `Cmd/Ctrl+Shift+M` | 隐藏当前主窗口 | P0 | R1+R3 |
| SHORTCUT-005 | 录制新快捷键 — 4 个全部 | 设置→点击任一快捷键的「修改」→ 按下新组合键 → 保存 | 显示已录入的组合（mac 用符号 ⌘⌥⇧；win 用文字）；保存成功；旧快捷键失效，新快捷键生效 | P0 | R3 |
| SHORTCUT-006 | 录制取消 | 录制中点击「取消」 | 回到原值，不保存 | P1 | R3 |
| SHORTCUT-007 | 录制冲突保护 | 录制时按下与另一类相同的组合 | 保存时后端报错；UI 显示「快捷键设置失败，请重试」 | P1 | R3 |
| SHORTCUT-008 | Esc 关闭设置 | 录制中按 Esc | 关闭整个设置弹窗（**关键**：父组件 keydown 转发逻辑要正确分发给录制中的 Editor） | P1 | R3 |
| SHORTCUT-009 | 设置弹窗内 Tab 键不影响 | 设置弹窗中按 Tab | 不触发录制（仅在 recording 状态消费 keydown） | P2 | R3 |
| SHORTCUT-010 | 重启后快捷键持久化 | 改一个快捷键 → 重启应用 | 启动后新快捷键依然生效 | P0 | R1（config.save） |

---

## 五、QuickAdd 复用 LinkForm（R3）

R3 把 QuickAdd 从 518 行重写为 186 行，复用 LinkForm 的 standalone 模式。

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| QADD-R3-001 | 快捷键唤起后表单为空 | 唤起 quick-add → 第一次出现 | URL/标题/分组/标签全部为空 | P0 | R3 |
| QADD-R3-002 | 重复唤起清空 | 第一次唤起后输入一些内容 → 关闭 → 再次唤起 | 表单已清空（reset） | P0 | R3 |
| QADD-R3-003 | URL 抓取自动填充 | quick-add 输入有效 URL | 标题/描述/favicon 自动填充 | P0 | R3 |
| QADD-R3-004 | 已编辑字段不被覆盖 | 先在标题手填内容 → 再粘贴 URL | URL 触发抓取后**不覆盖**用户已编辑的标题 | P0 | R3 |
| QADD-R3-005 | 重复检测警告 | quick-add 输入已存在的 URL | 显示「已有相同链接」 | P1 | R3 |
| QADD-R3-006 | 提交保存 | 填写完成 → 回车或点保存 | 链接创建成功；窗口关闭；主窗口列表自动刷新（emit links-changed） | P0 | R3 |
| QADD-R3-007 | 取消按钮 | 点取消 | 窗口关闭，未保存 | P1 | R3 |
| QADD-R3-008 | Esc 关闭 | 输入内容后按 Esc | 窗口关闭，未保存 | P1 | R3 |
| QADD-R3-009 | IME 守卫 — 中文输入回车不提交 | URL 框已填，标题框中文输入法正在组词时按回车 | 不提交表单，组词正常完成（**关键**：R3 抽 createImeGuard 后两端一致行为） |  P0 | R3 |
| QADD-R3-010 | 浏览器扩展 deep-link 预填 | 浏览器扩展按钮捕获页面 → 唤起 quick-add | URL/标题已预填；自动触发抓取与去重检测（R3.b 接口 triggerFetch）| P0 | R3 |
| QADD-R3-011 | 冷启动 deep-link | 浏览器扩展首次唤起（应用未运行） | 冷启动后 quick-add 自动出现并预填 | P1 | R3 |
| QADD-R3-012 | 保存失败提示 | 模拟 createLink 失败（如 URL 极端长触发约束） | 显示「保存失败 ✗」；按钮重新可点（setSaving(false)） | P2 | R3 |
| QADD-R3-013 | quick-add 主题跟随 | 设置主题为系统/亮/暗 → 唤起 quick-add | 主题与主窗口一致；切主题时 quick-add 同步更新（themeStore 跨窗口同步） | P1 | R3 |

---

## 六、主题统一管理（R3 themeStore）

R3 把 App / QuickAdd / Spotlight 三处 `apply_theme` 合并为单一 store。

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| THEME-001 | 切换为亮色 | 设置→外观→亮色 | 立即变亮色；主窗口 + quick-add + spotlight 三个窗口都同步（跨窗口 theme-changed 事件） | P0 | R3 |
| THEME-002 | 切换为暗色 | 设置→外观→暗色 | 立即变暗色；三窗口同步 | P0 | R3 |
| THEME-003 | 跟随系统 | 设置→外观→跟随系统 → 切换 macOS 系统外观 | 主题跟随系统切换（prefers-color-scheme 监听） | P1 | R3 |
| THEME-004 | 主题持久化 | 切换主题 → 重启应用 | 启动后主题状态保留 | P0 | R3 |
| THEME-005 | 老配置兼容 | 在 config.json 中保留 `dark-mode=true`（无 `theme-mode`），启动应用 | 应用读取为 dark；不报错 | P1 | R3（legacy fallback） |
| THEME-006 | 暗色搜索高亮可读性 | 暗色模式下搜索关键词 | 高亮颜色清晰可读（不再是亮黄色刺眼） | P2 | 历史 + R3 不破坏 |
| THEME-007 | sidebar 暗色模式样式 | 暗色 + 鼠标 hover 分组/标签项 | hover/active 背景色正确（R6 把 :global(.dark) 选择器留在 Sidebar 父） | P1 | R6 |

---

## 七、Sidebar 拆分（R4 + R6）

R4 抽 Brand；R6 抽 CategorySection / TagSection；总计 Sidebar 主组件从 1213 行 → 266 行。

### 7.1 Brand 头部

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| BRAND-001 | 版本号显示 | 启动应用 | sidebar 顶部显示「Links v1.3.x」 | P1 | R4 |
| BRAND-002 | 设置齿轮 | 点击齿轮 | 打开设置弹窗 | P0 | R4 |
| BRAND-003 | 更新铃显示 | 模拟有可用更新（手动改 update_available） | 铃图标出现；带 pulse-glow 动画 | P2 | R4 |
| BRAND-004 | 更新铃点击 | 点击更新铃 | 触发 onupdate 回调，弹出更新对话框 | P1 | R4 |

### 7.2 CategorySection（**关键**：包含拖拽 + 重命名 + 子分组创建）

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| CAT-R6-001 | 创建顶级分组 | 点击+号 → 输入名称 → 回车 | 分组出现在树顶层 | P0 | R6 |
| CAT-R6-002 | 创建子分组 | 鼠标 hover 一个分组 → 点击+号 → 输入 → 回车 | 子分组创建；父分组自动展开 | P0 | R6 |
| CAT-R6-003 | 重命名分组 | hover → 点击编辑图标 → 改名 → Enter | 名称更新 | P0 | R6 |
| CAT-R6-004 | 重命名取消 | 编辑中按 Esc 或点其他 | 不保存，恢复原名 | P1 | R6 |
| CAT-R6-005 | 删除两阶段确认 | hover 文件夹图标 → 变红色删除图标 → 第一次点击 | 显示「再点一下就删除」；鼠标移走后回归 | P0 | R6 |
| CAT-R6-006 | 删除两阶段 — 确认删除 | 接上 → 第二次点击删除图标 | 分组被删；其下链接的 category_id 变 NULL | P0 | R6 |
| CAT-R6-007 | 拖拽分组改父级 | 拖拽 A 到 B 分组上 | A 变为 B 的子分组 | P0 | R4+R6 |
| CAT-R6-008 | 拖拽到根级 | 拖拽子分组到顶部「移到根级」区域 | 升为根级 | P0 | R4+R6 |
| CAT-R6-009 | 拖拽防循环 | 拖拽父分组到自己的子分组 | drop-target 不高亮，无效操作（isCategoryDescendant 防止） | P0 | R4 |
| CAT-R6-010 | 拖拽 ghost 显示 | 拖拽中观察 | 鼠标右下出现幽灵浮层显示分组名；阴影使用 `var(--shadow-drag)` | P1 | R4 |
| CAT-R6-011 | 拖拽期间不能重命名 | 拖拽中点编辑图标 | 不进入编辑模式（canStartDrag → isEditing 互斥） | P1 | R6 |
| CAT-R6-012 | 节折叠 | 点击「分组管理」标题 | 整个 CategorySection 折叠/展开 | P1 | R6 |
| CAT-R6-013 | 节内搜索（>10 个分组时显示） | 创建超过 10 个分组 → 在搜索框输入 | 仅显示匹配的分组 | P1 | R6 |
| CAT-R6-014 | 未分组项点击 | 点击「未分组」 | 主列表只显示无分组的链接（uncategorized_only=true） | P0 | R6 |
| CAT-R6-015 | 创建空名提示循环 | 不输入名字直接回车 | placeholder 在「给我一点输入」↔「你是认真的吗？」间循环（cyclePlaceholder） | P2 | R4 |
| CAT-R6-016 | 创建包含 `/` | 输入「a/b」并回车 | 显示「分组名不能包含 /」 | P2 | R6 |
| CAT-R6-017 | 创建重复名 | 输入已有分组名（不区分大小写）| 显示「已经有这个分组了」 | P2 | R6 |
| CAT-R6-018 | 嵌套展开/收起 | 点击有子项的分组前的箭头 | 子项展开/收起（`expanded` Set 切换） | P1 | R6 |
| CAT-R6-019 | 子分组缩进 | 多层嵌套时观察 | 每层缩进 12px；展开标识旋转动画正常 | P2 | R6 |
| CAT-R6-020 | 暗色模式 active 颜色 | 暗色 + 选中某分组 | 背景色变 cat-soft 暗色版（:global(.dark) 选择器留在 Sidebar 父） | P1 | R6 |

### 7.3 TagSection

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| TAG-R6-001 | 创建标签 | 点击+号 → 输入 → 回车 | 标签出现 | P0 | R6 |
| TAG-R6-002 | 重命名标签 | hover → 点击编辑 → 改名 → Enter | 名称更新；所有关联链接的标签也更新 | P0 | R6 |
| TAG-R6-003 | 删除两阶段 | hover tag → 红色删除 → 点 1 次 → 提示 → 点 2 次 | 标签删除；link_tags 记录级联删除 | P0 | R6 |
| TAG-R6-004 | 标签筛选 | 点击侧边栏的某标签 | 列表只显示带该标签的链接 | P0 | R6 |
| TAG-R6-005 | 无标签项 | 点击「无标签」 | 列表只显示无任何标签的链接 | P0 | R6 |
| TAG-R6-006 | 节折叠 | 点击「标签管理」标题 | 整节折叠/展开 | P1 | R6 |
| TAG-R6-007 | 节内搜索 | >10 个标签时输入搜索 | 仅匹配项可见 | P1 | R6 |
| TAG-R6-008 | 创建重复标签 | 输入已有名 | 显示「已经有这个标签了」 | P2 | R6 |
| TAG-R6-009 | 暗色模式 active 颜色 | 暗色 + 选中某标签 | 背景色变 accent-soft 暗色版 | P1 | R6 |

### 7.4 顶部 nav（全部 / 特别关注 / 统计）

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| NAV-R6-001 | 全部链接 | 点击「全部链接」 | 显示所有链接，无筛选 | P0 | R6 |
| NAV-R6-002 | 特别关注 | 点击「特别关注」 | 仅显示 is_favorite=true 的链接 | P0 | R6 |
| NAV-R6-003 | 统计视图 | 点击「统计」 | 切换到 StatsPanel 视图（R4） | P0 | R6+R4 |
| NAV-R6-004 | nav-item 样式一致 | 同时观察顶部 3 按钮、CategorySection 项、TagSection 项 | 高度、字号、hover 反馈一致（R6 把 .nav-item 提到 app.css） | P1 | R6 |

---

## 八、StatsPanel（R4）

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| STATS-001 | 数据加载 | sidebar 点击「统计」 | 显示总数、本周新增；Top 3 链接按 click_count 排序 | P0 | R4 |
| STATS-002 | 加载占位 | 切换到统计视图的瞬间 | 短暂显示「加载中...」直到 sidebar_stats 返回 | P2 | R4 |
| STATS-003 | Top 3 显示 | 多次点击同一链接 | 该链接进入 Top 3，click_count 正确 | P1 | R4 |
| STATS-004 | Top 3 标签 | Top 3 中的链接含标签 | 标签字段正确显示（R5 批量加载接入了 get_stats） | P1 | R4+R5 |
| STATS-005 | 空库 | 删空所有链接 → 切到统计 | 显示总数 0；不显示 Top 3 列表 | P2 | R4 |

---

## 九、CloseDialog（R4）

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| CLOSE-001 | 询问关闭行为 | 设置→关闭行为→「每次询问」→ 关闭主窗口 | 弹出 CloseDialog；显示「要走了吗？」+ 两个按钮 | P0 | R4 |
| CLOSE-002 | 最小化到托盘 | CloseDialog 中点「最小化到托盘」 | 主窗口隐藏；托盘图标存在；可重新唤起 | P0 | R4 |
| CLOSE-003 | 退出应用 | CloseDialog 中点「退出应用」 | 应用完全退出 | P0 | R4 |
| CLOSE-004 | 点击遮罩取消 | CloseDialog 出现 → 点击灰色背景区域 | 对话框关闭，主窗口未关闭 | P1 | R4 |
| CLOSE-005 | Esc 关闭 | CloseDialog 出现 → Esc | 对话框关闭，主窗口未关闭 | P1 | R4 |
| CLOSE-006 | 直接最小化策略 | 设置→关闭行为→「最小化到托盘」 → 关闭 | 不弹 CloseDialog，直接最小化 | P1 | R4 |
| CLOSE-007 | 直接退出策略 | 设置→关闭行为→「直接退出」 → 关闭 | 不弹 CloseDialog，直接退出 | P1 | R4 |
| CLOSE-008 | scrim 颜色 | 暗色模式下打开 CloseDialog | 遮罩背景使用 `--scrim-bg` 暗色版（R3 替换硬编码）| P2 | R3+R4 |

---

## 十、ReleaseNotesDialog（R4）

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| RN-001 | 升级触发说明弹窗 | 修改 config.json 的 `last-known-version` 为更小值 → 重启 | 启动后弹出 ReleaseNotesDialog；显示当前版本 + Markdown 渲染的更新说明 | P0 | R4 |
| RN-002 | Markdown 渲染 | 弹窗中查看内容 | 标题、列表、链接等 markdown 元素正确渲染（R4 把 marked import 移到 ReleaseNotesDialog） | P1 | R4 |
| RN-003 | 知道了关闭 | 点底部「知道了」 | 弹窗关闭 | P1 | R4 |
| RN-004 | Esc / 点击遮罩关闭 | 任一方式 | 弹窗关闭 | P1 | R4 |
| RN-005 | 无更新说明文案 | last-update-notes 为空 | 显示「暂无更新说明」 | P2 | R4 |

---

## 十一、LinkCard 工具抽取（R3）

R3 抽出 `formatRelativeTime` / `formatLinkAs` / `getDomain`。

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| CARD-R3-001 | 最后打开时间 — 刚刚 | 打开链接，立即查看卡片 | 显示「刚刚」 | P1 | R3 |
| CARD-R3-002 | 最后打开 — N 分钟前 | 5 分钟后看 | 显示「5 分钟前」（带空格，主窗口风格） | P1 | R3 |
| CARD-R3-003 | 最后打开 — 7 天后 | 修改 `last_opened_at` 为 8 天前 | 显示「MM-DD HH:mm」格式（短格式） | P2 | R3 |
| CARD-R3-004 | 触碰显示具体时间 | hover 最后打开时间 | tooltip 显示完整时间（formatAbsoluteTime） | P2 | R3 |
| CARD-R3-005 | 域名显示 | 卡片显示 url 域名 | 正确去除 `www.`（R3 getDomain stripWww=true） | P1 | R3 |
| CARD-R3-006 | 复制为 URL | 卡片→分享菜单→「URL」 | 剪贴板内容为纯 URL | P1 | R3 |
| CARD-R3-007 | 复制为 Markdown | 选「Markdown」 | 剪贴板为 `[标题](url)`，标题中的 `[]*_~` 转义正确 | P1 | R3 |
| CARD-R3-008 | 复制为 HTML | 选「HTML」 | 剪贴板为 `<a href="...">标题</a>`；标题内 HTML 实体转义 | P1 | R3 |
| CARD-R3-009 | confirm-overlay 颜色 | 删除卡片时观察确认对话框遮罩 | 使用 `--scrim-bg` 而非硬编码 rgba | P2 | R3 |

---

## 十二、Spotlight 工具接入（R3）

R3 让 Spotlight 复用 themeStore / formatRelativeTime / getDomain。

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| SP-R3-001 | Spotlight 域名展示 | 搜索一个 www 域名 | 域名保留 `www.`（R3 getDomain stripWww=false 与主窗口风格不同） | P2 | R3 |
| SP-R3-002 | Spotlight 时间格式 | 搜索结果显示最后打开时间 | 显示「3分钟前」（无空格，spaceBeforeUnit=false） | P2 | R3 |
| SP-R3-003 | Spotlight 主题切换跟随 | 主窗口切主题 → 唤起 Spotlight | Spotlight 主题与主窗口一致 | P1 | R3 |
| SP-R3-004 | Spotlight 上下键导航 | 在结果列表上下键 | 高亮项切换正常 | P0 | 历史 |
| SP-R3-005 | Spotlight Tab 键切排序 | 按 Tab 键 | 排序选项循环切换（最近更新 / 最多访问 / 最近打开） | P1 | 历史 |

---

## 十三、IME 守卫（R3）

R3 抽 `createImeGuard` 后 LinkForm / QuickAdd 共享同一逻辑。

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| IME-001 | 主表单标题中文输入 | 编辑链接 → 标题中文输入法组词时按 Enter | 不提交，组词正常完成 | P0 | R3 |
| IME-002 | quick-add 中文输入 | quick-add → 标题中文输入法组词时按 Enter | 同上 | P0 | R3 |
| IME-003 | 描述长输入中文 | 描述 textarea 中长篇中文输入回车 | 不提交（IME 守卫尾巴防抖期 200ms） | P1 | R3 |
| IME-004 | 守卫释放 | 组词结束 200ms 后再按 Enter | 正常提交 | P1 | R3 |

---

## 十四、视觉一致性回归（R3 + R6）

R3 替换 8 处硬编码颜色为 CSS 变量；R6 把 `.nav-item` 提到 app.css。

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 来源 |
|---|---|---|---|---|---|
| VIS-001 | 模态遮罩颜色一致 | 依次打开 LinkForm / SettingsDialog / ExportDialog / CloseDialog / 卡片删除确认 | 所有遮罩用同一 `--scrim-bg` 颜色（暗/亮模式都一致） | P1 | R3 |
| VIS-002 | 拖拽 ghost 阴影 | 拖动一个分组 | 浮层阴影使用 `--shadow-drag` 在亮/暗模式都正确 | P2 | R3 |
| VIS-003 | spotlight.html 启动闪烁 | Spotlight 快捷键唤起 | 无主题闪白；theme-ready 类生效后才显示（R3 修了重复 CSS 块） | P1 | R3 |
| VIS-004 | sidebar footer 样式 | 观察暗/亮模式下的导出 / 导入 / 主题按钮区 | 背景色一致；hover 反馈正确（R3 修了重复 CSS 块） | P1 | R3 |
| VIS-005 | nav-item hover 一致性 | 顶部 nav + 分组项 + 标签项 hover | 所有 nav-item 同一交互反馈（背景与文字色一致） | P2 | R6 |
| VIS-006 | titlebar 暗色 | macOS 暗色模式 | 顶部 36px titlebar 区域颜色为 `var(--bg-0)`（无冗余 .dark 覆盖）| P2 | R3 |
| VIS-007 | UpdateDialog danger 色 | 触发更新失败 | 错误文字使用 `var(--danger)` 不带 fallback（R3 删除了 fallback） | P2 | R3 |

---

## 十五、跨重构融合点（高风险）

这些场景跨多次重构，是回归概率最高的位置：

| ID | 测试用例 | 步骤 | 预期 | 优先级 | 涉及重构 |
|---|---|---|---|---|---|
| MIX-001 | 大批量导入 + 即时搜索 | 导入 200+ 链接的同时立即在主窗口搜索 | 搜索响应正常（R1 修复 import 持锁后才能并行）；标签批量加载（R5）；列表分页正常 | P0 | R1+R5 |
| MIX-002 | 拖拽 + 重命名互斥 | 一只手按住拖拽中、另一只点击编辑图标 | 编辑模式不进入（R4 抽出的 canStartDrag + R6 的 isEditing） | P0 | R4+R6 |
| MIX-003 | quick-add + 主窗口主题同步 | quick-add 唤起时 → 主窗口切主题 | quick-add 主题立即跟随（R3 themeStore 跨窗口事件） | P1 | R3 |
| MIX-004 | 设置改快捷键 → 立即生效 | 录入新快捷键保存 → 不重启直接按新快捷键 | 新快捷键唤起对应窗口（R1 update_shortcut helper 注销旧值并注册新值） | P0 | R1+R3 |
| MIX-005 | 老库无 user_version + 浏览器扩展 | 用 R1 之前的代码建库 → 切到 R6 → 用浏览器扩展添加链接 | 自动迁移 user_version=1；invoke 命名兼容；deep-link 正常预填 | P0 | R2+R5 |
| MIX-006 | sidebar Brand 与 update 协同 | 应用启动→后台检查到更新→sidebar 出现更新铃 | 铃显示+点击弹更新对话框（R4 Brand 抽出后接口仍正确） | P1 | R4 |
| MIX-007 | 暗色 + 拖拽 + scrim | 暗色模式 → 拖动分组 → 拖到自身后代 | drop-target 不亮；ghost 阴影暗色版可见 | P2 | R3+R4+R6 |
| MIX-008 | Spotlight + Cmd+Enter 定位 | Spotlight 搜索结果上 Cmd+Enter | 唤起主窗口并定位到该链接 | P1 | 历史 |
| MIX-009 | 关闭主窗口策略一致 | 在 quick-add 唤起期间通过快捷键关闭主窗口 | 关闭行为按设置生效；不影响 quick-add | P1 | R4 |
| MIX-010 | 导出含子分组的 HTML 书签 | 导出格式选 HTML | Netscape Bookmark 格式正确；嵌套结构保留；标签批量加载（R5） | P1 | R5 |

---

## 执行小贴士

1. **测试前**：备份 `links.db` 与 `config.json`，避免污染。
2. **回归定位**：发现失败先查表中「来源」对应的 commit，使用 `git diff <commit>^ <commit>` 看具体改动。
3. **观察日志**：`tail -f links.log`（macOS：`~/Library/Application Support/com.links.desktop/links.log`）。
4. **DB 状态检查**：`sqlite3 path/to/links.db "PRAGMA user_version; SELECT COUNT(*) FROM links;"`。
5. **断网场景**：可以用系统代理工具（如 ProxyMan / Fiddler）拦截 fetch_metadata 来模拟可达性检查的不同结果。

---

## 优先级汇总

- **P0（必测）**：约 40 项 — 核心数据流（创建/编辑/删除/搜索/导入）、迁移兼容、跨窗口主题同步、快捷键全套
- **P1（重要）**：约 35 项 — 边界与回归点
- **P2（体验）**：约 20 项 — 视觉一致性与提示文案

整套清单约 95 项，全套通过约需 1.5-2 小时。如时间紧张，建议优先跑：
1. MIX-001 / MIX-002 / MIX-005（跨重构融合最高风险）
2. RENAME-001 / RENAME-002（外部接入兼容性）
3. MIGRATE-002（老库升级）
4. IMPORT-002（持锁 IO 修复验证）
5. PERF-001（N+1 性能改进）
6. SHORTCUT-005 / QADD-R3-002 / QADD-R3-009（高频用户场景）
