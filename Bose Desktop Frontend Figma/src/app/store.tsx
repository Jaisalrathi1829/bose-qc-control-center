import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { toast } from "sonner";

/* ============================================================
 * Types — the frontend state model. This mirrors the future
 * Tauri/Rust backend shape but is entirely mock data here.
 * ============================================================ */

export type ConnectionState =
  | "connected"
  | "disconnected"
  | "connecting"
  | "discovering"
  | "reconnecting"
  | "error"
  | "bluetooth-disabled"
  | "device-unavailable"
  | "simulated";

export type Capability =
  | "verified"
  | "supported"
  | "unknown"
  | "experimental"
  | "unsupported";

export type NoiseMode = "quiet" | "aware" | "custom";

export interface EqValues {
  bass: number;
  mid: number;
  treble: number;
}

export interface Profile {
  id: string;
  name: string;
  eq: EqValues;
  noise: NoiseMode;
  windBlock: boolean;
  lastUsed: string;
}

export interface DiagEvent {
  id: string;
  time: string;
  type: string;
  description: string;
  status: "info" | "user" | "success" | "warning";
}

export interface FeatureCapabilities {
  battery: Capability;
  volume: Capability;
  noiseControl: Capability;
  awareMode: Capability;
  eq: Capability;
  customAnc: Capability;
}

export type ThemePref = "system" | "light" | "dark";

interface AppState {
  connection: ConnectionState;
  setConnection: (c: ConnectionState) => void;
  battery: number;
  charging: boolean;
  deviceName: string;

  capabilities: FeatureCapabilities;

  noise: NoiseMode;
  setNoise: (n: NoiseMode) => void;
  ancLevel: number;
  setAncLevel: (n: number) => void;
  windBlock: boolean;
  setWindBlock: (b: boolean) => void;

  volume: number;
  setVolume: (n: number) => void;

  hardwareEq: EqValues;
  setHardwareEq: (e: EqValues) => void;
  softwareEq: EqValues;
  setSoftwareEq: (e: EqValues) => void;

  profiles: Profile[];
  addProfile: (p: Profile) => void;
  updateProfile: (p: Profile) => void;
  deleteProfile: (id: string) => void;
  applyProfile: (p: Profile) => void;

  events: DiagEvent[];
  capturing: boolean;
  toggleCapture: () => void;
  clearEvents: () => void;

  theme: ThemePref;
  setTheme: (t: ThemePref) => void;

  settings: Record<string, boolean>;
  toggleSetting: (k: string) => void;
  logLevel: string;
  setLogLevel: (s: string) => void;

  reconnect: () => void;
  disconnect: () => void;
}

const AppCtx = createContext<AppState | null>(null);

const now = () => {
  const d = new Date();
  return d.toLocaleTimeString([], { hour12: false });
};

const defaultProfiles: Profile[] = [
  { id: "music", name: "Music", eq: { bass: 4, mid: 0, treble: 2 }, noise: "quiet", windBlock: false, lastUsed: "2h ago" },
  { id: "gaming", name: "Gaming", eq: { bass: 6, mid: -2, treble: 4 }, noise: "aware", windBlock: false, lastUsed: "Yesterday" },
  { id: "study", name: "Study", eq: { bass: -2, mid: 1, treble: -1 }, noise: "quiet", windBlock: true, lastUsed: "3d ago" },
  { id: "podcast", name: "Podcast", eq: { bass: -3, mid: 5, treble: 1 }, noise: "aware", windBlock: false, lastUsed: "1w ago" },
];

const seedEvents: DiagEvent[] = [
  { id: "e1", time: "08:42:13", type: "NOTIFICATION", description: "Device notification received", status: "info" },
  { id: "e2", time: "08:42:15", type: "USER ACTION", description: "ANC_CHANGE — Quiet", status: "user" },
  { id: "e3", time: "08:42:15", type: "CHARACTERISTIC", description: "Characteristic updated", status: "success" },
  { id: "e4", time: "08:42:18", type: "STATE", description: "Device state changed", status: "info" },
];

