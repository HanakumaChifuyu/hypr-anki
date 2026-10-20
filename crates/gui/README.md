# GUI

Tauri 2 + React + TypeScript + Vite + Tailwind CSS。开发工具由仓库根目录的 `flake.nix` 管理，前端依赖由 npm 和 `ui/package-lock.json` 锁定。

## 启动

在仓库根目录进入开发环境并安装前端依赖：

```sh
nix develop
npm --prefix crates/gui/ui ci
```

启动桌面应用（自动启动 Vite，支持热更新）：

```sh
cd crates/gui
cargo tauri dev
```

只在浏览器中开发页面：

```sh
npm --prefix crates/gui/ui run dev
```

访问 http://127.0.0.1:1420。浏览器中不能直接调用 Tauri 原生 API。

## 构建

在 Nix 开发环境中：

```sh
cd crates/gui
cargo tauri build
```

Tauri CLI 自动执行前端类型检查和构建，再将 `ui/dist` 嵌入桌面应用；当前未启用安装包生成。

单独验证前端：

```sh
npm --prefix crates/gui/ui run build
```

如果直接使用 `cargo run -p gui --features custom-protocol`，必须先构建前端；普通 Cargo 命令不会执行 Tauri CLI 的前端钩子。

## 目录

- `src/main.rs`：Tauri 入口，后续注册 Rust commands。
- `build.rs`：Tauri 构建脚本。
- `tauri.conf.json`：窗口、Vite 开发地址、前端构建钩子和 CSP。
- `ui/index.html`：HTML 入口，加载 `src/main.tsx`。
- `ui/src/main.tsx`：挂载 React。
- `ui/src/App.tsx`：路由定义。
- `ui/src/components/navigation/MainNavigation.tsx`：顶部导航。
- `ui/src/components/layout/AppLayout.tsx`：公共布局和 Outlet。
- `ui/src/components/ui/`：按需添加通用 UI 组件。
- `ui/src/pages/`：各路由的页面组件。
- `ui/styles.css`：Tailwind 入口、设计 token 和基础规则。
- `ui/vite.config.ts`：固定开发端口 1420，端口占用时报错。
- `asset/`：应用图标等资源。

当前使用 React Router 的 HashRouter，提供 `/deck`、`/browser`、`/status`、`/setting` 四个页面入口，根路径和未知路径回到 `/deck`。公共布局中的顶部导航通过 NavLink 自动显示选中状态，Outlet 承载页面内容；Deck 页面提供卡组树，其他页面暂为占位。尚未引入状态库和 UI 组件库。`@tauri-apps/api` 已安装，接入 Rust commands 时可从 `@tauri-apps/api/core` 导入 `invoke`。

## 环境

`flake.nix` 提供 Rust、Node.js 24、npm 和 Tauri CLI，并在 Linux 下提供 GTK / WebKitGTK 等依赖。macOS 还需要 Xcode Command Line Tools。发布 CSP 允许本地脚本和内联脚本，以支持 next-themes 初始化；开发 CSP 另允许 Vite 的 WebSocket 连接。

## 样式约定

使用 Tailwind CSS 的 Vite 插件。颜色、圆角、阴影和布局宽度在 `ui/styles.css` 的 `@theme` 中定义，例如 `--color-surface` 对应 `bg-surface`。组件和页面使用工具类，全局 CSS 不放组件选择器。

## 主题

`next-themes` 默认跟随操作系统深浅色，并监听系统变化。配置在 `ui/src/theme.ts`，独立存储键为 `hypranki-theme`。后续设置页可通过 `useTheme().setTheme("light" | "dark" | "system")` 修改选择。

深浅两套完整配色在 `ui/src/styles/themes.css`；`ui/styles.css` 使用 `@theme inline` 将动态 token 映射为 Tailwind 类，字体、尺寸和布局不随配色改变。

next-themes 使用原生 ThemeProvider 和内联初始化脚本，无自定义 Vite 插件。CSP 的 `script-src` 允许 `self` 和 `unsafe-inline`，并通过 `dangerousDisableAssetCspModification: ["script-src"]` 关闭该指令的自动 nonce/hash 注入，避免浏览器忽略 `unsafe-inline`。这会放行所有内联脚本；后续展示不可信卡片 HTML 时，应单独设计净化与隔离。

## 卡组树

`ui/src/types/deck.ts` 描述业务查询返回的前端 Deck，使用 `kind: "deck" | "group"` 显式区分卡组和分组（不按是否有子节点推断），并在数据库实体字段上增加 `new_count`（今日剩余新学）和 `review_count`（当前待复习）。计数由业务层计算，前端不重新汇总父子计数；当前数字 ID 必须在 JavaScript 安全整数范围内。`DeckTree` 通过 `decks: readonly Deck[]` 接收父节点先于子节点的先序列表，用 Map 单次遍历组树。

`DeckTree` 放在 `components/decks/`，树形交互使用 react-arborist / react-dnd，外观使用 Tailwind token。支持折叠、键盘导航，以及整棵子树的拖拽、改父级、同级排序和拖回根层级。

- `onMove({ deck_id, parent_id, index })`：移动意图，index 为移除源节点后的目标同级索引。
- `onCollapsedChange(id, collapsed)`：折叠状态变化。
- `onActivate(deck)`：激活卡组，留给后续概览或学习页面。
- `onSettings(deck)`：末尾 `…` 设置按钮，进入独立的 `/deck/:deckId/settings` 卡组／分组设置占位页；`/setting` 只负责应用设置。

页面负责数据更新与持久化，组件不调用 Tauri。当前 DeckPage 使用 `features/decks/sample-decks.ts` 示例列表，拖拽和折叠仅保存在页面内存中。后端尚无顺序字段或排序接口，需先定义持久化规则再接数据库。计数目前也是示例值。

窗口配置 `dragDropEnabled: false` 是为让浏览器拖拽事件在 Windows 上正常工作，不代表禁用页面内拖拽。

验证树构建、子树移动和循环保护：`npm --prefix crates/gui/ui test`。
