import { Link } from '@tanstack/react-router';
import type { ReactNode } from 'react';
import { routes } from '../router';

export function Icon({
  children,
  className = 'size-2.5 scale-150',
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <svg
      className={className}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {children}
    </svg>
  );
}

export function Navigation() {
  return (
    <nav
      className="flex w-12 shrink-0 flex-col items-center gap-1 border-r 
      border-white/8 pt-1.5 "
      aria-label="主导航"
    >
      {routes.map(({ fullPath, options: { staticData } }) => (
        <Link
          key={fullPath}
          to={fullPath}
          aria-label={staticData!.label}
          className="group relative flex size-10 items-center justify-center rounded-lg
          transition-colors hover:bg-white/5 focus-visible:outline-2 
          focus-visible:outline-offset-2 focus-visible:outline-zinc-400 motion-reduce:transition-none
          hover:bg-gray-200
          "
          activeOptions={{ exact: true }}
          activeProps={{ className: 'text-zinc-100', 'aria-current': 'page' }}
          inactiveProps={{ className: 'text-[#858585] hover:text-zinc-300' }}
        >
          <Icon>{staticData!.icon}</Icon>
          <span
            aria-hidden="true"
            className="invisible absolute left-full top-1/2 z-50 -translate-y-1/2
            pl-2 transition-[visibility] duration-0 delay-0
            group-hover:visible group-hover:delay-300
            group-focus-visible:visible group-focus-visible:delay-0"
          >
            <span
              className="relative block whitespace-nowrap rounded-md border border-white/10
              bg-zinc-800 px-2.5 py-1.5 text-xs font-medium text-zinc-100
              shadow-lg shadow-black/30 before:absolute before:-left-1 before:top-1/2
              before:size-2 before:-translate-y-1/2 before:rotate-45
              before:border-b before:border-l before:border-white/10 before:bg-zinc-800"
            >
              {staticData!.label}
            </span>
          </span>
        </Link>
      ))}
    </nav>
  );
}
