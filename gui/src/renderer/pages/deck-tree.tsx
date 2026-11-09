import React, {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import { Icon } from '../component/Icon';
import {
  Panel,
  Group,
  Separator,
  usePanelRef,
  PanelImperativeHandle,
} from 'react-resizable-panels';
import { NodeRendererProps, Tree } from 'react-arborist';

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
  const deckNodeTree = useMemo(() => createDebugDeckTree(), []);

  return (
    <Panel
      id="deck-tree"
      defaultSize={260}
      groupResizeBehavior="preserve-pixel-size"
      panelRef={treeRef}
      collapsible
      onResize={({ inPixels }) => onCollapsedChange(inPixels === 0)}
    >
      <div className="flex h-full min-w-0 flex-col bg-white/[0.015]">
        <div className="flex h-11 shrink-0 items-center gap-2 border-b border-white/5 px-4">
          <Icon className="size-4 text-zinc-500">
            <rect x="4" y="7" width="16" height="14" rx="2" />
            <path d="M7 4h10M10 1h4" />
          </Icon>
          <span className="text-xs font-medium tracking-wide text-zinc-300">
            牌组
          </span>
          <span className="ml-auto rounded bg-white/5 px-1.5 py-0.5 text-[10px] tabular-nums text-zinc-500">
            {deckNodeTree.root.length}
          </span>
        </div>
        <div className="min-h-0 flex-1 pt-2 pl-1.5">
          <DeckTreeView deckNodeTree={deckNodeTree} />
        </div>
      </div>
    </Panel>
  );
}

function DeckTreeView({ deckNodeTree }: { deckNodeTree: DeckNodeTree }) {
  const divRef = useRef<HTMLDivElement>(null);
  const [height, setHeight] = useState(0);
  useEffect(() => {
    const divref = divRef.current;
    if (!divref) return;
    const observer = new ResizeObserver(([entry]) => {
      setHeight(entry.contentRect.height);
    });
    observer.observe(divref);
    return () => observer.disconnect();
  }, []);
  return (
    <div ref={divRef} className="h-full min-h-0">
      <Tree<DeckNode>
        aria-label="牌组列表"
        className="[scrollbar-width:auto] [scrollbar-color:auto]
          [&::-webkit-scrollbar]:size-2
          [&::-webkit-scrollbar-track]:bg-transparent
          [&::-webkit-scrollbar-thumb]:rounded-full
          [&::-webkit-scrollbar-thumb]:border-2
          [&::-webkit-scrollbar-thumb]:border-solid
          [&::-webkit-scrollbar-thumb]:border-transparent
          [&::-webkit-scrollbar-thumb]:bg-white/20
          [&::-webkit-scrollbar-thumb]:bg-clip-padding
          [&::-webkit-scrollbar-thumb:hover]:bg-white/35
          [&::-webkit-scrollbar-thumb:active]:bg-white/50
          [&::-webkit-scrollbar-corner]:bg-transparent
          [&::-webkit-scrollbar-button]:hidden"
        data={deckNodeTree.root}
        idAccessor={(deck) => String(deck.data.id)}
        childrenAccessor={(deck) => (deck.folder ? deck.children : null)}
        height={height}
        width="100%"
        rowHeight={32}
        indent={8}
        rowClassName="rounded-md focus-visible:outline-none
        focus-visible:ring-1 focus-visible:ring-inset focus-visible:ring-white/20"
        openByDefault
        disableDrag
        disableDrop
        disableEdit
      >
        {DeckNodeRaw}
      </Tree>
    </div>
  );
}

function DeckNodeRaw({ node, style }: NodeRendererProps<DeckNode>) {
  const isFolder = node.data.folder;

  return (
    <div
      style={style}
      className={`group pl-0.5 relative flex h-[30px] min-w-0 items-center gap-1 rounded-md pr-2
        font-app text-[13px] leading-none transition-colors motion-reduce:transition-none
        ${node.isSelected ? 'bg-sky-400/10 text-sky-100 ring-1 ring-inset ring-sky-300/15' : 'text-zinc-400 hover:bg-white/5 hover:text-zinc-200'}`}
    >
      {isFolder ? (
        <button
          type="button"
          className="flex size-5 shrink-0 items-center justify-center rounded text-zinc-500 outline-none focus-visible:outline-2 focus-visible:outline-sky-300"
          aria-label={node.isOpen ? '收起子牌组' : '展开子牌组'}
          aria-expanded={node.isOpen}
          onClick={(event) => {
            event.stopPropagation();
            node.toggle();
          }}
        >
          <Icon className="size-3.5">
            {node.isOpen ? (
              <>
                <path d="M3 10V5a2 2 0 0 1 2-2h5l2 3h7a2 2 0 0 1 2 2v2" />
                <path d="M3 10h18l-2 9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2l-1-7a2 2 0 0 1 1-2Z" />
              </>
            ) : (
              <>
                <path d="M3 7V5a2 2 0 0 1 2-2h5l2 3h7a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z" />
                <path d="M3 9h18" />
              </>
            )}
          </Icon>
        </button>
      ) : (
        <span className="flex size-5 shrink-0 items-center justify-center">
          <Icon
            className={`size-3.5 ${node.isSelected ? 'text-sky-300/80' : 'text-zinc-500'}`}
          >
            <rect x="5" y="5" width="15" height="16" rx="2" />
            <path d="M16 2H4a2 2 0 0 0-2 2v12" />
          </Icon>
        </span>
      )}
      <span
        className={`min-w-0 flex-1 truncate pl-1 ${node.level === 0 ? 'font-medium' : 'font-normal'}`}
        title={node.data.data.name}
      >
        {node.data.data.name}
      </span>
    </div>
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
    public folder: boolean,
  ) {}
}

