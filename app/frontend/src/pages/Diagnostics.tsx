import { useEffect } from 'react';
import { Panel, Button, EmptyState, Field } from '@/components/primitives';
import { CapabilityBadge } from '@/components/CapabilityBadge';
import { useDeviceStore } from '@/stores/deviceStore';
import { CAPABILITY_KEYS, type CapabilityKey } from '@/types/capability';
import { runtimeMode } from '@/services/ipc';

const LABEL: Record<CapabilityKey, string> = {
  battery: 'Battery',
  volume: 'Volume',
  playback: 'Playback',
  noiseControl: 'Noise Control',
  awareMode: 'Aware Mode',
  customNoiseControl: 'Custom ANC',
  equalizer: 'Equalizer',
  multipoint: 'Multipoint',
  deviceSettings: 'Device Info',
  firmwareVersion: 'Firmware Version',
  autoOff: 'Auto-Off',
  voicePrompts: 'Voice Prompts',
  sidetone: 'Sidetone',
  deviceRename: 'Device Rename',
};

/**
 * The primary tool for real-device integration.
 *
 * This page is the honest inventory: for every tracked feature it shows the
 * status, the mechanism it would use, whether it was confirmed on physical
 * hardware, and the evidence behind that judgement.
 */
export function Diagnostics() {
  const snapshot = useDeviceStore((s) => s.snapshot);
  const bluetooth = useDeviceStore((s) => s.bluetooth);
  const loadBluetooth = useDeviceStore((s) => s.loadBluetooth);
  const refresh = useDeviceStore((s) => s.refresh);
  const loading = useDeviceStore((s) => s.loading);

  useEffect(() => {
    void loadBluetooth();
  }, [loadBluetooth]);

  if (!snapshot) return <EmptyState title="Loading" detail="Reading device state…" />;

  const verified = snapshot.capabilities
    ? Object.values(snapshot.capabilities).filter((c) => c.hardwareVerified).length
    : 0;

  return (
    <div className="space-y-4">
      <Panel
        title="Capability Matrix"
        subtitle={`${verified} of ${CAPABILITY_KEYS.length} features confirmed against physical hardware.`}
        actions={
          <Button busy={loading} onClick={() => void refresh()}>
            Re-evaluate
          </Button>
        }
      >
        <div className="overflow-x-auto">
          <table className="w-full text-left text-[12.5px]">
            <thead>
              <tr className="border-b border-[var(--border-subtle)] text-[11px] uppercase tracking-wide text-[var(--text-tertiary)]">
                <th className="pb-2 pr-4 font-medium">Feature</th>
                <th className="pb-2 pr-4 font-medium">Status</th>
                <th className="pb-2 pr-4 font-medium">Mechanism</th>
                <th className="pb-2 font-medium">Evidence</th>
              </tr>
            </thead>
            <tbody>
              {CAPABILITY_KEYS.map((key) => {
                const cap = snapshot.capabilities[key];
                return (
                  <tr key={key} className="border-b border-[var(--border-subtle)] last:border-0">
                    <td className="py-2.5 pr-4 font-medium">{LABEL[key]}</td>
                    <td className="py-2.5 pr-4">
                      <CapabilityBadge
                        status={cap.status}
                        hardwareVerified={cap.hardwareVerified}
                        size="sm"
                      />
                    </td>
                    <td className="py-2.5 pr-4 text-[var(--text-tertiary)]">
                      {cap.mechanism === 'none' ? '—' : cap.mechanism}
                    </td>
                    <td className="py-2.5 text-[11.5px] leading-snug text-[var(--text-secondary)]">
                      {cap.evidence}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </Panel>

      <Panel title="Environment">
        <dl>
          <Field
            label="Runtime"
            value={runtimeMode() === 'native' ? 'Desktop application' : 'Browser preview'}
          />
          <Field label="Backend" value={snapshot.source === 'mock' ? 'Simulated' : 'Real hardware'} />
          <Field label="Transport" value={snapshot.transport} />
          <Field
            label="Bluetooth radio"
            value={
              bluetooth
                ? bluetooth.radioPresent
                  ? 'Present'
                  : 'Not present'
                : 'Checking…'
            }
          />
        </dl>
        {bluetooth && (
          <p className="mt-3 text-[11.5px] leading-relaxed text-[var(--text-tertiary)]">
            {bluetooth.detail}
          </p>
        )}
      </Panel>

      <Panel
        title="Capture"
        subtitle="Record device events while you physically operate the headphones, to discover which actions the device reports."
      >
        <EmptyState
          title="Capture not available in this build"
          detail="The passive event-capture harness ships with the discovery tool. Until a Bose device has been interrogated, there is no event stream to record."
        />
      </Panel>
    </div>
  );
}
