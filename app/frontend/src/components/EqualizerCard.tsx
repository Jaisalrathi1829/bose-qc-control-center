import { useEffect, useState } from 'react';
import { Panel, Button, EmptyState } from './primitives';
import { CapabilityBadge } from './CapabilityBadge';
import type { CapabilityRecord } from '@/types/capability';
import type { EqState, EqSettings } from '@/types/device';
import { useDeviceStore } from '@/stores/deviceStore';

const BANDS: { key: keyof EqSettings; label: string }[] = [
  { key: 'bass', label: 'Bass' },
  { key: 'mid', label: 'Mid' },
  { key: 'treble', label: 'Treble' },
];

export function EqualizerCard({
  capability,
  state,
  connected,
}: {
  capability: CapabilityRecord;
  state: EqState | null;
  connected: boolean;
}) {
  const run = useDeviceStore((s) => s.run);
  const pending = useDeviceStore((s) => s.pending);

  // Local draft so dragging feels immediate. It is seeded from device state and
  // re-seeded whenever the device reports something new — the device remains
  // the source of truth, the draft is only for the drag gesture.
  const [draft, setDraft] = useState<EqSettings>({ bass: 0, mid: 0, treble: 0 });
  const [dirty, setDirty] = useState(false);

  useEffect(() => {
    if (state && !dirty) {
      setDraft({ bass: state.bass, mid: state.mid, treble: state.treble });
    }
  }, [state, dirty]);

  const header = (
    <CapabilityBadge status={capability.status} hardwareVerified={capability.hardwareVerified} />
  );

  if (capability.status === 'unsupported') {
    return (
      <Panel title="Equalizer" actions={header}>
        <EmptyState title="Not available on this device" detail={capability.evidence} />
      </Panel>
    );
  }

  if (capability.status === 'unknown') {
    return (
      <Panel title="Equalizer" actions={header}>
        <EmptyState title="Not yet verified on this device" detail={capability.evidence} />
      </Panel>
    );
  }

  return (
    <Panel
      title="Equalizer"
      subtitle={
        capability.status !== 'verified'
          ? 'Not confirmed on your headphones. Changes are reported honestly.'
          : undefined
      }
      actions={header}
    >
      <div className="space-y-4">
        {BANDS.map(({ key, label }) => (
          <div key={key}>
            <div className="mb-1.5 flex items-baseline justify-between">
              <label htmlFor={`eq-${key}`} className="text-[12.5px] text-[var(--text-secondary)]">
                {label}
              </label>
              <span className="tabular-nums text-[12px] font-medium">
                {draft[key] > 0 ? '+' : ''}
                {draft[key]} dB
              </span>
            </div>
            <input
              id={`eq-${key}`}
              type="range"
              min={-10}
              max={10}
              step={1}
              value={draft[key]}
              disabled={!connected || pending !== null}
              onChange={(e) => {
                setDirty(true);
                setDraft((d) => ({ ...d, [key]: Number(e.target.value) }));
              }}
              className="w-full accent-[var(--color-accent-500)] disabled:opacity-45"
            />
          </div>
        ))}
      </div>

      <div className="mt-4 flex items-center gap-2">
        <Button
          variant="primary"
          disabled={!connected || !dirty}
          busy={pending === 'setEqualizer'}
          onClick={async () => {
            await run({ kind: 'setEqualizer', settings: draft });
            setDirty(false);
          }}
        >
          Apply
        </Button>
        <Button
          variant="ghost"
          disabled={!connected || pending !== null}
          onClick={() => {
            setDirty(true);
            setDraft({ bass: 0, mid: 0, treble: 0 });
          }}
        >
          Flat
        </Button>
        {dirty && (
          <span className="text-[11.5px] text-[var(--text-tertiary)]">Unsaved changes</span>
        )}
      </div>

      {state?.source === 'simulated' && (
        <p className="mt-3 text-[12px] text-[var(--text-tertiary)]">Values are simulated.</p>
      )}
    </Panel>
  );
}
