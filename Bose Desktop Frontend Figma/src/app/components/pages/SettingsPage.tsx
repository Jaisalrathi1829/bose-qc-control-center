import { useState } from "react";
import { Monitor, Moon, Sun, Trash2, Download } from "lucide-react";
import {
  Button,
  Panel,
  SectionLabel,
  Toggle,
} from "../primitives";
import { Modal } from "../Modal";
import { useApp, type ThemePref } from "../../store";
import { cn } from "../ui/utils";

function SettingRow({
  title,
  desc,
  children,
}: {
  title: string;
  desc?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between gap-4 border-b border-border py-3.5 last:border-0">
      <div>
        <div className="text-sm">{title}</div>
        {desc && <div className="text-xs text-muted-foreground">{desc}</div>}
      </div>
      {children}
    </div>
  );
}

const themes: { value: ThemePref; label: string; icon: React.ElementType }[] = [
  { value: "system", label: "System", icon: Monitor },
  { value: "light", label: "Light", icon: Sun },
  { value: "dark", label: "Dark", icon: Moon },
];

const logLevels = ["Off", "Error", "Warn", "Info", "Debug", "Trace"];

export function SettingsPage() {
  const { settings, toggleSetting, theme, setTheme, logLevel, setLogLevel, deviceName } = useApp();
  const [clearOpen, setClearOpen] = useState(false);

  return (
    <div className="mx-auto max-w-2xl space-y-5">
      <Panel className="p-6">
        <SectionLabel>General</SectionLabel>
        <div className="mt-2">
          <SettingRow title="Start with Windows"><Toggle checked={settings.startWithWindows} onChange={() => toggleSetting("startWithWindows")} /></SettingRow>
          <SettingRow title="Start minimized"><Toggle checked={settings.startMinimized} onChange={() => toggleSetting("startMinimized")} /></SettingRow>
          <SettingRow title="Minimize to tray"><Toggle checked={settings.minimizeToTray} onChange={() => toggleSetting("minimizeToTray")} /></SettingRow>
          <SettingRow title="Auto-connect" desc="Connect to preferred device on launch"><Toggle checked={settings.autoConnect} onChange={() => toggleSetting("autoConnect")} /></SettingRow>
        </div>
      </Panel>

      <Panel className="p-6">
        <SectionLabel>Appearance</SectionLabel>
        <div className="mt-3 grid grid-cols-3 gap-2">
          {themes.map((t) => {
            const Icon = t.icon;
            const active = theme === t.value;
            return (
              <button
                key={t.value}
                onClick={() => setTheme(t.value)}
                className={cn(
                  "flex flex-col items-center gap-2 rounded-lg border p-4 text-sm outline-none transition-all focus-visible:ring-2 focus-visible:ring-ring",
                  active ? "border-primary bg-accent text-accent-foreground" : "border-border text-muted-foreground hover:bg-accent",
                )}
              >
                <Icon className="size-5" />
                {t.label}
              </button>
            );
          })}
        </div>
      </Panel>

      <Panel className="p-6">
        <SectionLabel>Device</SectionLabel>
        <div className="mt-2">
          <SettingRow title="Preferred Device" desc="Reconnect to this device automatically">
            <span className="text-sm text-muted-foreground">{deviceName}</span>
          </SettingRow>
          <SettingRow title="Auto reconnect" desc="Reconnect automatically when in range"><Toggle checked={settings.autoReconnect} onChange={() => toggleSetting("autoReconnect")} /></SettingRow>
        </div>
      </Panel>

      <Panel className="p-6">
        <SectionLabel>Notifications</SectionLabel>
        <div className="mt-2">
          <SettingRow title="Device connected"><Toggle checked={settings.notifyConnected} onChange={() => toggleSetting("notifyConnected")} /></SettingRow>
          <SettingRow title="Device disconnected"><Toggle checked={settings.notifyDisconnected} onChange={() => toggleSetting("notifyDisconnected")} /></SettingRow>
          <SettingRow title="Low battery"><Toggle checked={settings.notifyLowBattery} onChange={() => toggleSetting("notifyLowBattery")} /></SettingRow>
        </div>
      </Panel>

      <Panel className="p-6">
        <SectionLabel>Diagnostics</SectionLabel>
        <div className="mt-3">
          <label className="mb-2 block">Logging level</label>
          <div className="flex flex-wrap gap-1.5">
            {logLevels.map((l) => (
              <button
                key={l}
                onClick={() => setLogLevel(l)}
                className={cn(
                  "rounded-md border px-3 py-1 text-sm outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring",
                  logLevel === l ? "border-primary/50 bg-accent text-accent-foreground" : "border-border text-muted-foreground hover:bg-accent",
                )}
              >
                {l}
              </button>
            ))}
          </div>
          <div className="mt-4 flex gap-2">
            <Button variant="outline" size="sm" icon={<Download className="size-4" />}>Export Logs</Button>
            <Button variant="ghost" size="sm" icon={<Trash2 className="size-4" />} onClick={() => setClearOpen(true)}>Clear Logs</Button>
          </div>
        </div>
      </Panel>

      <Modal
        open={clearOpen}
        onClose={() => setClearOpen(false)}
        title="Clear Logs"
        description="This will permanently remove all locally stored diagnostic logs."
        footer={
          <>
            <Button variant="outline" onClick={() => setClearOpen(false)}>Cancel</Button>
            <Button variant="danger" onClick={() => setClearOpen(false)}>Clear Logs</Button>
          </>
        }
      />
    </div>
  );
}
