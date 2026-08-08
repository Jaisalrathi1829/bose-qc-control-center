import { useState } from "react";
import {
  Activity,
  Bluetooth,
  Eye,
  EyeOff,
  Power,
  RefreshCw,
} from "lucide-react";
import {
  BatteryIndicator,
  Button,
  ConnectionBadge,
  Panel,
  SectionLabel,
} from "../primitives";
import { DeviceVisual } from "../DeviceVisual";
import { useApp } from "../../store";
import { cn } from "../ui/utils";

const timeline = [
  { time: "08:42:18", label: "Reconnected", tone: "success" },
  { time: "08:40:02", label: "Disconnected", tone: "warning" },
  { time: "08:12:47", label: "Connected", tone: "success" },
];

function InfoRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between border-b border-border py-3 last:border-0">
      <span className="text-sm text-muted-foreground">{label}</span>
      <span className="text-sm">{children}</span>
    </div>
  );
}

export function DevicePage({ onNavigate }: { onNavigate: (p: any) => void }) {
  const { deviceName, connection, battery, charging, reconnect, disconnect } = useApp();
  const [showId, setShowId] = useState(false);

  return (
    <div className="grid grid-cols-1 gap-5 lg:grid-cols-3">
      <div className="flex flex-col gap-5 lg:col-span-2">
        <Panel className="flex items-center gap-6 p-6">
          <DeviceVisual size={180} active={connection === "connected" || connection === "simulated"} />
          <div className="flex-1">
            <div className="flex items-center gap-3">
              <h2>{deviceName}</h2>
              <ConnectionBadge state={connection} />
            </div>
            <p className="mt-1 text-sm text-muted-foreground">Over-ear • Bluetooth 5.3 • Adaptive ANC</p>
            <div className="mt-4 flex flex-wrap gap-2">
              <Button icon={<RefreshCw className="size-4" />} onClick={reconnect}>Reconnect</Button>
              <Button variant="outline" icon={<Power className="size-4" />} onClick={disconnect}>Disconnect</Button>
              <Button variant="ghost" icon={<Activity className="size-4" />} onClick={() => onNavigate("diagnostics")}>
                Run Diagnostics
              </Button>
            </div>
          </div>
        </Panel>

        <Panel className="p-6">
          <SectionLabel>Device Information</SectionLabel>
          <div className="mt-3">
            <InfoRow label="Connection"><ConnectionBadge state={connection} /></InfoRow>
            <InfoRow label="Transport">
              <span className="inline-flex items-center gap-1.5"><Bluetooth className="size-4 text-primary" /> Bluetooth</span>
            </InfoRow>
            <InfoRow label="Battery"><BatteryIndicator level={battery} charging={charging} /></InfoRow>
            <InfoRow label="Device name">{deviceName}</InfoRow>
            <InfoRow label="Device ID">
              <span className="inline-flex items-center gap-2 font-mono">
                {showId ? "QC-8F3A-11D7" : "••••••••"}
                <button onClick={() => setShowId((s) => !s)} className="text-muted-foreground hover:text-foreground">
                  {showId ? <EyeOff className="size-3.5" /> : <Eye className="size-3.5" />}
                </button>
              </span>
            </InfoRow>
            <InfoRow label="Firmware"><span className="text-muted-foreground">Unavailable</span></InfoRow>
          </div>
        </Panel>
      </div>

      <Panel className="p-6">
        <SectionLabel>Connection Timeline</SectionLabel>
        <ol className="mt-4 space-y-0">
          {timeline.map((e, i) => (
            <li key={i} className="relative flex gap-3 pb-6 last:pb-0">
              <div className="flex flex-col items-center">
                <span className={cn("mt-1 size-2.5 rounded-full",
                  e.tone === "success" ? "bg-success" : "bg-warning")} />
                {i < timeline.length - 1 && <span className="w-px flex-1 bg-border" />}
              </div>
              <div className="-mt-0.5">
                <div className="text-sm">{e.label}</div>
                <div className="text-xs tabular-nums text-muted-foreground">{e.time}</div>
              </div>
            </li>
          ))}
        </ol>
      </Panel>
    </div>
  );
}
