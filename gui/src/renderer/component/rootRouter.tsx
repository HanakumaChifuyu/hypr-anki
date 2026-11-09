import { Outlet, useRouterState } from '@tanstack/react-router';
import { Navigation } from './Navigation';

export default function AppLayout() {
  const currentRoute = useRouterState({
    select: (s) => s.matches.at(-1),
  });

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-surface text-content m-0">
      <header
        className="relative flex h-10 shrink-0  justify-start 
      border-b  border-white/8 items-center [-webkit-app-region:drag]"
      >
        <div className="w-22 shrink-0" />
        <h1 className="m-0 text-sm font-normal text-[#a1a1aa] select-none">
          {currentRoute?.staticData.label ?? ' '}
        </h1>
      </header>
      <div className="flex min-h-0 flex-1 pt-0 bg-surface">
        <Navigation />
        <main className="min-w-0 flex-1 overflow-y-auto">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
