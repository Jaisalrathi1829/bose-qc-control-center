import { Panel, Button, EmptyState, Field, StatusDot } from '@/components/primitives';
import { useDeviceStore } from '@/stores/deviceStore';

export function DevicePage() {
  const snapshot = useDeviceStore((s) => s.snapshot);
  const run = useDeviceStore((s) => s.run);
  const pending = useDeviceStore((s) => s.pending);

  if (!snapshot) return <EmptyState title="Loading" detail="Reading device state…" />;

  const id = snapshot.identity;
  const connected = snapshot.connection === 'connected';

  return (
    <div className="space-y-4">
      <Panel title="Device">
        {id ? (
          <dl>
            <Field label="Name" value={id.name} />
            <Field
              label="Status"
              value={
                <span className="inline-flex items-center gap-1.5">
                  <StatusDot state={connected ? 'on' : 'off'} />
                  <span className="capitalize">{snapshot.connection}</span>
                </span>
              }
            />
            <Field
              label="Battery"
              value={snapshot.battery ? `${snapshot.battery.percent}%` : 'Not reported'}
            />
            <Field label="Transport" value={snapshot.transport} />
            <Field label="Device ID" value={<code className="text-[11.5px]">{id.id}</code>} />
            <Field label="Manufacturer" value={id.manufacturer ?? 'Not reported'} />
            <Field label="Model" value={id.modelNumber ?? 'Not reported'} />
            <Field label="Firmware" value={id.firmwareVersion ?? 'Not exposed to Windows'} />
          </dl>
        ) : (
          <EmptyState
            title="No device"
            detail="Nothing is connected, so there is no device information to show."
          />
        )}

        <p className="mt-4 text-[11.5px] leading-relaxed text-[var(--text-tertiary)]">
          The device ID shown is a salted hash. Your headphones' Bluetooth address never leaves the
          native layer and is not included in exported reports.
        </p>
      </Panel>

      <Panel title="Connection">
        <div className="flex flex-wrap gap-2">
          <Button
            variant="primary"
            busy={pending === 'reconnect'}
            onClick={() => void run({ kind: 'reconnect' })}
          >
            Reconnect
          </Button>
          <Button
            disabled={!connected}
            busy={pending === 'disconnect'}
            onClick={() => void run({ kind: 'disconnect' })}
          >
            Disconnect
          </Button>
        </div>
      </Panel>
    </div>
  );
}
