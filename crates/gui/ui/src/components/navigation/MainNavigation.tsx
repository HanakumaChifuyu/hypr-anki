import { NavLink } from "react-router";

const items = [
  { to: "/deck", label: "Decks" },
  { to: "/browser", label: "Browser" },
  { to: "/status", label: "Stats" },
  { to: "/setting", label: "Settings" },
];

export default function MainNavigation() {
  return (
    <nav
      aria-label="主导航"
      className="grid w-full grid-cols-4 border-b border-border"
    >
      {items.map((item) => (
        <NavLink
          key={item.to}
          to={item.to}
          draggable={false}
          className={({ isActive }) =>
            `-mb-px flex min-h-[34px] min-w-0 cursor-pointer touch-manipulation select-none items-center justify-center border-b-2 px-2 py-1.5 text-[13px] leading-normal font-medium no-underline transition-colors duration-150 focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-focus motion-reduce:transition-none ${
              isActive
                ? "border-accent text-accent"
                : "border-transparent text-muted hover:text-foreground active:text-accent"
            }`
          }
        >
          {item.label}
        </NavLink>
      ))}
    </nav>
  );
}