type DeckNode =
  | {
      readonly folder: true;
      readonly data: Readonly<Omit<RawDeck, 'folder'>>;
      readonly children: DeckNode[];
    }
  | {
      readonly folder: false;
      readonly data: Readonly<Omit<RawDeck, 'folder'>>;
      readonly children?: never;
    };

class DeckNodeTree {
  constructor(public root: DeckNode[]) {}

  public static buildTree(decks: readonly RawDeck[]): DeckNodeTree {
    const roots: DeckNode[] = [];
    const nodes = new Map<number, DeckNode>();

    // 先创建所有节点，不要求父节点排在子节点之前。
    for (const { folder, ...data } of decks) {
      nodes.set(
        data.id,
        folder ? { folder: true, data, children: [] } : { folder: false, data },
      );
    }

    // 数据合法性由后端保证，这里只组装父子关系。
    for (const node of nodes.values()) {
      const parentId = node.data.parent;
      if (parentId === null) {
        roots.push(node);
        continue;
      }
      const parent = nodes.get(parentId);
      if (parent?.folder) parent.children.push(node);
    }

    return new DeckNodeTree(roots);
  }
}

// 调试数据：同时覆盖长列表、深层嵌套和超长名称。
function createDebugDeckTree(): DeckNodeTree {
  const decks: RawDeck[] = [];
  const subjects = ['英语', '日语', '计算机科学', '数学', '历史', '医学'];
  let nextId = 1;

  for (const subject of subjects) {
    const rootId = nextId++;
    decks.push(
      new RawDeck(rootId, `${subject} · 学习资料与长期复习计划`, null, true),
    );

    for (let chapter = 1; chapter <= 8; chapter++) {
      const chapterId = nextId++;
      decks.push(
        new RawDeck(
          chapterId,
          `第 ${chapter} 章 · ${subject}基础知识与重点难点整理`,
          rootId,
          true,
        ),
      );

      for (let section = 1; section <= 4; section++) {
        const sectionId = nextId++;
        decks.push(
          new RawDeck(
            sectionId,
            `${chapter}.${section} · 这是一条用于测试侧栏文字截断与悬浮提示的超长牌组名称 — Long deck name for overflow debugging`,
            chapterId,
            chapter === 1 && section === 1,
          ),
        );

        // 第一条分支延伸至 10 层，方便观察连续缩进。
        if (chapter === 1 && section === 1) {
          let parentId = sectionId;
          for (let depth = 4; depth <= 10; depth++) {
            const id = nextId++;
            decks.push(
              new RawDeck(
                id,
                `第 ${depth} 层 · ${subject}嵌套牌组 / Nested deck level ${depth}`,
                parentId,
                depth < 10,
              ),
            );
            parentId = id;
          }
        }
      }
    }

    decks.push(new RawDeck(nextId++, `${subject} · 空文件夹`, rootId, true));
    decks.push(
      new RawDeck(nextId++, `${subject} · 综合复习牌组`, rootId, false),
    );
  }

  return DeckNodeTree.buildTree(decks);
}

if (import.meta.vitest) {
  const { test, expect } = import.meta.vitest;

  test('builds unordered records and preserves sibling order', () => {
    const records = [
      new RawDeck(3, 'First deck', 1, false),
      new RawDeck(2, 'Empty folder', 1, true),
      new RawDeck(1, 'Root', null, true),
      new RawDeck(4, 'Root deck', null, false),
    ];
    const tree = DeckNodeTree.buildTree(records);
    expect(tree.root.map((node) => node.data.id)).toEqual([1, 4]);
    const root = tree.root[0];
    if (!root.folder) throw new Error('Expected folder');
    expect(root.children.map((node) => node.data.id)).toEqual([3, 2]);
    expect(root.children[0]).not.toHaveProperty('children');
    expect(root.children[1]).toMatchObject({ folder: true, children: [] });
    expect(root.data).not.toHaveProperty('folder');
    expect(records[0].folder).toBe(false);
  });

  test('accepts empty input and the debug fixture', () => {
    expect(DeckNodeTree.buildTree([]).root).toEqual([]);
    expect(createDebugDeckTree().root).toHaveLength(6);
  });
}
