/** Business query result for the UI, including daily counts. IDs must be JS-safe integers. */
export type DeckKind = "deck" | "group";

export interface Deck {
  id: number;
  name: string;
  /** Explicit business type; an empty group is still a group. */
  kind: DeckKind;
  /** Remaining new cards for today, already calculated by the business layer. */
  new_count: number;
  /** Cards currently due for review, already calculated by the business layer. */
  review_count: number;
  is_collapsed: boolean;
  parent_id: number | null;
  config_id: number;
  created_at: number;
  updated_at: number;
}
