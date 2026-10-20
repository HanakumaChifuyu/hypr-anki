import { Outlet } from "react-router";
import MainNavigation from "../navigation/MainNavigation";

const isMacOS = /Macintosh|MacIntel/.test(navigator.userAgent);

export default function AppLayout() {
  return (
    <div className="mx-auto flex h-dvh w-full max-w-app flex-col overflow-hidden px-5 pb-4 max-[360px]:px-4">
      {isMacOS && (
        <div data-tauri-drag-region className="relative -mx-5 h-11 shrink-0 select-none max-[360px]:-mx-4">
          <span className="pointer-events-none absolute inset-0 flex items-center justify-center text-[13px] font-medium text-muted">HyprAnki</span>
        </div>
      )}
      <header className={`shrink-0 ${isMacOS ? "pt-3" : "pt-6"}`}>
        <MainNavigation />
      </header>
      <main className="mt-8 min-h-0 flex-1 overflow-hidden">
        <Outlet />
      </main>
    </div>
  );
}
