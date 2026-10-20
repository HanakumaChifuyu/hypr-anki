import { test } from "node:test";
import assert from "node:assert/strict";
import { buildDeckTree, moveDeck } from "../src/features/decks/deck-tree.ts";

const deck = (id, parent_id = null) => ({
  id, name: String(id), kind: "deck", parent_id, is_collapsed: false,
  new_count: 0, review_count: 0, config_id: 1, created_at: 0, updated_at: 0,
});
const input = () => [deck(1), deck(2, 1), deck(3, 2), deck(4, 1), deck(5)];
const ids = (decks) => decks.map((deck) => deck.id);

test("build preserves roots, nested children and sibling order", () => {
  const roots = buildDeckTree(input());
  assert.deepEqual(roots.map((node) => node.deck.id), [1, 5]);
  assert.deepEqual(roots[0].children.map((node) => node.deck.id), [2, 4]);
  assert.equal(roots[0].children[0].children[0].deck.id, 3);
  assert.deepEqual(buildDeckTree([]), []);
});

test("malformed lists fail explicitly rather than silently losing nodes", () => {
  assert.throws(() => buildDeckTree([deck(1), deck(1)]));
  assert.throws(() => buildDeckTree([deck(2, 1), deck(1)]));
  assert.throws(() => buildDeckTree([deck(1, 1)]));
  assert.throws(() => buildDeckTree([deck(1, 99)]));
  assert.throws(() => buildDeckTree([deck(Number.MAX_SAFE_INTEGER + 1)]));
});

test("moving a subtree to root preserves descendants and input", () => {
  const original = input();
  const snapshot = structuredClone(original);
  const next = moveDeck(original, { deck_id: 2, parent_id: null, index: 1 });
  assert.deepEqual(ids(next), [1, 4, 2, 3, 5]);
  assert.equal(next.find((deck) => deck.id === 2).parent_id, null);
  assert.equal(next.find((deck) => deck.id === 3).parent_id, 2);
  assert.deepEqual(original, snapshot);
});

test("sibling reorder works in both directions and preserves subtrees", () => {
  const forward = moveDeck(input(), { deck_id: 2, parent_id: 1, index: 1 });
  assert.deepEqual(ids(forward), [1, 4, 2, 3, 5]);
  const backward = moveDeck(forward, { deck_id: 2, parent_id: 1, index: 0 });
  assert.deepEqual(ids(backward), [1, 2, 3, 4, 5]);
});

test("empty decks accept children and are expanded on drop", () => {
  const original = input().map((value) => value.id === 5 ? { ...value, is_collapsed: true } : value);
  const next = moveDeck(original, { deck_id: 2, parent_id: 5, index: 0 });
  assert.deepEqual(ids(next), [1, 4, 5, 2, 3]);
  assert.equal(next.find((deck) => deck.id === 2).parent_id, 5);
  assert.equal(next.find((deck) => deck.id === 5).is_collapsed, false);
});

test("self, descendant, missing targets and invalid indices are rejected", () => {
  for (const parent_id of [1, 2, 3]) {
    assert.throws(() => moveDeck(input(), { deck_id: 1, parent_id, index: 0 }));
  }
  assert.throws(() => moveDeck(input(), { deck_id: 99, parent_id: null, index: 0 }));
  assert.throws(() => moveDeck(input(), { deck_id: 1, parent_id: 99, index: 0 }));
  for (const index of [-1, 99, 0.5]) {
    assert.throws(() => moveDeck(input(), { deck_id: 2, parent_id: null, index }));
  }
});
