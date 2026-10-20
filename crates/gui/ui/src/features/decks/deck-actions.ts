import type { Deck } from "../../types/deck";

/** Parents precede children in the business query result. Includes the target. */
export function deckSubtreeIds(decks: readonly Deck[], id: number): Set<number> {
  const ids = new Set([id]);
  for (const deck of decks) {
    if (deck.parent_id !== null && ids.has(deck.parent_id)) ids.add(deck.id);
  }
  return ids;
}

export function insertDeck(decks: readonly Deck[], deck: Deck): Deck[] {
  if (deck.parent_id === null) return [...decks, deck];
  const parent = decks.find((item) => item.id === deck.parent_id);
  if (!parent || parent.kind !== "group") throw new Error("只能在文件夹内创建项目。");
  const descendants = deckSubtreeIds(decks, parent.id);
  let index = decks.findIndex((item) => item.id === parent.id) + 1;
  while (index < decks.length && descendants.has(decks[index].id)) index++;
  const result = decks.map((item) => item.id === parent.id ? { ...item, is_collapsed: false } : item);
  result.splice(index, 0, deck);
  return result;
}
