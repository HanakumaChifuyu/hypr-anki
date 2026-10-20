import type { Deck } from "../../types/deck";

export interface DeckNode {
  id: string;
  deck: Deck;
  children: DeckNode[];
}

export interface DeckMove {
  deck_id: number;
  parent_id: number | null;
  /** Destination sibling index AFTER removing the moving deck. */
  index: number;
}

/** One pass, O(n). Parents must precede children; sibling order is preserved. */
export function buildDeckTree(decks: readonly Deck[]): DeckNode[] {
  const roots: DeckNode[] = [];
  const nodes = new Map<number, DeckNode>();
  for (const deck of decks) {
    if (!Number.isSafeInteger(deck.id)) throw new Error("卡组 ID 超出前端安全整数范围。");
    if (nodes.has(deck.id)) throw new Error(`卡组 ID 重复：${deck.id}`);
    const node: DeckNode = { id: String(deck.id), deck, children: [] };
    if (deck.parent_id === null) roots.push(node);
    else {
      const parent = nodes.get(deck.parent_id);
      if (!parent) throw new Error(`卡组“${deck.name}”的父节点缺失或未排在子节点之前。`);
      parent.children.push(node);
    }
    nodes.set(deck.id, node);
  }
  return roots;
}

/** Move the whole subtree, then return a new preorder list without mutating input. */
export function moveDeck(decks: readonly Deck[], move: DeckMove): Deck[] {
  const roots = buildDeckTree(decks);
  const nodes = new Map<number, DeckNode>();
  const stack = [...roots];
  while (stack.length) {
    const node = stack.pop()!;
    nodes.set(node.deck.id, node);
    stack.push(...node.children);
  }
  const source = nodes.get(move.deck_id);
  const parent = move.parent_id === null ? null : nodes.get(move.parent_id);
  if (!source || parent === undefined) throw new Error("找不到需要移动的卡组或目标父卡组。");
  for (let ancestor = parent; ancestor; ) {
    if (ancestor.id === source.id) throw new Error("不能将卡组移入自己或自己的子卡组。");
    ancestor = ancestor.deck.parent_id === null ? null : nodes.get(ancestor.deck.parent_id)!;
  }
  const oldSiblings = source.deck.parent_id === null
    ? roots : nodes.get(source.deck.parent_id)!.children;
  oldSiblings.splice(oldSiblings.indexOf(source), 1);
  const siblings = parent ? parent.children : roots;
  if (!Number.isInteger(move.index) || move.index < 0 || move.index > siblings.length) {
    throw new Error("目标排序位置无效。");
  }
  source.deck = { ...source.deck, parent_id: move.parent_id };
  if (parent) parent.deck = { ...parent.deck, is_collapsed: false };
  siblings.splice(move.index, 0, source);
  const result: Deck[] = [];
  const pending = [...roots].reverse();
  while (pending.length) {
    const node = pending.pop()!;
    result.push(node.deck);
    for (let i = node.children.length - 1; i >= 0; i--) pending.push(node.children[i]);
  }
  return result;
}
