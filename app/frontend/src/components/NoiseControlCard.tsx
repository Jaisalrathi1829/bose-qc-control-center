import { Panel, EmptyState } from './primitives';
import { CapabilityBadge } from './CapabilityBadge';
import type { CapabilityRecord } from '@/types/capability';
import type { NoiseControlState, NoiseControlMode } from '@/types/device';
import { useDeviceStore } from '@/stores/deviceStore';

const MODES: { id: NoiseControlMode; label: string; hint: string }[] = [
  { id: 'quiet', label: 'Quiet', hint: 'Maximum noise cancellation' },
  { id: 'aware', label: 'Aware', hint: 'Let the outside in' },
  { id: 'custom', label: 'Custom', hint: 'Your saved level' },
];

/**
 * Noise control.
 *
 * The rendering is driven entirely by the capability record:
 *
 *  - `unsupported` — the card explains why and offers nothing.
 *  - `unknown`     — we say plainly that it has not been verified, and do not
 *                    draw controls that would imply it works.
 *  - `supported` /
 *    `experimental`— controls are shown but carry an explicit caveat, and the
 *                    current mode is only displayed if the device told us one.
 *  - `verified`    — normal controls.
 */
export function NoiseControlCard({
  capability,
  state,
  connected,
}: {
  capability: CapabilityRecord;
  state: NoiseControlState | null;
  connected: boolean;
}) {
  const run = useDeviceStore((s) => s.run);
  const pending = useDeviceStore((s) => s.pending);

  const header = (
    <CapabilityBadge status={capability.status} hardwareVerified={capability.hardwareVerified} />
  );

  if (capability.status === 'unsupported') {
    return (
      <Panel title="Noise Control" actions={header}>
        <EmptyState title="Not available on this device" detail={capability.evidence} />
      </Panel>
    );
  }

  if (capability.status === 'unknown') {
    return (
      <Panel title="Noise Control" actions={header}>
        <EmptyState
          title="Not yet verified on this device"
          detail={
            <>
              {capability.evidence} Run <strong>Diagnostics</strong> with the headphones connected
              to investigate what this device actually exposes.
            </>
          }
        />
      </Panel>
    );
  }

  const caveat = capability.status !== 'verified';

  return (
    <Panel
      title="Noise Control"
      subtitle={
        caveat
          ? 'This control has not been confirmed on your headphones. Results will be reported honestly.'
          : undefined
      }
      actions={header}
    >
      <div className="grid grid-cols-3 gap-2" role="group" aria-label="Noise control mode">
        {MODES.map((m) => {
          // Only reflect a selected mode if the device actually reported one.
          const selected = state?.mode === m.id;
          return (
            <button
              key={m.id}
              type="button"
              disabled={!connected || pending !== null}
              aria-pressed={selected}
              onClick={() => void run({ kind: 'setNoiseControl', mode: m.id })}
              className={[
                'rounded-lg border px-3 py-3 text-left transition-all duration-150',
                'disabled:cursor-not-allowed disabled:opacity-45',
                selected
                  ? 'border-[var(--color-accent-500)] bg-[color-mix(in_oklch,var(--color-accent-500)_12%,transparent)]'
                  : 'border-[var(--border-strong)] hover:bg-[var(--surface-inset)]',
              ].join(' ')}
            >
              <div className="text-[13px] font-medium">{m.label}</div>
              <div className="mt-0.5 text-[11px] leading-snug text-[var(--text-secondary)]">
                {m.hint}
              </div>
            </button>
          );
        })}
      </div>

      {state === null && connected && (
        <p className="mt-3 text-[12px] text-[var(--text-tertiary)]">
          The device has not reported its current mode, so none is shown as active.
        </p>
      )}

      {state && (
        <p className="mt-3 text-[12px] text-[var(--text-tertiary)]">
          Reported by device as <strong className="text-[var(--text-secondary)]">{state.mode}</strong>
          {state.source === 'simulated' && ' (simulated)'}
        </p>
      )}
    </Panel>
  );
}
