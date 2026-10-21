import { createContext, useContext, useEffect, useMemo, useRef, useState } from "react";
import { Tree, type CursorProps, type DragPreviewProps, type NodeRendererProps, type TreeApi } from "react-arborist";
import { buildDeckTree, type DeckMove, type DeckNode } from "../../features/decks/deck-tree";
import { insertDeck } from "../../features/decks/deck-actions";
import type { Deck } from "../../types/deck";
import DeckContextMenu, { type DeckMenuCommand } from "./DeckContextMenu";

/** Rename/create are emitted after editing is confirmed; delete requests confirmation. */
export type DeckMenuAction =
  | { type: "rename"; id: number; name: string }
  /** The caller assigns the confirmed draft's persistent ID and timestamps. */
  | { type: "create"; deck: Deck; afterId: number }
  | { type: "delete"; deck: Deck };

export interface DeckTreeProps {
  decks: readonly Deck[];
  onMove?: (move: DeckMove) => void;
  onCollapsedChange?: (id: number, collapsed: boolean) => void;
  onActivate?: (deck: Deck) => void;
  onAction?: (action: DeckMenuAction) => void;
}

type DeckTreeNode = NodeRendererProps<DeckNode>["node"];
const DeckMenuContext = createContext<((command: DeckMenuCommand, node: DeckTreeNode) => void) | undefined>(undefined);

const DeckPreviewContext = createContext<readonly Deck[]>([]);

function DeckDragPreview({ id, offset, isDragging }: DragPreviewProps) {
  const decks = useContext(DeckPreviewContext);
  const deck = decks.find((deck) => String(deck.id) === id);
  if (!isDragging || !offset || !deck) return null;
  const hasChildren = deck.kind === "group";

  return (
    <div className="pointer-events-none fixed top-0 left-0 z-50 w-[min(300px,calc(100vw-40px))]" style={{ transform: `translate(${offset.x}px, ${offset.y}px)` }}>
      <div className="flex h-[34px] origin-left items-center gap-2 rounded-lg border border-border bg-surface px-3 text-[13px] text-foreground shadow-[0_12px_28px_-8px_#0006,0_3px_8px_#0002] motion-safe:animate-deck-lift">
        <svg aria-hidden="true" viewBox="0 0 20 20" className={`size-3.5 shrink-0 ${hasChildren ? "text-accent" : "text-deck-icon"}`} fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinejoin="round">
          {hasChildren ? <path d="M2.5 6V4.5A1.5 1.5 0 0 1 4 3h4l2 2h6A1.5 1.5 0 0 1 17.5 6.5V15a1.5 1.5 0 0 1-1.5 1.5H4A1.5 1.5 0 0 1 2.5 15Z" /> : <><rect x="6" y="3" width="10" height="12" rx="1.5" /><path d="M12 17H5a1 1 0 0 1-1-1V7" /></>}
        </svg>
        <span className="min-w-0 flex-1 truncate font-medium">{deck.name}</span>
        <span className="flex flex-col text-center text-[10px] leading-3 tabular-nums"><span className="text-new">{deck.new_count}</span><span className="text-review">{deck.review_count}</span></span>
      </div>
    </div>
  );
}

function DeckNameEditor({ node }: { node: NodeRendererProps<DeckNode>["node"] }) {
  const input = useRef<HTMLInputElement>(null);
  const finished = useRef(false);
  const [name, setName] = useState(node.data.deck.name);
  const [error, setError] = useState(false);
  useEffect(() => { input.current?.focus(); input.current?.select(); }, []);

  function confirm() {
    if (finished.current) return;
    const value = name.trim();
    if (!value) { setError(true); input.current?.focus(); return; }
    finished.current = true;
    node.submit(value);
  }

  function cancel() {
    if (finished.current) return;
    finished.current = true;
    node.reset();
  }

  return <input
    ref={input}
    aria-label="名称"
    aria-invalid={error}
    title={error ? "名称不能为空" : "Enter 确认，Esc 取消"}
    placeholder={node.data.deck.kind === "group" ? "文件夹名称" : "Deck 名称"}
    value={name}
    className={`h-6 min-w-0 flex-1 rounded border bg-background px-1.5 text-[13px] text-foreground outline-none ${error ? "border-danger" : "border-focus"}`}
    onChange={(event) => { setName(event.target.value); setError(false); }}
    onClick={(event) => event.stopPropagation()}
    onDoubleClick={(event) => event.stopPropagation()}
    onKeyDown={(event) => {
      event.stopPropagation();
      if (event.nativeEvent.isComposing) return;
      if (event.key === "Enter") { event.preventDefault(); confirm(); }
      if (event.key === "Escape") { event.preventDefault(); cancel(); }
    }}
    onBlur={() => { if (name.trim()) confirm(); else cancel(); }}
  />;
}

