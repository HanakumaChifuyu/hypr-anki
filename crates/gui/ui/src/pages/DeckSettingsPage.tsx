import { Link, useLocation, useParams } from "react-router";
import PagePlaceholder from "./PagePlaceholder";

export default function DeckSettingsPage() {
  const { deckId } = useParams();
  const { state } = useLocation();
  const label = state?.kind === "group" ? "分组设置" : "卡组设置";
  const name = typeof state?.deckName === "string" ? state.deckName : `卡组 ${deckId}`;

  return (
    <section>
      <Link to="/deck" className="text-sm text-muted hover:text-foreground">← 返回卡组</Link>
      <p className="mt-6 text-xs text-muted">{label}</p>
      <PagePlaceholder title={name} description="设置选项即将开放。" />
    </section>
  );
}
