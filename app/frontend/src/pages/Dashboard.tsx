import { Panel, Button, EmptyState, StatusDot } from '@/components/primitives';
import { CapabilityBadge } from '@/components/CapabilityBadge';
import { NoiseControlCard } from '@/components/NoiseControlCard';
import { EqualizerCard } from '@/components/EqualizerCard';
import { useDeviceStore } from '@/stores/deviceStore';
import { useUiStore } from '@/stores/uiStore';

function BatteryRing({ percent }: { percent: number }) {
  const r = 34;
  const c = 2 * Math.PI * r;
  const low = percent <= 20;
  return (
    <div className="relative grid h-24 w-24 place-items-center">
      <svg viewBox="0 0 80 80" className="h-24 w-24 -rotate-90">
        <circle cx="40" cy="40" r={r} fill="none" stroke="var(--border-subtle)" strokeWidth="6" />
        <circle
          cx="40"
          cy="40"
          r={r}
          fill="none"
          stroke={low ? 'var(--color-status-unsupported)' : 'var(--color-accent-500)'}
          strokeWidth="6"
          strokeLinecap="round"
          strokeDasharray={c}
          strokeDashoffset={c - (c * percent) / 100}
          className="transition-[stroke-dashoffset] duration-500"
        />
      </svg>
      <div className="absolute text-center">
        <div className="text-[22px] font-semibold tabular-nums leading-none">{percent}</div>
        <div className="text-[10px] text-[var(--text-tertiary)]">percent</div>
      </div>
    </div>
  );
}

export function Dashboard() {
  const snapshot = useDeviceStore((s) => s.snapshot);
  const run = useDeviceStore((s) => s.run);
  const pending = useDeviceStore((s) => s.pending);
  const setPage = useUiStore((s) => s.setPage);

  if (!snapshot) {
    return <EmptyState title="Loading" detail="Reading device state…" />;
  }

  const connected = snapshot.connection === 'connected';
  const caps = snapshot.capabilities;

  return (
    <div className="space-y-4">
      {/* Connection + battery */}
      <Panel>
        <div className="flex flex-wrap items-center justify-between gap-6">
          <div className="flex items-center gap-5">
            {snapshot.battery ? (
              <BatteryRing percent={snapshot.battery.percent} />
            ) : (
              <div className="grid h-24 w-24 place-items-center rounded-full border border-dashed border-[var(--border-strong)]">
                <span className="px-2 text-center text-[10.5px] leading-tight text-[var(--text-tertiary)]">
                  No battery reading
                </span>
              </div>
            )}
            <div>
              <div className="flex items-center gap-2">
                <StatusDot state={connected ? 'on' : 'off'} />
                <span className="text-[12.5px] capitalize text-[var(--text-secondary)]">
                  {snapshot.connection}
                </span>
              </div>
              <h1 className="mt-1 font-[var(--font-display)] text-[20px] font-semibold tracking-tight">
                {snapshot.identity?.name ?? 'No device'}
              </h1>
              {snapshot.battery ? (
                <p className="mt-1 text-[12px] text-[var(--text-tertiary)]">
                  Battery via{' '}
                  {snapshot.battery.source === 'windows-pnp'
                    ? 'Windows'
                    : snapshot.battery.source === 'simulated'
                      ? 'simulation'
                      : snapshot.battery.source}
                </p>
              ) : (
                connected && (
                  <p className="mt-1 max-w-sm text-[12px] leading-snug text-[var(--text-tertiary)]">
                    This device has not reported a battery level to Windows.
                  </p>
                )
              )}
            </div>
          </div>

          <div className="flex gap-2">
            {connected ? (
              <Button
                onClick={() => void run({ kind: 'disconnect' })}
                busy={pending === 'disconnect'}
              >
                Disconnect
              </Button>
            ) : (
              <Button
                variant="primary"
                onClick={() => void run({ kind: 'connect' })}
                busy={pending === 'connect'}
              >
                Connect
              </Button>
            )}
            <Button variant="ghost" onClick={() => setPage('diagnostics')}>
              Diagnostics
            </Button>
          </div>
        </div>
      </Panel>

      {!connected && (
        <EmptyState
          title="Nothing is connected"
          detail="Connect a device to see what it actually supports. Controls stay hidden until the device tells us they exist."
        />
      )}

      {connected && (
        <div className="grid gap-4 lg:grid-cols-2">
          <NoiseControlCard
            capability={caps.noiseControl}
            state={snapshot.noiseControl}
            connected={connected}
          />
          <EqualizerCard
            capability={caps.equalizer}
            state={snapshot.equalizer}
            connected={connected}
          />
        </div>
      )}

      {connected && (
        <Panel
          title="Windows Volume"
          subtitle="System volume for the audio endpoint. This is separate from any volume the headphones keep internally."
          actions={<CapabilityBadge status={caps.volume.status} hardwareVerified={caps.volume.hardwareVerified} />}
        >
          {snapshot.windowsAudio ? (
            <div>
              <div className="mb-2 flex items-baseline justify-between">
                <span className="text-[12.5px] text-[var(--text-secondary)]">
                  {snapshot.windowsAudio.endpointName}
                </span>
                <span className="tabular-nums text-[13px] font-medium">
                  {snapshot.windowsAudio.volumePercent}%
                </span>
              </div>
              <input
                type="range"
                min={0}
                max={100}
                value={snapshot.windowsAudio.volumePercent}
                onChange={(e) =>
                  void run({ kind: 'setSystemVolume', percent: Number(e.target.value) })
                }
                className="w-full accent-[var(--color-accent-500)]"
                aria-label="Windows system volume"
              />
            </div>
          ) : (
            <EmptyState
              title="Windows audio not wired up yet"
              detail="The Core Audio integration is not connected in this build, so no endpoint is shown."
            />
          )}
        </Panel>
      )}
    </div>
  );
}
