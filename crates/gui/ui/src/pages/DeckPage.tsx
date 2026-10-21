import { useRef, useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import DeckTree from "../components/decks/DeckTree";
import type { DeckMenuAction } from "../components/decks/DeckContextMenu";
import { deckSubtreeIds, insertDeck } from "../features/decks/deck-actions";
import { moveDeck, type DeckMove } from "../features/decks/deck-tree";
import { sampleDecks } from "../features/decks/sample-decks";
import type { Deck } from "../types/deck";

export default function DeckPage() {
  const [decks, setDecks] = useState<Deck[]>(sampleDecks);
  const [error, setError] = useState<string | null>(null);
  const [pendingDelete, setPendingDelete] = useState<Deck | null>(null);
  const nextId = useRef(Math.max(0, ...sampleDecks.map((deck) => deck.id)) + 1);

  function handleMenuAction(action: DeckMenuAction, deck: Deck) {
    if (action === "delete") setPendingDelete(deck);
  }

  function handleCreate(draft: Deck) {
    const now = Math.floor(Date.now() / 1000);
    const deck = { ...draft, id: nextId.current++, created_at: now, updated_at: now };
    setDecks((current) => insertDeck(current, deck));
  }

  function handleRename(id: number, name: string) {
    setDecks((current) => current.map((deck) => deck.id === id ? { ...deck, name } : deck));
  }

  function deletePending() {
    if (!pendingDelete) return;
    setDecks((current) => {
      const removed = deckSubtreeIds(current, pendingDelete.id);
      return current.filter((deck) => !removed.has(deck.id));
    });
    setPendingDelete(null);
  }

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
    <section aria-label="卡组" className="mx-auto flex h-full min-h-0 w-full flex-col">
      <div className="mb-4 flex shrink-0 items-center gap-2.5 px-1">
        <h1 className="text-xl leading-none font-semibold tracking-tight">Decks</h1>
        <span className="rounded-md bg-hover px-1.5 py-0.5 text-[11px] leading-4 font-medium text-muted tabular-nums">{decks.length}</span>
      </div>
      {error && <p role="alert" className="py-2 text-sm text-muted">{error}</p>}
      <div className="mb-1 grid shrink-0 grid-cols-[minmax(0,1fr)_40px] items-center gap-x-1 border-b border-border/50 pb-2 pr-1 text-center text-[10px] font-medium text-muted">
        <span className="pl-1 text-left">牌组</span>
      </div>
      <DeckTree
        decks={decks}
        onMove={handleMove}
        onMenuAction={handleMenuAction}
        onCreate={handleCreate}
        onRename={handleRename}
        onCollapsedChange={(id, collapsed) => setDecks((current) => current.map((deck) => deck.id === id ? { ...deck, is_collapsed: collapsed } : deck))}
      />
      <Dialog.Root open={pendingDelete !== null} onOpenChange={(open) => { if (!open) setPendingDelete(null); }}>
        <Dialog.Portal>
          <Dialog.Overlay className="fixed inset-0 z-50 bg-black/40" />
          <Dialog.Content className="fixed top-1/2 left-1/2 z-50 w-[calc(100%-32px)] max-w-80 -translate-x-1/2 -translate-y-1/2 rounded-xl border border-border bg-surface p-5 text-foreground shadow-xl focus:outline-none">
            <Dialog.Title className="text-base font-semibold">删除</Dialog.Title>
            <Dialog.Description className="mt-2 break-words text-xs leading-5 text-muted">
              删除“{pendingDelete?.name}”及其所有子项？此操作无法撤销。
            </Dialog.Description>
            <div className="mt-5 flex justify-end gap-2">
              <Dialog.Close asChild><button type="button" className="rounded-md px-3 py-1.5 text-xs text-secondary hover:bg-hover focus-visible:outline-2 focus-visible:outline-focus">取消</button></Dialog.Close>
              <button type="button" onClick={deletePending} className="rounded-md bg-danger/15 px-3 py-1.5 text-xs font-medium text-danger hover:bg-danger/25 focus-visible:outline-2 focus-visible:outline-focus">删除</button>
            </div>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
    </section>
  );
}
