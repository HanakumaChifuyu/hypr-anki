import React, { createContext, useContext, useMemo, useState } from 'react';
import { Icon } from '../component/Icon';
import {
  Panel,
  Group,
  Separator,
  usePanelRef,
  PanelImperativeHandle,
} from 'react-resizable-panels';

interface DeckContextType {
  toggleTree: () => void;
  isTreeCollapsed: boolean;
}

const DeckContext = createContext<DeckContextType | null>(null);

export function Deck() {
  const treeRef = usePanelRef();
  const [isTreeCollapsed, setIsTreeCollapsed] = useState(false);

  const deckContextValue: DeckContextType = {
    isTreeCollapsed,
    toggleTree: () => {
      if (treeRef.current?.isCollapsed()) {
        treeRef.current.expand();
      } else {
        treeRef.current?.collapse();
      }
    },
  };
  return (
    <DeckContext.Provider value={deckContextValue}>
      <Group className="h-full">
        <DeckTree treeRef={treeRef} onCollapsedChange={setIsTreeCollapsed} />
        <Separator
          className={` ${!isTreeCollapsed ? 'border-r' : 'border-0'}  border-white/8`}
        />
        <DeckContent />
      </Group>
    </DeckContext.Provider>
  );
}
function DeckTree({
  treeRef,
  onCollapsedChange,
}: {
  treeRef: React.RefObject<PanelImperativeHandle | null>;
  onCollapsedChange: (collapsed: boolean) => void;
}) {
  return (
    <Panel
      className="w-20 shrink-0"
      id="deck-tree"
      panelRef={treeRef}
      collapsible
      onResize={({ inPixels }) => onCollapsedChange(inPixels === 0)}
    />
  );
}

function DeckContent() {
  const deckTreeContext = useContext(DeckContext);
  const isCollapsed = deckTreeContext?.isTreeCollapsed ?? false;
  const label = isCollapsed ? '展开牌组侧栏' : '收起牌组侧栏';

  return (
    <Panel>
      <div className="flex h-11 items-center px-2">
        <button
          type="button"
          onClick={deckTreeContext?.toggleTree}
          aria-label={label}
          aria-expanded={!isCollapsed}
          aria-controls="deck-tree"
          title={label}
          className="inline-flex size-8 items-center justify-center rounded-md
            text-zinc-400 transition-colors hover:bg-white/8 hover:text-zinc-100
            active:bg-white/12 focus-visible:outline-2 focus-visible:outline-offset-2
            focus-visible:outline-zinc-400 motion-reduce:transition-none"
        >
          <Icon className="size-4.5">
            <rect x="3" y="4" width="18" height="16" rx="2.5" />
            <path d="M9 4v16" />
            <path d={isCollapsed ? 'm13 9 3 3-3 3' : 'm16 9-3 3 3 3'} />
          </Icon>
        </button>
      </div>
    </Panel>
  );
}

class RawDeck {
  constructor(
    public id: number,
    public name: string,
    public parent: number | null,
  ) {}
}

class DeckNode {
  constructor(
    public data: RawDeck,
    public children?: DeckNode[],
  ) {
    this.children = children ?? [];
  }
}

class DeckNodeTree {
  constructor(public root: DeckNode[]) {}

  public static buildTree = function (preOrderedDecks: RawDeck[]) {
    let adjacencyList: DeckNode[] = [];
    let nodeMaps = new Map<number, DeckNode>();
    for (const deck of preOrderedDecks) {
      const node = new DeckNode(deck);
      nodeMaps.set(deck.id, node);
      if (deck.parent == null) {
        adjacencyList.push(node);
      } else {
        nodeMaps.get(deck.parent)?.children?.push(node);
      }
    }
    return new DeckNodeTree(adjacencyList);
  };
}

if (import.meta.vitest) {
  const { test, expect } = import.meta.vitest;

  test('test build deck node tree', () => {});
}
