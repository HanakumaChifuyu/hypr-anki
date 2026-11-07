import {
  createHashHistory,
  createRootRoute,
  createRoute,
  createRouter,
  RouteComponent,
} from '@tanstack/react-router';
import AppLayout from './component/rootRouter';

export class AppRouteItem {
  constructor(
    public path: string,
    public name: string,
    public component: RouteComponent | undefined = undefined,
  ) {}
}

export const routeItems = [
  new AppRouteItem('/', '牌组'),
  new AppRouteItem('/review', '复习'),
  new AppRouteItem('/manage', '管理'),
  new AppRouteItem('/statistic', '统计'),
];

const routes = routeItems.map((v) => {
  return createRoute({
    getParentRoute: () => rootRoute,
    path: v.path,
    component: () => <section aria-label={v.name} />,
  });
});

// Keep the router instance outside component Fast Refresh boundaries.
const rootRoute = createRootRoute({ component: AppLayout });

export const router = createRouter({
  routeTree: rootRoute.addChildren(routes),
  history: createHashHistory(),
  defaultPreload: 'intent',
  scrollRestoration: true,
});

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
