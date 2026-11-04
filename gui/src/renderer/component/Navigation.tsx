import { Link } from '@tanstack/react-router';
import type { ReactNode } from 'react';

export function Icon({
  children,
  className = 'size-6',
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

const items = [
  {
    to: '/',
    label: '牌组',
    icon: (
      <>
        <path d="m3 10 4-5h10l4 5-2 2H5z" />
        <path d="M4 16h16M5 20h14" />
      </>
    ),
  },
  {
    to: '/review',
    label: '复习',
    icon: (
      <>
        <path d="M12 6v15M12 6C9 3 5 3 2 4v15c4-1 7 0 10 2 3-2 6-3 10-2V4c-3-1-7-1-10 2Z" />
      </>
    ),
  },
  {
    to: '/manage',
    label: '管理',
    icon: (
      <>
        <path d="M11 20H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h6l3 3h7a2 2 0 0 1 2 2v4" />
        <circle cx="17" cy="17" r="3" />
        <path d="m19.5 19.5 2.5 2.5" />
      </>
    ),
  },
  {
    to: '/statistics',
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
  },
] as const;

export function Navigation() {
  return (
    <nav
      className="flex w-14 shrink-0 flex-col items-center gap-3 border-r 
      border-[#353535] pt-3 sm:w-14"
      aria-label="主导航"
    >
      {items.map(({ to, label, icon }) => (
        <Link
          key={to}
          to={to}
          aria-label={label}
          title={label}
          className="flex size-11 items-center justify-center rounded-lg 
          transition-colors hover:bg-white/5 focus-visible:outline-2 
          focus-visible:outline-offset-2 focus-visible:outline-zinc-400 motion-reduce:transition-none"
          activeOptions={{ exact: true }}
          activeProps={{ className: 'text-zinc-100', 'aria-current': 'page' }}
          inactiveProps={{ className: 'text-[#858585] hover:text-zinc-300' }}
        >
          <Icon>{icon}</Icon>
        </Link>
      ))}
    </nav>
  );
}
