import {
  createHashHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from '@tanstack/react-router';
import AppLayout from './component/rootRouter';

// Keep the router instance outside component Fast Refresh boundaries.
const rootRoute = createRootRoute({ component: AppLayout });
const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: () => <section aria-label="牌组" />,
});
const reviewRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/review',
  component: () => <section aria-label="复习" />,
});
const manageRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/manage',
  component: () => <section aria-label="管理" />,
});
const statisticsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/statistics',
  component: () => <section aria-label="统计" />,
});

export const router = createRouter({
  routeTree: rootRoute.addChildren([
    indexRoute,
    reviewRoute,
    manageRoute,
    statisticsRoute,
  ]),
  history: createHashHistory(),
  defaultPreload: 'intent',
  scrollRestoration: true,
});

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
