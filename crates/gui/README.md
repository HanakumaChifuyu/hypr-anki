# GUI

Tauri 2 + React + TypeScript + Vite。开发工具由仓库根目录的 `flake.nix` 管理，前端依赖由 npm 和 `ui/package-lock.json` 锁定。

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
- `ui/src/App.tsx`：根组件，从这里开始添加页面。
- `ui/styles.css`：全局样式。
- `ui/vite.config.ts`：固定开发端口 1420，端口占用时报错。
- `asset/`：应用图标等资源。

当前使用 React Router 的 HashRouter，提供 `/deck`、`/browser`、`/status`、`/setting` 四个页面入口，根路径和未知路径回到 `/deck`。公共布局中的顶部导航通过 NavLink 自动显示选中状态，Outlet 承载页面内容；各页面暂为占位，不包含卡组树。尚未引入状态库和 UI 组件库。`@tauri-apps/api` 已安装，接入 Rust commands 时可从 `@tauri-apps/api/core` 导入 `invoke`。

## 环境

`flake.nix` 提供 Rust、Node.js 24、npm 和 Tauri CLI，并在 Linux 下提供 GTK / WebKitGTK 等依赖。macOS 还需要 Xcode Command Line Tools。发布 CSP 保持严格，开发 CSP 单独允许 React Refresh 和 Vite 的 WebSocket 连接。
