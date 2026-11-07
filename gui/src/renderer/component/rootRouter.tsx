import { Outlet, useRouterState } from '@tanstack/react-router';
import { Navigation } from './Navigation';
import { AppRouteItem, routeItems } from '../router';

export default function AppLayout() {
  const pathname = useRouterState({
    select: (state) => state.location.pathname,
  });

  let name = routeItems.filter((v) => {
    return v.path === pathname;
  })[0].name;
  return (
    <div className="flex h-screen flex-col overflow-hidden bg-[#181818] text-zinc-200 m-0">
      <header
        className="relative flex h-10 shrink-0  justify-start 
      border-b  border-white/8 items-center [-webkit-app-region:drag]"
      >
        <div className="w-22 shrink-0" />
        <h1 className="m-0 text-sm font-normal text-[#a1a1aa] select-none">
          {name}
        </h1>
      </header>
      <div className="flex min-h-0 flex-1 pt-0 bg-[#181818]">
        <Navigation />
        <main className="min-w-0 flex-1 overflow-y-auto px-6 pb-6 pt-5 sm:px-5">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
