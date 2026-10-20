import type { Deck, DeckKind } from "../../types/deck";

// Preview data only; replace at the page boundary when the Rust API is ready.
const seeds: [number, string, number | null, boolean, number, number, DeckKind][] = [
  [1, "Languages", null, false, 34, 12, "group"],
  [2, "Japanese", 1, false, 26, 8, "deck"],
  [3, "Vocabulary", 2, false, 18, 6, "deck"],
  [4, "Grammar", 2, false, 8, 2, "deck"],
  [5, "English", 1, true, 8, 4, "deck"],
  [6, "Daily expressions", 5, false, 8, 4, "deck"],
  [7, "Computer science", null, true, 14, 3, "group"],
  [8, "Algorithms", 7, false, 14, 3, "deck"],
  [9, "Reading", null, false, 0, 2, "deck"],
];

export const sampleDecks: Deck[] = seeds.map(([id, name, parent_id, is_collapsed, new_count, review_count, kind]) => ({
  id,
  name,
  kind,
  parent_id,
  is_collapsed,
  new_count,
  review_count,
  config_id: 1,
  created_at: 0,
  updated_at: 0,
}));
