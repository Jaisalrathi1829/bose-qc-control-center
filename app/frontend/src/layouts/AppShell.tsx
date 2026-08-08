import type { ReactNode } from 'react';
import { useUiStore, type PageId } from '@/stores/uiStore';
import { useDeviceStore } from '@/stores/deviceStore';
import { StatusDot } from '@/components/primitives';

const NAV: { id: PageId; label: string; icon: ReactNode }[] = [
  {
    id: 'dashboard',
    label: 'Dashboard',
    icon: (
      <path d="M3 9.5L10 4l7 5.5V16a1 1 0 0 1-1 1h-3v-4H7v4H4a1 1 0 0 1-1-1V9.5z" />
    ),
  },
  {
    id: 'device',
    label: 'Device',
    icon: (
      <path d="M4 11V9a6 6 0 1 1 12 0v2m-12 0v3a1 1 0 0 0 1 1h1V11H4zm12 0v4h1a1 1 0 0 0 1-1v-3h-2z" />
    ),
  },
  {
    id: 'diagnostics',
    label: 'Diagnostics',
    icon: <path d="M3 10h3l2-5 4 10 2-5h3" />,
  },
  {
    id: 'profiles',
    label: 'Profiles',
    icon: <path d="M4 5h12M4 10h12M4 15h7" />,
  },
  {
    id: 'settings',
    label: 'Settings',
    icon: (
      <>
        <circle cx="10" cy="10" r="2.5" />
        <path d="M10 3v2m0 10v2m7-7h-2M5 10H3m12-5l-1.5 1.5M6.5 13.5L5 15m10 0l-1.5-1.5M6.5 6.5L5 5" />
      </>
    ),
  },
];

export function AppShell({ children }: { children: ReactNode }) {
  const page = useUiStore((s) => s.page);
  const setPage = useUiStore((s) => s.setPage);
  const snapshot = useDeviceStore((s) => s.snapshot);

  const connected = snapshot?.connection === 'connected';

  return (
    <div className="flex h-full">
      <nav
        className="flex w-[196px] shrink-0 flex-col border-r border-[var(--border-subtle)] bg-[var(--surface-panel)] px-3 py-4"
        aria-label="Main"
      >
        <div className="mb-6 px-2">
          <div className="font-[var(--font-display)] text-[13.5px] font-semibold leading-tight tracking-tight">
            QC Control Center
          </div>
          <div className="mt-1.5 flex items-center gap-1.5">
            <StatusDot state={connected ? 'on' : 'off'} />
            <span className="text-[11px] text-[var(--text-tertiary)]">
              {connected ? 'Connected' : 'Not connected'}
            </span>
          </div>
        </div>

        <ul className="space-y-0.5">
          {NAV.map((item) => {
            const active = page === item.id;
            return (
              <li key={item.id}>
                <button
                  type="button"
                  onClick={() => setPage(item.id)}
                  aria-current={active ? 'page' : undefined}
                  className={[
                    'flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-[13px] transition-colors duration-150',
                    active
                      ? 'bg-[var(--surface-inset)] font-medium text-[var(--text-primary)]'
                      : 'text-[var(--text-secondary)] hover:bg-[var(--surface-inset)]',
                  ].join(' ')}
                >
                  <svg
                    width="17"
                    height="17"
                    viewBox="0 0 20 20"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="1.5"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    className={active ? 'text-[var(--color-accent-500)]' : ''}
                    aria-hidden="true"
                  >
                    {item.icon}
                  </svg>
                  {item.label}
                </button>
              </li>
            );
          })}
        </ul>

        <div className="mt-auto px-2 pt-4">
          <p className="text-[10.5px] leading-relaxed text-[var(--text-tertiary)]">
            Fully local. No cloud, no accounts, no telemetry.
          </p>
        </div>
      </nav>

      <main className="flex-1 overflow-y-auto px-6 py-5">
        <div className="mx-auto max-w-3xl">{children}</div>
      </main>
    </div>
  );
}
