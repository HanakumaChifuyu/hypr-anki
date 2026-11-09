import {
  createHashHistory,
  createRootRoute,
  createRoute,
  createRouter,
  RouteComponent,
} from '@tanstack/react-router';
import type { ReactNode } from 'react';
import AppLayout from './component/rootRouter';
import { Deck } from './pages/deck-tree';

type NavigationMetadata = {
  label: string;
  icon: ReactNode;
};

// Keep the router instance outside component Fast Refresh boundaries.
const rootRoute = createRootRoute({ component: AppLayout });

function defineAppRoute<const TPath extends string>({
  path,
  label,
  icon,
  component,
}: NavigationMetadata & { path: TPath; component?: RouteComponent }) {
  return createRoute({
    getParentRoute: () => rootRoute,
    path,
    staticData: { label, icon },
    component: component ?? (() => <section aria-label={label} />),
  });
}

export const routes = [
  defineAppRoute({
    path: '/',
    label: '牌组',
    icon: (
      <>
        <path d="m3 10 4-5h10l4 5-2 2H5z" />
        <path d="M4 16h16M5 20h14" />
      </>
    ),
    component: Deck,
  }),
  defineAppRoute({
    path: '/review',
    label: '复习',
    icon: (
      <>
        <path d="M12 6v15M12 6C9 3 5 3 2 4v15c4-1 7 0 10 2 3-2 6-3 10-2V4c-3-1-7-1-10 2Z" />
      </>
    ),
  }),
  defineAppRoute({
    path: '/manage',
    label: '管理',
    icon: (
      <>
        <path d="M11 20H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h6l3 3h7a2 2 0 0 1 2 2v4" />
        <circle cx="17" cy="17" r="3" />
        <path d="m19.5 19.5 2.5 2.5" />
      </>
    ),
  }),
  defineAppRoute({
    path: '/statistics',
    label: '统计',
    icon: (
      <>
        <rect
          x="3"
          y="11"
          width="4"
          height="10"
          rx="1"
          fill="currentColor"
          stroke="none"
        />
        <rect
          x="10"
          y="4"
          width="4"
          height="17"
          rx="1"
          fill="currentColor"
          stroke="none"
        />
        <rect
          x="17"
          y="8"
          width="4"
          height="13"
          rx="1"
          fill="currentColor"
          stroke="none"
        />
      </>
    ),
  }),
] as const;

export const router = createRouter({
  routeTree: rootRoute.addChildren(routes),
  history: createHashHistory(),
  defaultPreload: 'intent',
  scrollRestoration: true,
});

declare module '@tanstack/react-router' {
  interface StaticDataRouteOption {
    label?: string;
    icon?: ReactNode;
  }

  interface Register {
    router: typeof router;
  }
}