function DeckRow({ node, style, dragHandle }: NodeRendererProps<DeckNode>) {
  const onMenuCommand = useContext(DeckMenuContext);
  const receiving = node.willReceiveDrop;
  const hasChildren = node.children !== null && node.children.length > 0;
  return (
    <DeckContextMenu name={node.data.deck.name} isFolder={node.data.deck.kind === "group"} onAction={(command) => onMenuCommand?.(command, node)}>
    <div
      style={style}
      title={node.data.deck.name}
      onClick={(event) => {
        event.stopPropagation();
        node.select();
      }}
      onDoubleClick={(event) => {
        event.stopPropagation();
        node.activate();
      }}
      className={`relative flex h-full min-w-0 items-center gap-1 rounded-md pr-1 text-[13px] select-none ${receiving ? "bg-accent/10 ring-1 ring-inset ring-accent/60"
        : node.isSelected ? "bg-accent/10" : "hover:bg-hover"
        } transition-opacity duration-150 ${node.isDragging ? "opacity-20" : ""} ${node.isFocused ? "outline-1 -outline-offset-1 outline-focus" : ""}`}
    >
      <div ref={node.isEditing ? undefined : dragHandle} className="flex min-w-0 flex-1 items-center gap-0.5">
        {hasChildren ? (
          <button
            type="button"
            aria-label={`${node.isOpen ? "收起" : "展开"} ${node.data.deck.name}`}
            aria-expanded={node.isOpen}
            className="flex h-6 w-4 shrink-0 cursor-pointer items-center justify-center rounded text-muted hover:text-foreground focus-visible:outline-2 focus-visible:outline-focus"
            onClick={(event) => { event.stopPropagation(); node.toggle(); }}
            onDoubleClick={(event) => event.stopPropagation()}
          >
            <svg aria-hidden="true" viewBox="0 0 16 16" className={`size-3 transition-transform motion-reduce:transition-none ${node.isOpen ? "rotate-90" : ""}`} fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="m6 4 4 4-4 4" />
            </svg>
          </button>
        ) : <span className="w-4 shrink-0" />}
        {node.data.deck.kind === "group" ? (
          <svg role="img" aria-label="卡组分组" viewBox="0 0 20 20" className="mr-0.5 size-3.5 shrink-0 text-accent" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinejoin="round">
            <path d="M2.5 6V4.5A1.5 1.5 0 0 1 4 3h4l2 2h6A1.5 1.5 0 0 1 17.5 6.5V15a1.5 1.5 0 0 1-1.5 1.5H4A1.5 1.5 0 0 1 2.5 15Z" />
            <path d="M2.5 7h15" />
          </svg>
        ) : (
          <svg role="img" aria-label="卡组" viewBox="0 0 20 20" className="mr-0.5 size-3.5 shrink-0 text-deck-icon" fill="none" stroke="currentColor" strokeWidth="1.3">
            <rect x="6" y="3" width="10" height="12" rx="1.5" />
            <path d="M12 17H5a1 1 0 0 1-1-1V7" />
          </svg>
        )}
        {node.isEditing ? <DeckNameEditor node={node} /> : <span className={`truncate ${node.level === 0 ? "font-medium text-foreground" : "text-secondary"}`}>{node.data.deck.name}</span>}
      </div>
      <div className="grid shrink-0 grid-cols-[40px] items-center gap-x-1 text-center text-[10px] font-medium tabular-nums">
        <div className="flex flex-col items-center leading-3">
        <span aria-label={`今日新学 ${node.data.deck.new_count} 张`} className={node.data.deck.new_count === 0 ? "text-muted/50" : "text-new"}>{node.data.deck.new_count}</span>
        <span aria-label={`待复习 ${node.data.deck.review_count} 张`} className={node.data.deck.review_count === 0 ? "text-muted/50" : "text-review"}>{node.data.deck.review_count}</span>
        </div>
      </div>
    </div>
    </DeckContextMenu>
  );
}

function DeckInsertCursor({ top, left }: CursorProps) {
  return <div aria-hidden="true" className="pointer-events-none absolute right-1 z-10 h-0.5 rounded-full bg-accent" style={{ top, left }} />;
}

