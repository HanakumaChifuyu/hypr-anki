import { Link } from '@tanstack/react-router';
import type { ReactNode } from 'react';
import { routes } from '../router';

export function Icon({
  children,
  className = 'size-4',
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
      className="flex w-14 shrink-0 flex-col items-center gap-1 border-r 
      border-white/8 pt-3 sm:w-14"
      aria-label="主导航"
    >
      {routes.map(({ fullPath, options: { staticData } }) => (
        <Link
          key={fullPath}
          to={fullPath}
          aria-label={staticData!.label}
          title={staticData!.label}
          className="flex size-11 items-center justify-center rounded-lg 
          transition-colors hover:bg-white/5 focus-visible:outline-2 
          focus-visible:outline-offset-2 focus-visible:outline-zinc-400 motion-reduce:transition-none"
          activeOptions={{ exact: true }}
          activeProps={{ className: 'text-zinc-100', 'aria-current': 'page' }}
          inactiveProps={{ className: 'text-[#858585] hover:text-zinc-300' }}
        >
          <Icon>{staticData!.icon}</Icon>
        </Link>
      ))}
    </nav>
  );
}
