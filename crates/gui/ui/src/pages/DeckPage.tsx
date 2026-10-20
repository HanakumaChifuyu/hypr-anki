import { useState } from "react";
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
  const [pending, setPending] = useState<{ action: DeckMenuAction; deck: Deck } | null>(null);
  const [name, setName] = useState("");
  const [nameError, setNameError] = useState("");
  const deleting = pending?.action === "delete";
  const renaming = pending?.action === "rename";
  const dialogTitle = deleting ? "删除" : renaming ? "重命名" : pending?.action === "create-folder" ? "新增文件夹" : "新增 deck";

  function handleMenuAction(action: DeckMenuAction, deck: Deck) {
    setName(action === "rename" ? deck.name : "");
    setNameError("");
    setPending({ action, deck });
  }

  function submitAction() {
    if (!pending) return;
    const { action, deck } = pending;
    const trimmed = name.trim();
    if (!deleting && !trimmed) { setNameError("请输入名称。"); return; }
    setDecks((current) => {
      if (action === "delete") {
        const removed = deckSubtreeIds(current, deck.id);
        return current.filter((item) => !removed.has(item.id));
      }
      if (action === "rename") return current.map((item) => item.id === deck.id ? { ...item, name: trimmed } : item);
      const now = Math.floor(Date.now() / 1000);
      return insertDeck(current, {
        id: Math.max(0, ...current.map((item) => item.id)) + 1,
        name: trimmed,
        kind: action === "create-folder" ? "group" : "deck",
        parent_id: deck.kind === "group" ? deck.id : deck.parent_id,
        is_collapsed: false, new_count: 0, review_count: 0,
        config_id: 1, created_at: now, updated_at: now,
      });
    });
    setPending(null);
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
        onCollapsedChange={(id, collapsed) => setDecks((current) => current.map((deck) => deck.id === id ? { ...deck, is_collapsed: collapsed } : deck))}
      />
      <Dialog.Root open={pending !== null} onOpenChange={(open) => { if (!open) setPending(null); }}>
        <Dialog.Portal>
          <Dialog.Overlay className="fixed inset-0 z-50 bg-black/40" />
          <Dialog.Content className="fixed top-1/2 left-1/2 z-50 w-[calc(100%-32px)] max-w-80 -translate-x-1/2 -translate-y-1/2 rounded-xl border border-border bg-surface p-5 text-foreground shadow-xl focus:outline-none">
            <Dialog.Title className="text-base font-semibold">{dialogTitle}</Dialog.Title>
            <Dialog.Description className="mt-2 break-words text-xs leading-5 text-muted">
              {deleting ? `删除“${pending?.deck.name}”及其所有子项？此操作无法撤销。`
                : renaming ? `修改“${pending?.deck.name}”的名称。`
                : pending?.deck.kind === "group" ? `在“${pending.deck.name}”内创建。` : "在当前项目的同级创建。"}
            </Dialog.Description>
            <form onSubmit={(event) => { event.preventDefault(); submitAction(); }}>
              {!deleting && <>
                <label htmlFor="deck-name" className="mt-4 block text-xs text-secondary">名称</label>
                <input id="deck-name" value={name} onChange={(event) => { setName(event.target.value); setNameError(""); }} aria-invalid={!!nameError} aria-describedby={nameError ? "deck-name-error" : undefined} className="mt-1.5 w-full rounded-md border border-border bg-background px-3 py-2 text-sm outline-none focus:border-focus" />
                {nameError && <p id="deck-name-error" role="alert" className="mt-1 text-xs text-danger">{nameError}</p>}
              </>}
              <div className="mt-5 flex justify-end gap-2">
                <Dialog.Close asChild><button type="button" className="rounded-md px-3 py-1.5 text-xs text-secondary hover:bg-hover focus-visible:outline-2 focus-visible:outline-focus">取消</button></Dialog.Close>
                <button type="submit" className={`rounded-md px-3 py-1.5 text-xs font-medium focus-visible:outline-2 focus-visible:outline-focus ${deleting ? "bg-danger/15 text-danger hover:bg-danger/25" : "bg-accent text-background hover:opacity-90"}`}>{deleting ? "删除" : "确定"}</button>
              </div>
            </form>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
    </section>
  );
}
