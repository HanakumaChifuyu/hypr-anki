import { Navigate, NavLink, Outlet, Route, Routes } from "react-router";

const pages = [
  { path: "deck", label: "Deck", description: "卡组将在这里显示。" },
  { path: "browser", label: "Browser", description: "在这里浏览和管理卡片。" },
  { path: "status", label: "Status", description: "在这里查看学习统计。" },
  { path: "setting", label: "Setting", description: "在这里调整应用设置。" },
];

function MainLayout() {
  return (
    <div className="app-shell">
      <header className="app-header">
        <nav className="main-nav" aria-label="主导航">
          {pages.map((page) => (
            <NavLink
              key={page.path}
              to={`/${page.path}`}
              className={({ isActive }) =>
                `main-nav-link${isActive ? " is-active" : ""}`
              }
            >
              {page.label}
            </NavLink>
          ))}
        </nav>
      </header>
      <main className="page-content">
        <Outlet />
      </main>
    </div>
  );
}

function PagePlaceholder({ title, description }: { title: string; description: string }) {
  return (
    <section className="page-placeholder">
      <h1>{title}</h1>
      <p>{description}</p>
    </section>
  );
}

export default function App() {
  return (
    <Routes>
      <Route element={<MainLayout />}>
        <Route index element={<Navigate to="/deck" replace />} />
        {pages.map((page) => (
          <Route
            key={page.path}
            path={page.path}
            element={<PagePlaceholder title={page.label} description={page.description} />}
          />
        ))}
        <Route path="*" element={<Navigate to="/deck" replace />} />
      </Route>
    </Routes>
  );
}
