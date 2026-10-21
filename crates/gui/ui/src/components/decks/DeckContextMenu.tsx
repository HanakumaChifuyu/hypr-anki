import { useRef, type ReactNode } from "react";
import { ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuLabel, ContextMenuSeparator, ContextMenuTrigger } from "../ui/ContextMenu";

export type DeckMenuCommand = "collapse" | "expand" | "rename" | "delete" | "create-folder" | "create-deck";

export default function DeckContextMenu({ children, name, isFolder, onAction }: {
  children: ReactNode;
  name: string;
  isFolder: boolean;
  onAction: (action: DeckMenuCommand) => void;
}) {
  const inlineAction = useRef(false);
  function select(action: DeckMenuCommand) {
    inlineAction.current = action === "rename" || action === "create-folder" || action === "create-deck";
    onAction(action);
  }
  return (
    <ContextMenu onOpenChange={(open) => { if (open) inlineAction.current = false; }}>
      <ContextMenuTrigger asChild>{children}</ContextMenuTrigger>
      <ContextMenuContent className="w-48" onCloseAutoFocus={(event) => { if (inlineAction.current) event.preventDefault(); }} onClick={(event) => event.stopPropagation()} onDoubleClick={(event) => event.stopPropagation()}>
        <ContextMenuLabel className="max-w-44 truncate">{name}</ContextMenuLabel>
        <ContextMenuItem disabled={!isFolder} onSelect={() => select("collapse")}>折叠所有子目录项</ContextMenuItem>
        <ContextMenuItem disabled={!isFolder} onSelect={() => select("expand")}>展开所有目录项</ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem onSelect={() => select("create-folder")}>新增文件夹</ContextMenuItem>
        <ContextMenuItem onSelect={() => select("create-deck")}>新增 deck</ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem onSelect={() => select("rename")}>重命名</ContextMenuItem>
        <ContextMenuItem variant="danger" onSelect={() => select("delete")}>删除</ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}
