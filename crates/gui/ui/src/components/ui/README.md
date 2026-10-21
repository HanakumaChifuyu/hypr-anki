# 通用 UI 组件

这里存放按需增加的 Button、Input 等基础组件。组件样式使用 Tailwind 和全局语义 token，不依赖具体路由或业务数据。

应用导航放在 `../navigation/`，应用布局放在 `../layout/`。

## 右键菜单

`ContextMenu.tsx` 基于 Radix Context Menu，支持主题、禁用项、危险操作、分隔线和子菜单。菜单内容通过 Portal 渲染到 body，不会被列表的 overflow 裁剪。

```tsx
import {
  ContextMenu,
  ContextMenuTrigger,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
} from "./ContextMenu";

<ContextMenu>
  <ContextMenuTrigger asChild>
    <div>在这里右键</div>
  </ContextMenuTrigger>
  <ContextMenuContent>
    <ContextMenuItem onSelect={handleRename}>重命名</ContextMenuItem>
    <ContextMenuItem disabled>暂不可用</ContextMenuItem>
    <ContextMenuSeparator />
    <ContextMenuItem variant="danger" onSelect={handleDelete}>删除</ContextMenuItem>
  </ContextMenuContent>
</ContextMenu>
```

使用 `onSelect` 处理操作，可同时支持鼠标和键盘。`asChild` 会将触发行为附加到唯一的子元素；自定义子组件需要转发 props 和 ref 到 DOM。

DeckTree 单击行只选中，双击行通过 `onActivate(deck)` 通知调用方。树获得焦点时，Enter 激活当前卡组，或展开/收起当前文件夹；重命名通过右键菜单进入。行内编辑时 Enter 仍用于确认名称。拖拽开始时移除 DOM 焦点并清除 Arborist 的树焦点状态，保留选中状态。

子菜单由 `ContextMenuSub`、`ContextMenuSubTrigger`、`ContextMenuSubContent` 组合。`ContextMenuShortcut` 仅显示快捷键提示，不注册快捷键。图标可直接放在菜单项内。

业务数据和重命名、删除、新建等逻辑由调用方负责。卡组菜单在 `../decks/DeckContextMenu.tsx`，当前由 DeckPage 操作内存中的演示数据，刷新后恢复。

新建草稿行位于右键目标下方：目标为卡组时创建同级项目，目标为文件夹时展开并创建第一个子节点。`create` 操作额外携带 `afterId`，父组件通过 `insertDeck(decks, deck, afterId)` 保持确认后的插入位置。

DeckTree 使用 react-arborist 的 `node.edit()`、`node.submit()`、`node.reset()` 管理行内编辑，统一通过 `onAction(action: DeckMenuAction)` 通知父组件。`rename` 携带确认后的 `id` 和 `name`；`create` 携带确认后的草稿 `deck`，由父组件分配正式 ID 和时间戳并写入列表；`delete` 携带目标 `deck`，由 DeckPage 打开确认弹窗，确认后删除。菜单命令在 DeckTree 内部统一处理，展开和折叠通过 `onCollapsedChange` 同步状态。新建先插入仅用于展示的临时行，取消不修改正式数据。Enter 确认、Esc 取消，失焦时保存非空名称、取消空白新建行。
