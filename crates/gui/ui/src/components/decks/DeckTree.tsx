import { createContext, useContext, useEffect, useMemo, useRef, useState, type CSSProperties } from "react";
import { useDrop } from "react-dnd";
import { Tree, type DragPreviewProps, type NodeRendererProps, type TreeApi } from "react-arborist";
import { buildDeckTree, type DeckMove, type DeckNode } from "../../features/decks/deck-tree";
import type { Deck } from "../../types/deck";

export interface DeckTreeProps {
  decks: readonly Deck[];
  onMove?: (move: DeckMove) => void;
  onCollapsedChange?: (id: number, collapsed: boolean) => void;
  onActivate?: (deck: Deck) => void;
  onSettings?: (deck: Deck) => void;
}

const DECK_DRAG_TYPE = "HYPRANKI_DECK";
const DeckMoveContext = createContext<DeckTreeProps["onMove"]>(undefined);

const DeckSettingsContext = createContext<((deck: Deck) => void) | undefined>(undefined);
const DeckPreviewContext = createContext<readonly Deck[]>([]);

function DeckDragPreview({ id, offset, isDragging }: DragPreviewProps) {
  const decks = useContext(DeckPreviewContext);
  const deck = decks.find((deck) => String(deck.id) === id);
  if (!isDragging || !offset || !deck) return null;
  const hasChildren = decks.some((child) => child.parent_id === deck.id);

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
  const onMove = useContext(DeckMoveContext);
  const [{ isOver, canDrop }, drop] = useDrop<{ id: string }, unknown, { isOver: boolean; canDrop: boolean }>(() => ({
    accept: DECK_DRAG_TYPE,
    canDrop: ({ id }) => {
      const source = tree.get(id);
      return !preview && !!onMove && !!source && source.id !== node.id
        && (source.children?.length ?? 0) === 0 && (node.children?.length ?? 0) > 0
        && !source.isAncestorOf(node) && source.parent?.id !== node.id;
    },
    drop: ({ id }, monitor) => {
      if (monitor.didDrop()) return;
      const source = tree.get(id);
      if (!source || !onMove || preview || (source.children?.length ?? 0) > 0
        || (node.children?.length ?? 0) === 0 || source.id === node.id
        || source.parent?.id === node.id) return;
      onMove({ deck_id: source.data.deck.id, parent_id: node.data.deck.id, index: node.children?.length ?? 0 });
      node.open();
      return {};
    },
    collect: (monitor) => ({ isOver: monitor.isOver({ shallow: true }), canDrop: monitor.canDrop() }),
  }), [node, tree, onMove, preview]);
  const receiving = isOver && canDrop;
  const onSettings = useContext(DeckSettingsContext);
  const hasChildren = node.children !== null && node.children.length > 0;
  return (
    <div
      ref={(element) => { drop(element); }}
      style={style}
      title={node.data.deck.name}
      className={`relative flex h-full min-w-0 items-center gap-1 rounded-md pr-1 text-[13px] select-none ${receiving ? "bg-accent/10"
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
        {hasChildren ? (
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
      <div className="grid shrink-0 grid-cols-[40px_24px] items-center gap-x-1 text-center text-[10px] font-medium tabular-nums">
        <div className="flex flex-col items-center leading-3">
        <span aria-label={`今日新学 ${node.data.deck.new_count} 张`} className={node.data.deck.new_count === 0 ? "text-muted/50" : "text-new"}>{node.data.deck.new_count}</span>
        <span aria-label={`待复习 ${node.data.deck.review_count} 张`} className={node.data.deck.review_count === 0 ? "text-muted/50" : "text-review"}>{node.data.deck.review_count}</span>
        </div>
        <button
          type="button"
          aria-label={`设置 ${node.data.deck.name}`}
          title={node.data.deck.kind === "group" ? "分组设置" : "卡组设置"}
          disabled={!onSettings}
          className="flex size-6 cursor-pointer items-center justify-center rounded text-muted/40 transition-colors hover:bg-hover hover:text-foreground focus-visible:text-foreground focus-visible:outline-2 focus-visible:outline-focus disabled:cursor-default disabled:opacity-40"
          onClick={(event) => { event.stopPropagation(); onSettings?.(node.data.deck); }}
          onDoubleClick={(event) => event.stopPropagation()}
        >
          <svg aria-hidden="true" viewBox="0 0 24 24" className="size-4" fill="currentColor">
            <circle cx="5" cy="12" r="1.8" />
            <circle cx="12" cy="12" r="1.8" />
            <circle cx="19" cy="12" r="1.8" />
          </svg>
        </button>
      </div>
      {receiving && (
        <div
          aria-hidden="true"
          className="pointer-events-none absolute right-1 bottom-0 h-0.5 rounded-full bg-accent"
          style={{ left: (node.level + 1) * tree.indent }}
        >
        </div>
      )}
    </div>
  );
}

export default function DeckTree({ decks, onMove, onCollapsedChange, onActivate, onSettings }: DeckTreeProps) {
  const tree = useRef<TreeApi<DeckNode> | null>(null);
  const container = useRef<HTMLDivElement>(null);
  const syncing = useRef(false);
  const [height, setHeight] = useState(420);
  const { nodes, error } = useMemo<{ nodes: DeckNode[]; error: String | null }>(() => {
    try { return { nodes: buildDeckTree(decks), error: null }; }
    catch (error) { return { nodes: [], error: error instanceof Error ? error.message : "卡组数据无效。" }; }
  }, [decks]);
  const visibleCount = useMemo(() => {
    let count = 0;
    const pending = [...nodes];
    while (pending.length) {
      const node = pending.pop()!;
      count++;
      if (!node.deck.is_collapsed) pending.push(...node.children);
    }
    return count;
  }, [nodes]);

  return (
    <DeckPreviewContext.Provider value={decks}>
    <DeckMoveContext.Provider value={onMove}>
    <DeckSettingsContext.Provider value={onSettings}>
      <div
        ref={container}
        className="h-[min(var(--tree-content-height),max(170px,calc(100dvh-220px)))] w-full"
        style={{ "--tree-content-height": `${Math.max(100, (visibleCount + 1) * 34)}px` } as CSSProperties}
      >
        {error ? <p role="alert" className="py-8 text-sm text-muted">{error}</p>
          : decks.length === 0 ? <p className="py-8 text-center text-sm text-muted">还没有卡组。</p>
            : <Tree<DeckNode>
              ref={tree}
              data={nodes}
              width="100%"
              height={height}
              rowHeight={34}
              indent={14}
              openByDefault
              disableEdit
              disableMultiSelection
              disableDrag={(node) => !onMove || (node.children?.length ?? 0) > 0}
              // Dedicated row targets accept only reparenting, with no insertion slots.
              dragType={DECK_DRAG_TYPE}
              disableDrop
              onToggle={(id) => {
                let node = tree.current!.get(id);
                if (!node) return;
                onCollapsedChange?.(node.data.deck.id, node.isClosed);
              }}
              onActivate={(node) => onActivate?.(node.data.deck)}
              renderCursor={() => null}
              renderDragPreview={DeckDragPreview}
              aria-label="卡组树"
              className="rounded-md focus-visible:outline-2 focus-visible:outline-focus"
            >
              {DeckRow}
            </Tree>}
      </div>
    </DeckSettingsContext.Provider>
    </DeckMoveContext.Provider>
    </DeckPreviewContext.Provider>
  );
}
