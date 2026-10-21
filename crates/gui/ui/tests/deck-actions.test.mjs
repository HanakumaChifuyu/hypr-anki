import test from 'node:test';
import assert from 'node:assert/strict';
import { deckSubtreeIds, insertDeck } from '../src/features/decks/deck-actions.ts';

const decks = [
  { id: 1, parent_id: null, kind: 'group', is_collapsed: true },
  { id: 2, parent_id: 1, kind: 'group' },
  { id: 3, parent_id: 2, kind: 'deck' },
  { id: 4, parent_id: 1, kind: 'deck' },
  { id: 5, parent_id: null, kind: 'deck' },
];

test('subtree selection includes nested children but excludes siblings', () => {
  assert.deepEqual([...deckSubtreeIds(decks, 2)], [2, 3]);
  assert.deepEqual(decks.filter(deck => !deckSubtreeIds(decks, 2).has(deck.id)).map(deck => deck.id), [1, 4, 5]);
});

test('creating a child preserves preorder, expands its folder and keeps input unchanged', () => {
  const next = insertDeck(decks, { id: 6, parent_id: 1, kind: 'group' });
  assert.deepEqual(next.map(deck => deck.id), [1, 2, 3, 4, 6, 5]);
  assert.equal(next[0].is_collapsed, false);
  assert.equal(decks[0].is_collapsed, true);
  const withChild = insertDeck(next, { id: 7, parent_id: 6, kind: 'deck' });
  assert.deepEqual(withChild.map(deck => deck.id), [1, 2, 3, 4, 6, 7, 5]);
});

test('new root is appended and files cannot receive new children', () => {
  assert.equal(insertDeck(decks, { id: 6, parent_id: null, kind: 'deck' }).at(-1).id, 6);
  assert.throws(() => insertDeck(decks, { id: 6, parent_id: 5, kind: 'deck' }));
  assert.throws(() => insertDeck(decks, { id: 6, parent_id: 99, kind: 'deck' }));
});

test('creating from a folder inserts its first child and expands it', () => {
  const next = insertDeck(decks, { id: 6, parent_id: 1, kind: 'deck' }, 1);
  assert.deepEqual(next.map(deck => deck.id), [1, 6, 2, 3, 4, 5]);
  assert.equal(next[0].is_collapsed, false);
  assert.equal(decks[0].is_collapsed, true);
  const nested = insertDeck(decks, { id: 6, parent_id: 2, kind: 'group' }, 2);
  assert.deepEqual(nested.map(deck => deck.id), [1, 2, 6, 3, 4, 5]);
});

test('creating from a deck inserts immediately after it at the same level', () => {
  const nested = insertDeck(decks, { id: 6, parent_id: 2, kind: 'deck' }, 3);
  assert.deepEqual(nested.map(deck => deck.id), [1, 2, 3, 6, 4, 5]);
  const roots = [...decks, { id: 7, parent_id: null, kind: 'deck' }];
  const next = insertDeck(roots, { id: 6, parent_id: null, kind: 'group' }, 5);
  assert.deepEqual(next.map(deck => deck.id), [1, 2, 3, 4, 5, 6, 7]);
  assert.deepEqual(roots.map(deck => deck.id), [1, 2, 3, 4, 5, 7]);
});

test('creation rejects missing anchors and anchors in a different parent', () => {
  const draft = { id: 6, parent_id: 1, kind: 'deck' };
  assert.throws(() => insertDeck(decks, draft, 99));
  assert.throws(() => insertDeck(decks, draft, 3));
  assert.throws(() => insertDeck(decks, { ...draft, parent_id: null }, 1));
});
