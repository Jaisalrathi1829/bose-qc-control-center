import { Panel, Button } from '@/components/primitives';
import { useUiStore, type ThemePreference } from '@/stores/uiStore';
import { useDeviceStore } from '@/stores/deviceStore';
import { runtimeMode } from '@/services/ipc';

function Row({
  label,
  detail,
  children,
}: {
  label: string;
  detail?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between gap-6 border-b border-[var(--border-subtle)] py-3 last:border-0">
      <div className="min-w-0">
        <div className="text-[13px] font-medium">{label}</div>
        {detail && (
          <div className="mt-0.5 text-[11.5px] leading-snug text-[var(--text-secondary)]">
            {detail}
          </div>
        )}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

const THEMES: { id: ThemePreference; label: string }[] = [
  { id: 'system', label: 'System' },
  { id: 'light', label: 'Light' },
  { id: 'dark', label: 'Dark' },
];

export function Settings() {
  const theme = useUiStore((s) => s.theme);
  const setTheme = useUiStore((s) => s.setTheme);
  const snapshot = useDeviceStore((s) => s.snapshot);
  const switchSource = useDeviceStore((s) => s.switchSource);
  const pending = useDeviceStore((s) => s.pending);

  const native = runtimeMode() === 'native';

  return (
    <div className="space-y-4">
      <Panel title="Appearance">
        <Row label="Theme" detail="Follows Windows when set to System.">
          <div className="flex gap-1 rounded-lg border border-[var(--border-strong)] p-0.5">
            {THEMES.map((t) => (
              <button
                key={t.id}
                type="button"
                onClick={() => setTheme(t.id)}
                aria-pressed={theme === t.id}
                className={[
                  'rounded-md px-3 py-1.5 text-[12.5px] transition-colors duration-150',
                  theme === t.id
                    ? 'bg-[var(--surface-inset)] font-medium'
                    : 'text-[var(--text-secondary)] hover:bg-[var(--surface-inset)]',
                ].join(' ')}
              >
                {t.label}
              </button>
            ))}
          </div>
        </Row>
      </Panel>

      <Panel
        title="Device Source"
        subtitle="Mock mode stays available permanently, for development and regression testing."
      >
        <Row
          label="Backend"
          detail={
            snapshot?.source === 'mock'
              ? 'Currently simulated. No hardware is being contacted.'
              : 'Currently reading real hardware through Windows.'
          }
        >
          <div className="flex gap-2">
            <Button
              variant={snapshot?.source === 'mock' ? 'primary' : 'secondary'}
              busy={pending === 'switch-source'}
              onClick={() => void switchSource('mock')}
            >
              Simulated
            </Button>
            <Button
              variant={snapshot?.source === 'real' ? 'primary' : 'secondary'}
              disabled={!native}
              busy={pending === 'switch-source'}
              onClick={() => void switchSource('real')}
            >
              Real hardware
            </Button>
          </div>
        </Row>
        {!native && (
          <p className="mt-3 text-[11.5px] leading-relaxed text-[var(--text-tertiary)]">
            Real hardware is only reachable from the desktop application. This is a browser preview.
          </p>
        )}
      </Panel>

      <Panel title="Privacy">
        <Row
          label="Network access"
          detail="This application makes no network requests. There is no backend, no analytics, and no update check."
        >
          <span className="text-[12.5px] text-[var(--color-status-verified)]">None</span>
        </Row>
        <Row
          label="Device identifiers"
          detail="Bluetooth addresses stay in the native layer. Exported reports contain only salted hashes."
        >
          <span className="text-[12.5px] text-[var(--color-status-verified)]">Local only</span>
        </Row>
      </Panel>

      <Panel
        title="General"
        subtitle="Startup, tray and notification behaviour are not wired up in this build."
      >
        <Row label="Start with Windows" detail="Not implemented yet.">
          <span className="text-[12.5px] text-[var(--text-tertiary)]">Unavailable</span>
        </Row>
        <Row label="Minimize to tray" detail="Not implemented yet.">
          <span className="text-[12.5px] text-[var(--text-tertiary)]">Unavailable</span>
        </Row>
        <Row label="Auto-connect" detail="Not implemented yet.">
          <span className="text-[12.5px] text-[var(--text-tertiary)]">Unavailable</span>
        </Row>
      </Panel>
    </div>
  );
}
