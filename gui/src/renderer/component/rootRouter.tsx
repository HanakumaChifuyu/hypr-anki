import { Outlet, useRouterState } from '@tanstack/react-router';
import { Navigation } from './Navigation';

const titles: Record<string, string> = {
  '/': '牌组',
  '/review': '复习',
  '/manage': '管理',
  '/statistics': '统计',
};

export default function AppLayout() {
  const pathname = useRouterState({
    select: (state) => state.location.pathname,
  });
  return (
    <div className="flex h-screen flex-col overflow-hidden bg-[#11111] text-zinc-200 m-0">
      <header
        className="relative flex h-10 shrink-0  justify-start 
      border-b border-[#353535] items-center [-webkit-app-region:drag]"
      >
        <div className="w-22 shrink-0" />
        <h1 className="m-0 text-sm font-normal text-[#999] select-none">
          {titles[pathname] ?? '牌组'}
        </h1>
      </header>
      <div className="flex min-h-0 flex-1 pt-0">
        <Navigation />
        <main className="min-w-0 flex-1 overflow-y-auto px-6 pb-6 pt-5 sm:px-5">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
