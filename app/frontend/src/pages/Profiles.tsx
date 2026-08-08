import { Panel, EmptyState } from '@/components/primitives';
import { useDeviceStore } from '@/stores/deviceStore';

/**
 * Local profiles.
 *
 * A profile bundles settings to apply together. Deliberately not implemented
 * as a working feature yet: a profile can only apply settings the device
 * actually supports, and right now the only verified capabilities are reads.
 * Shipping an "apply profile" button that silently did nothing would be
 * precisely the kind of fake control this project exists to avoid.
 */
export function Profiles() {
  const snapshot = useDeviceStore((s) => s.snapshot);

  const controllable = snapshot
    ? Object.values(snapshot.capabilities).filter(
        (c) => c.status === 'verified' && ['noiseControl', 'equalizer'].includes(c.key),
      ).length
    : 0;

  return (
    <div className="space-y-4">
      <Panel
        title="Profiles"
        subtitle="Bundles of settings applied together — for example Music, Gaming, Study or Podcast."
      >
        {controllable === 0 ? (
          <EmptyState
            title="No controllable settings yet"
            detail="A profile can only apply settings this device is confirmed to support. So far no writable capability has been verified on your headphones, so there is nothing a profile could change. This page will populate once Diagnostics confirms a controllable feature."
          />
        ) : (
          <EmptyState
            title="Profile editing not implemented"
            detail={`${controllable} controllable setting(s) are verified. Profile storage is the next piece of work.`}
          />
        )}
      </Panel>
    </div>
  );
}
