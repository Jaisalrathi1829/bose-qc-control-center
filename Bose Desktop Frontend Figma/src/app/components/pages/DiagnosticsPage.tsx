import { useState } from "react";
import { CircleStop, Download, Play, Trash2 } from "lucide-react";
import {
  Button,
  CapabilityBadge,
  ConnectionBadge,
  Panel,
  SectionLabel,
} from "../primitives";
import { Modal } from "../Modal";
import { useApp, type Capability } from "../../store";
import { cn } from "../ui/utils";

const sections = ["Bluetooth", "BLE", "Services", "Characteristics", "Battery", "Audio", "Capabilities"];

const eventTone: Record<string, string> = {
  info: "bg-info",
  user: "bg-chart-4",
  success: "bg-success",
  warning: "bg-warning",
};

export function DiagnosticsPage() {
  const { deviceName, connection, capabilities, events, capturing, toggleCapture, clearEvents } = useApp();
  const [exportOpen, setExportOpen] = useState(false);

  const capRows: { feature: string; cap: Capability }[] = [
    { feature: "Battery", cap: capabilities.battery },
    { feature: "Volume", cap: capabilities.volume },
    { feature: "Noise Control", cap: capabilities.noiseControl },
    { feature: "Aware Mode", cap: capabilities.awareMode },
    { feature: "EQ", cap: capabilities.eq },
    { feature: "Custom ANC", cap: capabilities.customAnc },
  ];

  return (
    <div className="grid grid-cols-1 gap-5 lg:grid-cols-3">
      <div className="flex flex-col gap-5 lg:col-span-2">
        <Panel className="p-6">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <SectionLabel>Session</SectionLabel>
              <div className="mt-1 flex items-center gap-2">
                <span className="text-sm">{deviceName}</span>
                <ConnectionBadge state={connection} />
              </div>
            </div>
            <div className="flex gap-2">
              {capturing ? (
                <Button variant="danger" icon={<CircleStop className="size-4" />} onClick={toggleCapture}>Stop Capture</Button>
              ) : (
                <Button icon={<Play className="size-4" />} onClick={toggleCapture}>Start Capture</Button>
              )}
              <Button variant="outline" icon={<Download className="size-4" />} onClick={() => setExportOpen(true)}>Export Report</Button>
            </div>
          </div>
          <div className="mt-4 flex flex-wrap gap-1.5">
            {sections.map((s) => (
              <span key={s} className="rounded-md border border-border bg-surface-2 px-2 py-0.5 text-xs text-muted-foreground">{s}</span>
            ))}
          </div>
        </Panel>

        {/* Discovery session timeline */}
        <Panel className="p-6">
          <div className="mb-4 flex items-center justify-between">
            <SectionLabel>Discovery Session</SectionLabel>
            {events.length > 0 && (
              <button onClick={clearEvents} className="inline-flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground">
                <Trash2 className="size-3.5" /> Clear
              </button>
            )}
          </div>
          {events.length === 0 ? (
            <div className="py-10 text-center">
              <p className="text-sm text-muted-foreground">No diagnostic sessions yet.</p>
              <Button className="mt-3" size="sm" icon={<Play className="size-4" />} onClick={toggleCapture}>Start Discovery</Button>
            </div>
          ) : (
            <ol className="space-y-0">
              {events.map((e) => (
                <li key={e.id} className="flex items-start gap-3 border-b border-border py-2.5 last:border-0">
                  <span className="mt-1.5 tabular-nums text-xs text-muted-foreground">{e.time}</span>
                  <span className={cn("mt-1.5 size-2 shrink-0 rounded-full", eventTone[e.status])} />
                  <div className="min-w-0 flex-1">
                    <div className="text-xs font-medium uppercase tracking-wide text-muted-foreground">{e.type}</div>
                    <div className="text-sm">{e.description}</div>
                  </div>
                </li>
              ))}
            </ol>
          )}
        </Panel>
      </div>

      {/* Capability table */}
      <Panel className="h-fit p-6">
        <SectionLabel>Capabilities</SectionLabel>
        <table className="mt-3 w-full">
          <thead>
            <tr className="text-left text-xs text-muted-foreground">
              <th className="pb-2 font-normal">Feature</th>
              <th className="pb-2 text-right font-normal">Status</th>
            </tr>
          </thead>
          <tbody>
            {capRows.map((r) => (
              <tr key={r.feature} className="border-t border-border">
                <td className="py-2.5 text-sm">{r.feature}</td>
                <td className="py-2.5 text-right"><CapabilityBadge cap={r.cap} /></td>
              </tr>
            ))}
          </tbody>
        </table>
      </Panel>

      <Modal
        open={exportOpen}
        onClose={() => setExportOpen(false)}
        title="Export Diagnostics Report"
        description="Save the current session as a local report file."
        footer={
          <>
            <Button variant="outline" onClick={() => setExportOpen(false)}>Cancel</Button>
            <Button icon={<Download className="size-4" />} onClick={() => setExportOpen(false)}>Export</Button>
          </>
        }
      >
        <div className="space-y-2 text-sm text-muted-foreground">
          <p>The report will include capability detection results, {events.length} captured events, and device metadata.</p>
          <p className="rounded-lg border border-border bg-warning-subtle/40 p-3 text-xs">
            Note: Values are collected from a mock device interface and do not represent verified hardware behavior.
          </p>
        </div>
      </Modal>
    </div>
  );
}
