import type { ReactNode } from "react";
import { ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuLabel, ContextMenuSeparator, ContextMenuTrigger } from "../ui/ContextMenu";

export type DeckMenuAction = "collapse" | "expand" | "rename" | "delete" | "create-folder" | "create-deck";

export default function DeckContextMenu({ children, name, isFolder, onAction }: {
  children: ReactNode;
  name: string;
  isFolder: boolean;
  onAction: (action: DeckMenuAction) => void;
}) {
  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>{children}</ContextMenuTrigger>
      <ContextMenuContent className="w-48" onClick={(event) => event.stopPropagation()} onDoubleClick={(event) => event.stopPropagation()}>
        <ContextMenuLabel className="max-w-44 truncate">{name}</ContextMenuLabel>
        <ContextMenuItem disabled={!isFolder} onSelect={() => onAction("collapse")}>折叠所有子目录项</ContextMenuItem>
        <ContextMenuItem disabled={!isFolder} onSelect={() => onAction("expand")}>展开所有目录项</ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem onSelect={() => onAction("create-folder")}>新增文件夹</ContextMenuItem>
        <ContextMenuItem onSelect={() => onAction("create-deck")}>新增 deck</ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem onSelect={() => onAction("rename")}>重命名</ContextMenuItem>
        <ContextMenuItem variant="danger" onSelect={() => onAction("delete")}>删除</ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}
