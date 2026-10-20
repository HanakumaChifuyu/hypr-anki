import { useNavigate } from "react-router";
import { useState } from "react";
import DeckTree from "../components/decks/DeckTree";
import { moveDeck, type DeckMove } from "../features/decks/deck-tree";
import { sampleDecks } from "../features/decks/sample-decks";
import type { Deck } from "../types/deck";

export default function DeckPage() {
  const navigate = useNavigate();
  const [decks, setDecks] = useState<Deck[]>(sampleDecks);
  const [error, setError] = useState<string | null>(null);

  function handleMove(move: DeckMove) {
    try {
      const next = moveDeck(decks, move);
      setDecks(next);
      setError(null);
    } catch (error) {
      setError(error instanceof Error ? error.message : "移动失败，请重试。");
    }
  }

  return (
    <section aria-label="卡组" className="mx-auto w-full max-w-[340px]">
      <div className="mb-4 flex items-center gap-2.5 px-1">
        <h1 className="text-xl leading-none font-semibold tracking-tight">Decks</h1>
        <span className="rounded-md bg-hover px-1.5 py-0.5 text-[11px] leading-4 font-medium text-muted tabular-nums">{decks.length}</span>
      </div>
      {error && <p role="alert" className="py-2 text-sm text-muted">{error}</p>}
      <div className="mb-1 grid grid-cols-[minmax(0,1fr)_40px_24px] items-center gap-x-1 border-b border-border/50 pb-2 pr-1 text-center text-[10px] font-medium text-muted">
        <span className="pl-1 text-left">牌组</span>
        <span className="sr-only">设置</span>
      </div>
      <DeckTree
        decks={decks}
        onMove={handleMove}
        onSettings={(deck) => navigate(`/deck/${deck.id}/settings`, { state: { deckName: deck.name, kind: deck.kind } })}
        onCollapsedChange={(id, collapsed) => setDecks((current) => current.map((deck) => deck.id === id ? { ...deck, is_collapsed: collapsed } : deck))}
      />
    </section>
  );
}