export function AppProvider({ children }: { children: ReactNode }) {
  const [connection, setConnectionRaw] = useState<ConnectionState>("connected");
  const [battery] = useState(78);
  const [charging] = useState(false);
  const deviceName = "Bose QuietComfort";

  const capabilities: FeatureCapabilities = {
    battery: "verified",
    volume: "supported",
    noiseControl: "unknown",
    awareMode: "unknown",
    eq: "experimental",
    customAnc: "unknown",
  };

  const [noise, setNoiseRaw] = useState<NoiseMode>("quiet");
  const [ancLevel, setAncLevel] = useState(6);
  const [windBlock, setWindBlock] = useState(false);
  const [volume, setVolume] = useState(65);
  const [hardwareEq, setHardwareEq] = useState<EqValues>({ bass: 3, mid: 0, treble: 2 });
  const [softwareEq, setSoftwareEq] = useState<EqValues>({ bass: 0, mid: 0, treble: 0 });

  const [profiles, setProfiles] = useState<Profile[]>(defaultProfiles);
  const [events, setEvents] = useState<DiagEvent[]>(seedEvents);
  const [capturing, setCapturing] = useState(false);

  const [theme, setThemeRaw] = useState<ThemePref>("dark");
  const [settings, setSettings] = useState<Record<string, boolean>>({
    startWithWindows: true,
    startMinimized: false,
    minimizeToTray: true,
    autoConnect: true,
    notifyConnected: true,
    notifyDisconnected: true,
    notifyLowBattery: true,
    autoReconnect: true,
  });
  const [logLevel, setLogLevel] = useState("Info");

  /* ---- theme application ---- */
  useEffect(() => {
    const root = document.documentElement;
    const apply = () => {
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      const dark = theme === "dark" || (theme === "system" && prefersDark);
      root.classList.toggle("dark", dark);
    };
    apply();
    if (theme === "system") {
      const mq = window.matchMedia("(prefers-color-scheme: dark)");
      mq.addEventListener("change", apply);
      return () => mq.removeEventListener("change", apply);
    }
  }, [theme]);

  const pushEvent = useCallback((e: Omit<DiagEvent, "id" | "time">) => {
    setEvents((prev) => [{ id: crypto.randomUUID(), time: now(), ...e }, ...prev].slice(0, 60));
  }, []);

  const setConnection = useCallback((c: ConnectionState) => {
    setConnectionRaw(c);
    pushEvent({ type: "STATE", description: `Connection: ${c}`, status: "info" });
  }, [pushEvent]);

  const setNoise = useCallback((n: NoiseMode) => {
    setNoiseRaw(n);
    pushEvent({ type: "USER ACTION", description: `ANC_CHANGE — ${n}`, status: "user" });
    toast.success(`Noise Control changed to ${n[0].toUpperCase() + n.slice(1)}`, {
      description: "Command sent, but device state could not be verified.",
    });
  }, [pushEvent]);

  const reconnect = useCallback(() => {
    setConnectionRaw("reconnecting");
    pushEvent({ type: "STATE", description: "Reconnecting…", status: "warning" });
    setTimeout(() => {
      setConnectionRaw("connected");
      toast.success(`${deviceName} connected.`);
      pushEvent({ type: "STATE", description: "Reconnected", status: "success" });
    }, 1600);
  }, [pushEvent]);

  const disconnect = useCallback(() => {
    setConnectionRaw("disconnected");
    toast(`${deviceName} disconnected.`);
    pushEvent({ type: "STATE", description: "Disconnected", status: "warning" });
  }, [pushEvent]);

  const value: AppState = useMemo(
    () => ({
      connection,
      setConnection,
      battery,
      charging,
      deviceName,
      capabilities,
      noise,
      setNoise,
      ancLevel,
      setAncLevel,
      windBlock,
      setWindBlock,
      volume,
      setVolume,
      hardwareEq,
      setHardwareEq,
      softwareEq,
      setSoftwareEq,
      profiles,
      addProfile: (p) => { setProfiles((x) => [...x, p]); toast.success(`Profile “${p.name}” created.`); },
      updateProfile: (p) => { setProfiles((x) => x.map((i) => (i.id === p.id ? p : i))); toast.success(`Profile “${p.name}” saved.`); },
      deleteProfile: (id) => setProfiles((x) => x.filter((i) => i.id !== id)),
      applyProfile: (p) => {
        setNoiseRaw(p.noise);
        setHardwareEq(p.eq);
        setWindBlock(p.windBlock);
        setProfiles((x) => x.map((i) => (i.id === p.id ? { ...i, lastUsed: "Just now" } : i)));
        pushEvent({ type: "USER ACTION", description: `Applied profile ${p.name}`, status: "user" });
        toast.success(`Applied “${p.name}” profile.`);
      },
      events,
      capturing,
      toggleCapture: () => {
        setCapturing((c) => {
          const next = !c;
          pushEvent({ type: "SESSION", description: next ? "Discovery capture started" : "Discovery capture stopped", status: next ? "success" : "warning" });
          return next;
        });
      },
      clearEvents: () => setEvents([]),
      theme,
      setTheme: setThemeRaw,
      settings,
      toggleSetting: (k) => setSettings((s) => ({ ...s, [k]: !s[k] })),
      logLevel,
      setLogLevel,
      reconnect,
      disconnect,
    }),
    [connection, setConnection, battery, charging, noise, setNoise, ancLevel, windBlock, volume, hardwareEq, softwareEq, profiles, events, capturing, theme, settings, logLevel, reconnect, disconnect, pushEvent],
  );

  return <AppCtx.Provider value={value}>{children}</AppCtx.Provider>;
}

export function useApp() {
  const ctx = useContext(AppCtx);
  if (!ctx) throw new Error("useApp must be used within AppProvider");
  return ctx;
}

/* capability metadata helper */
export const capabilityMeta: Record<
  Capability,
  { label: string; tone: "success" | "info" | "neutral" | "warning" | "muted"; note: string }
> = {
  verified: { label: "Verified", tone: "success", note: "Confirmed working on your headphones." },
  supported: { label: "Supported", tone: "info", note: "Available through the detected device interface." },
  unknown: { label: "Unknown", tone: "neutral", note: "Not yet verified on your headphones." },
  experimental: { label: "Experimental", tone: "warning", note: "Not fully verified on this device." },
  unsupported: { label: "Unsupported", tone: "muted", note: "This feature is not exposed through the available device interface." },
};
