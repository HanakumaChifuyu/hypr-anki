import { NavLink } from "react-router";

const items = [
  { to: "/deck", label: "Deck" },
  { to: "/browser", label: "Browser" },
  { to: "/status", label: "Status" },
  { to: "/setting", label: "Setting" },
];

export default function MainNavigation() {
  return (
    <nav
      aria-label="主导航"
      className="grid w-full max-w-[340px] grid-cols-4 rounded-navigation border border-border bg-surface p-[3px] shadow-sm"
    >
      {items.map((item) => (
        <NavLink
          key={item.to}
          to={item.to}
          draggable={false}
          className={({ isActive }) =>
            `flex min-h-[34px] min-w-0 cursor-pointer touch-manipulation select-none items-center justify-center rounded-control px-2 py-1.5 text-[13px] leading-normal font-medium no-underline transition-colors duration-150 focus-visible:outline-2 focus-visible:outline-offset-3 focus-visible:outline-focus motion-reduce:transition-none ${
              isActive
                ? "bg-accent-gradient text-accent-foreground shadow-selected"
                : "text-secondary hover:bg-hover hover:text-hover-foreground active:bg-pressed"
            }`
          }
        >
          {item.label}
        </NavLink>
      ))}
    </nav>
  );
}
