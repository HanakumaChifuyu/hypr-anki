# HyprAnki design tokens

以 `output/hypranki-integrated-titlebar-design.png` 为视觉方向：深蓝底色、柔白文字、淡紫强调色。当前只定义深色主题。

## 配色

组件使用 `Colors` 语义色，`Palette` 仅负责提供原始色阶。保留原有语义 token 名称，蓝色继续用于信息提示。

| 用途 | Token | 默认值 |
| --- | --- | --- |
| 侧边栏及其顶部窗口区域 | `bg-sidebar` | `#101725` |
| 内容及其顶部窗口区域 | `bg-content` | `#1c2636` |
| 嵌入式容器 | `bg-surface` | `#172131` |
| 装饰分隔线 | `border-default` | `#2c394e` |
| 主要文字 | `text-primary` | `#f8fafc` |
| 次要文字 | `text-secondary` | `#cbd5e1` |
| 辅助说明 | `text-muted` | `#94a3b8` |
| 选中背景 / 前景 | `bg-selected` / `text-selected` | `#27253e` / `#cbb5fa` |
| 主按钮背景 / 前景 | `bg-primary` / `text-on-primary` | `#cbb5fa` / `#101725` |

- `*-selected` 用于导航选中态，`*-primary` 背景搭配 `*-on-primary` 深色前景。
- 交互状态提供默认、hover、pressed；disabled 使用专门的弱化颜色。
- `border-default` 仅作装饰分隔；需要辨识的控件轮廓使用 `border-control`，键盘焦点使用 `focus-ring`。
- 状态色有实色与低强调两组：`bg-success` 搭配 `text-on-success`，`bg-success-subtle` 搭配 `text-success`，其余状态同理。
- 标签、学习进度、保存状态分别使用 `bg-badge` / `text-badge`、`progress-track` / `progress-fill`、`status-saved`。
- 顶部背景沿用所属区域的背景色，不单独引入灰色标题栏色。

已核对 44 组预期的文字 / 背景搭配，对比度最低为 5.18:1（`text-muted` / `bg-subtle`）。这不代表任意 token 都可互相组合：`text-muted` 用于区域背景上的说明文字，按下态使用 `text-pressed` 等对应前景；disabled 不计入正常文字对比度检查。

## 字体

默认 `font-family: ""`，使用运行环境的默认字体；不硬编码可能未安装的 SF Pro，也不把 Linux 默认字体指定为微软雅黑。需要固定跨平台字形时，应先引入有许可的字体资源，再覆盖此 token。

字号为 Slint 逻辑像素，随显示缩放处理。正文、导航及中文不额外加字距；仅英文分组标签使用 `overline-tracking: 1px`。

| 用途 | 字号 token | 字号 | 字重 token | 字重 |
| --- | --- | --- | --- | --- |
| 页面标题 | `title-size` | 28px | `title-weight` | 600 |
| 区块标题 | `section-size` | 20px | `section-weight` | 600 |
| 应用名称 | `brand-size` | 20px | `brand-weight` | 600 |
| 引导正文 | `body-large-size` | 16px | `body-large-weight` | 400 |
| 正文 | `body-size` | 14px | `body-weight` | 400 |
| 导航 | `nav-size` | 14px | `nav-weight` | 500 |
| 按钮 | `button-size` | 14px | `button-weight` | 600 |
| 标签 | `label-size` | 13px | `label-weight` | 500 |
| 辅助说明 | `caption-size` | 12px | `caption-weight` | 400 |
| 分组标签 | `overline-size` | 12px | `overline-weight` | 500 |
| 统计数字 | `metric-size` | 32px | `metric-weight` | 600 |

字体属性以 [Slint 字体说明](https://docs.slint.dev/latest/docs/slint/guide/development/fonts/) 和 [Window 默认字体属性](https://docs.slint.dev/latest/docs/slint/reference/window/window/) 为准。系统缺少中文字库时仍需安装相应字体；这里不打包字体文件。

## 使用与预览

`theme.slint` 统一导出 `Colors`、`Palette`、`Typography`、`Space`、`Shape`。窗口设置 `default-font-*`，每个文字组件明确指定语义颜色；标题等组件再覆盖字号和字重。

```slint
import { Colors, Typography } from "../tokens/theme.slint";

export component SectionHeading inherits Text {
    font-family: Typography.font-family;
    font-size: Typography.section-size;
    font-weight: Typography.section-weight;
    color: Colors.text-primary;
}
```

在项目根目录运行：

```sh
nix develop -c slint-viewer crates/hello_slint/ui/tokens/tokens.preview.slint
```

预览页包含中英文字样、文字层级、背景层级、交互状态、提示色与进度条。侧边栏默认总宽 240px、导航行高 44px、导航图标 20px，应用标题图标 32px；尺寸由 `Shape` 和 `Space` 控制。
