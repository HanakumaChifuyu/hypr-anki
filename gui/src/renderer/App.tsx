import { useState } from "react"
import { createHashHistory, createRootRoute, createRoute, createRouter, Link, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";

const hashHistory = createHashHistory();

const rootRoute = createRootRoute({
  component: () =>
  (
    <>
      <div className="p-2 flex gap-2 justify-start">
        <Link to="/" className="[&.active]:font-bold" > Home </Link>
        {"|"}
        <Link to="/about" className="[&.active]:font-bold" > About </Link>
      </div>
      <hr />
      <Outlet />
      <TanStackRouterDevtools />
    </>
  )

})

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: () => (
    <>
      <div className="flex border-2 border-black bg-zinc-500">
        <span className="font-bold"> Here is Home </span>
      </div>
    </>
  )

})

const aboutRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/about',
  component: function About() {
    return <div className="p-2">Hello from About!</div>
  },
})

const routeTree = rootRoute.addChildren([indexRoute, aboutRoute])

export const router = createRouter({
  routeTree,
  history: hashHistory,
  defaultPreload: 'intent',
  scrollRestoration: true,
})

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}

export default function App() {
  const [count, setCount] = useState(0);
  return (
    <div
      className="flex min-h-screen bg-zinc-500" >
    </div>
  )
}