export default function DeckTree({ decks, onMove, onCollapsedChange, onActivate, onAction }: DeckTreeProps) {
  const tree = useRef<TreeApi<DeckNode> | null>(null);
  const container = useRef<HTMLDivElement>(null);
  const [initialOpenState] = useState(() => Object.fromEntries(
    decks.map((deck) => [String(deck.id), !deck.is_collapsed]),
  ));
  const [height, setHeight] = useState(1);
  const [draft, setDraft] = useState<{ deck: Deck; afterId: number } | null>(null);
  const { nodes, error } = useMemo<{ nodes: DeckNode[]; error: String | null }>(() => {
    try { return { nodes: buildDeckTree(draft ? insertDeck(decks, draft.deck, draft.afterId) : decks), error: null }; }
    catch (error) { return { nodes: [], error: error instanceof Error ? error.message : "卡组数据无效。" }; }
  }, [decks, draft]);

  // A draft is only a view row. Arborist owns its edit session and cancellation.
  useEffect(() => {
    if (!draft || !tree.current) return;
    void tree.current.edit(String(draft.deck.id)).then(() => setDraft((current) => current === draft ? null : current));
  }, [draft]);

  function handleMenuCommand(command: DeckMenuCommand, node: DeckTreeNode) {
    const deck = node.data.deck;
    if (command === "collapse" || command === "expand") {
      const pending = [node];
      while (pending.length) {
        const current = pending.pop()!;
        if (current.data.deck.kind === "group") {
          if (command === "collapse") current.close();
          else current.open();
        }
        pending.push(...current.children ?? []);
      }
    } else if (command === "rename") {
      if (onAction) void node.edit();
    } else if (command === "create-folder" || command === "create-deck") {
      if (!onAction) return;
      const parent = deck.kind === "group" ? node : node.parent;
      if (parent && !parent.isRoot) parent.open();
      setDraft({
        afterId: deck.id,
        deck: {
          id: Math.min(0, ...decks.map((item) => item.id)) - 1,
          name: "", kind: command === "create-folder" ? "group" : "deck",
          parent_id: deck.kind === "group" ? deck.id : deck.parent_id,
          is_collapsed: false, new_count: 0, review_count: 0,
          config_id: deck.config_id, created_at: 0, updated_at: 0,
        },
      });
    } else onAction?.({ type: "delete", deck });
  }

  function handleNameSubmit(id: string, name: string) {
    if (draft && id === String(draft.deck.id)) {
      onAction?.({ type: "create", deck: { ...draft.deck, name }, afterId: draft.afterId });
    } else {
      onAction?.({ type: "rename", id: Number(id), name });
    }
  }
  useEffect(() => {
    const element = container.current;
    if (!element) return;
    const observer = new ResizeObserver(([entry]) => {
      setHeight(Math.max(1, Math.floor(entry.contentRect.height)));
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  return (
    <DeckMenuContext.Provider value={handleMenuCommand}>
    <DeckPreviewContext.Provider value={decks}>
      <div
        ref={container}
        className="min-h-0 w-full flex-1 overflow-hidden [scrollbar-color:var(--app-border)_transparent]"
        onDragStartCapture={() => {
          const activeElement = document.activeElement;
          if (activeElement instanceof HTMLElement && container.current?.contains(activeElement)) {
            activeElement.blur();
          }
          tree.current?.onBlur();
        }}
        onKeyDownCapture={(event) => {
          // Arborist moves DOM focus from the tree container to the focused row.
          // Inputs, row buttons and portaled menus keep their own keyboard behavior.
          if (event.key !== "Enter" || event.nativeEvent.isComposing
            || !(event.target instanceof HTMLElement)
            || !event.target.matches('[role="tree"], [role="treeitem"]')
            || tree.current?.isEditing) return;
          event.preventDefault();
          event.stopPropagation();
          const node = tree.current?.focusedNode;
          if (!node) return;
          if (node.data.deck.kind === "group") node.toggle();
          else node.activate();
        }}
      >
        {error ? <p role="alert" className="py-8 text-sm text-muted">{error}</p>
          : decks.length === 0 ? <p className="py-8 text-center text-sm text-muted">还没有卡组。</p>
            : <Tree<DeckNode>
              ref={tree}
              data={nodes}
              childrenAccessor={(node) => node.deck.kind === "group" ? node.children : null}
              width="100%"
              height={height}
              rowHeight={34}
              indent={14}
              openByDefault
              initialOpenState={initialOpenState}
              disableEdit={!onAction}
              onRename={({ id, name }) => handleNameSubmit(id, name)}
              disableMultiSelection
              disableDrag={(node) => !onMove || tree.current?.editingId === node.id || node.deck.id === draft?.deck.id}
              disableDrop={({ parentNode, dragNodes }) => !onMove
                || (!parentNode.isRoot && parentNode.data.deck.kind !== "group")
                || dragNodes.some((source) => source.id === parentNode.id || source.isAncestorOf(parentNode))}
              onMove={({ dragNodes, parentId, parentNode, index }) => {
                const source = dragNodes[0];
                if (!source || !onMove) return;
                const siblings = parentNode?.children ?? tree.current?.root.children ?? [];
                const oldIndex = siblings.findIndex((node) => node.id === source.id);
                // Arborist reports insertion slots before removing the source.
                const destination = index - (oldIndex >= 0 && oldIndex < index ? 1 : 0);
                onMove({ deck_id: source.data.deck.id, parent_id: parentId === null ? null : Number(parentId), index: destination });
              }}
              onToggle={(id) => {
                // get() only finds visible nodes; recursive toggles also touch hidden descendants.
                const deck = decks.find((deck) => String(deck.id) === id);
                if (!deck || !tree.current) return;
                onCollapsedChange?.(deck.id, !tree.current.isOpen(id));
              }}
              onActivate={(node) => onActivate?.(node.data.deck)}
              renderCursor={DeckInsertCursor}
              renderDragPreview={DeckDragPreview}
              aria-label="卡组树"
              className="rounded-md focus-visible:outline-2 focus-visible:outline-focus"
            >
              {DeckRow}
            </Tree>}
      </div>
    </DeckPreviewContext.Provider>
    </DeckMenuContext.Provider>
  );
}
