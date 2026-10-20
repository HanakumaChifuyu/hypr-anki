import { createContext, useContext, useEffect, useMemo, useRef, useState, type CSSProperties } from "react";
import { Tree, type CursorProps, type DragPreviewProps, type NodeRendererProps, type TreeApi } from "react-arborist";
import { buildDeckTree, type DeckMove, type DeckNode } from "../../features/decks/deck-tree";
import type { Deck } from "../../types/deck";
import DeckContextMenu, { type DeckMenuAction } from "./DeckContextMenu";

export interface DeckTreeProps {
  decks: readonly Deck[];
  onMove?: (move: DeckMove) => void;
  onCollapsedChange?: (id: number, collapsed: boolean) => void;
  onActivate?: (deck: Deck) => void;
  onMenuAction?: (action: DeckMenuAction, deck: Deck) => void;
}

const DeckMenuContext = createContext<DeckTreeProps["onMenuAction"]>(undefined);

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

function DeckRow({ node, tree, style, dragHandle, preview }: NodeRendererProps<DeckNode>) {
  const onMenuAction = useContext(DeckMenuContext);
  const receiving = node.willReceiveDrop;
  const hasChildren = node.children !== null && node.children.length > 0;
  return (
    <DeckContextMenu name={node.data.deck.name} isFolder={node.data.deck.kind === "group"} onAction={(action) => {
      if (action === "collapse" || action === "expand") {
        const pending = [node];
        while (pending.length) {
          const current = pending.pop()!;
          if (current.data.deck.kind === "group") {
            if (action === "collapse") current.close();
            else current.open();
          }
          pending.push(...current.children ?? []);
        }
      } else onMenuAction?.(action, node.data.deck);
    }}>
    <div
      style={style}
      title={node.data.deck.name}
      className={`relative flex h-full min-w-0 items-center gap-1 rounded-md pr-1 text-[13px] select-none ${receiving ? "bg-accent/10 ring-1 ring-inset ring-accent/60"
        : node.isSelected ? "bg-accent/10" : "hover:bg-hover"
        } transition-opacity duration-150 ${node.isDragging ? "opacity-20" : ""} ${node.isFocused ? "outline-1 -outline-offset-1 outline-focus" : ""}`}
    >
      <div ref={dragHandle} className="flex min-w-0 flex-1 items-center gap-0.5">
        {hasChildren ? (
          <button
            type="button"
            aria-label={`${node.isOpen ? "收起" : "展开"} ${node.data.deck.name}`}
            aria-expanded={node.isOpen}
            className="flex h-6 w-4 shrink-0 cursor-pointer items-center justify-center rounded text-muted hover:text-foreground focus-visible:outline-2 focus-visible:outline-focus"
            onClick={(event) => { event.stopPropagation(); node.toggle(); }}
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
        <span className={`truncate ${node.level === 0 ? "font-medium text-foreground" : "text-secondary"}`}>{node.data.deck.name}</span>
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

export default function DeckTree({ decks, onMove, onCollapsedChange, onActivate, onMenuAction }: DeckTreeProps) {
  const tree = useRef<TreeApi<DeckNode> | null>(null);
  const container = useRef<HTMLDivElement>(null);
  const syncing = useRef(false);
  const [height, setHeight] = useState(1);
  const { nodes, error } = useMemo<{ nodes: DeckNode[]; error: String | null }>(() => {
    try { return { nodes: buildDeckTree(decks), error: null }; }
    catch (error) { return { nodes: [], error: error instanceof Error ? error.message : "卡组数据无效。" }; }
  }, [decks]);
  useEffect(() => {
    const element = container.current;
    if (!element) return;
    const observer = new ResizeObserver(([entry]) => {
      setHeight(Math.max(1, Math.floor(entry.contentRect.height)));
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const api = tree.current;
    if (!api) return;
    syncing.current = true;
    try {
      for (const deck of decks) {
        if (deck.is_collapsed && api.isOpen(String(deck.id))) api.close(String(deck.id));
        else if (!deck.is_collapsed && !api.isOpen(String(deck.id))) api.open(String(deck.id));
      }
    } finally { syncing.current = false; }
  }, [decks]);

  return (
    <DeckMenuContext.Provider value={onMenuAction}>
    <DeckPreviewContext.Provider value={decks}>
      <div
        ref={container}
        className="min-h-0 w-full flex-1 overflow-hidden [scrollbar-color:var(--app-border)_transparent]"
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
              disableEdit
              disableMultiSelection
              disableDrag={!onMove}
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
                if (syncing.current) return;
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
