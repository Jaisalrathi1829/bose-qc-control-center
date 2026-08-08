import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { toast } from "sonner";
import * as ipc from "@/services/ipc";
import type { CommandOutcome, DeviceSnapshot } from "@/types/device";
import type { CapabilityStatus } from "@/types/capability";

/* ============================================================
 * Application state, backed by the Rust native layer.
 *
 * This preserves the API the Figma pages were written against, but every
 * value now comes from the backend instead of being hardcoded. Where the
 * backend cannot yet do something, this module says so plainly rather than
 * simulating success — see `docs/capability-matrix.md`.
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

/** Same five states the Rust capability engine uses. */
export type Capability = CapabilityStatus;

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

  /**
   * True when the active backend is the mock. Deliberately separate from
   * `connection`: a simulated device can be connected, disconnected or
   * reconnecting, and the SIMULATED warning must show in every one of those
   * cases. Folding it into the connection state would hide the warning
   * exactly when the UI looks most like real hardware.
   */
  simulated: boolean;
  /** Switches between the mock and real-hardware backends. */
  setSimulated: (s: boolean) => Promise<void>;
  /** False in a plain browser preview, where no native layer exists. */
  nativeAvailable: boolean;

  battery: number | null;
  charging: boolean;
  deviceName: string;
  /** Provenance of the battery reading, shown so the claim is auditable. */
  batterySource: string | null;

  capabilities: FeatureCapabilities;
  /** Full per-capability detail, including the evidence behind each status. */
  snapshot: DeviceSnapshot | null;

  noise: NoiseMode | null;
  setNoise: (n: NoiseMode) => void;
  ancLevel: number;
  setAncLevel: (n: number) => void;
  windBlock: boolean;
  setWindBlock: (b: boolean) => void;

  volume: number;
  setVolume: (n: number) => void;
  muted: boolean;
  setMuted: (m: boolean) => void;
  /** Windows endpoint the volume applies to, or null if none is active. */
  audioEndpointName: string | null;

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

  connect: () => void;
  reconnect: () => void;
  disconnect: () => void;
  refresh: () => Promise<void>;
  busy: string | null;
}

const AppCtx = createContext<AppState | null>(null);

const now = () => new Date().toLocaleTimeString([], { hour12: false });

const THEME_KEY = "bose-qc.theme";
const PROFILES_KEY = "bose-qc.profiles";

/**
 * Settings whose backing implementation does not exist yet.
 *
 * The Settings page reads this to disable the control and explain why, rather
 * than presenting a switch that silently does nothing.
 */
export const UNIMPLEMENTED_SETTINGS = new Set([
  "startWithWindows",
  "startMinimized",
  "minimizeToTray",
  "autoConnect",
  "autoReconnect",
  "notifyConnected",
  "notifyDisconnected",
  "notifyLowBattery",
]);

function loadTheme(): ThemePref {
  if (typeof localStorage === "undefined") return "system";
  const raw = localStorage.getItem(THEME_KEY);
  return raw === "light" || raw === "dark" || raw === "system" ? raw : "system";
}

function loadProfiles(): Profile[] {
  if (typeof localStorage === "undefined") return [];
  try {
    const raw = localStorage.getItem(PROFILES_KEY);
    return raw ? (JSON.parse(raw) as Profile[]) : [];
  } catch {
    return [];
  }
}

/** Maps the backend's 14 tracked capabilities onto the 6 the UI renders. */
function mapCapabilities(snapshot: DeviceSnapshot | null): FeatureCapabilities {
  const unknown: Capability = "unknown";
  if (!snapshot) {
    return {
      battery: unknown,
      volume: unknown,
      noiseControl: unknown,
      awareMode: unknown,
      eq: unknown,
      customAnc: unknown,
    };
  }
  const c = snapshot.capabilities;
  return {
    battery: c.battery.status,
    volume: c.volume.status,
    noiseControl: c.noiseControl.status,
    awareMode: c.awareMode.status,
    eq: c.equalizer.status,
    customAnc: c.customNoiseControl.status,
  };
}

function connectionFrom(snapshot: DeviceSnapshot | null): ConnectionState {
  if (!snapshot) return "disconnected";
  switch (snapshot.connection) {
    case "connected":
      return "connected";
    case "connecting":
      return "connecting";
    case "discovering":
      return "discovering";
    case "reconnecting":
      return "reconnecting";
    case "error":
      return "error";
    default:
      return "disconnected";
  }
}

export function AppProvider({ children }: { children: ReactNode }) {
  const [snapshot, setSnapshot] = useState<DeviceSnapshot | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [events, setEvents] = useState<DiagEvent[]>([]);
  const [capturing, setCapturing] = useState(false);
  const capturingRef = useRef(false);

  const [ancLevel, setAncLevelRaw] = useState(6);
  const [windBlock, setWindBlockRaw] = useState(false);
  const [softwareEq, setSoftwareEqRaw] = useState<EqValues>({ bass: 0, mid: 0, treble: 0 });

  const [profiles, setProfiles] = useState<Profile[]>(loadProfiles);
  const [theme, setThemeRaw] = useState<ThemePref>(loadTheme);
  const [settings, setSettings] = useState<Record<string, boolean>>({
    startWithWindows: false,
    startMinimized: false,
    minimizeToTray: false,
    autoConnect: false,
    notifyConnected: false,
    notifyDisconnected: false,
    notifyLowBattery: false,
    autoReconnect: false,
  });
  const [logLevel, setLogLevel] = useState("Info");

  const nativeAvailable = ipc.hasNativeLayer();

  /* ---- theme ---- */
  useEffect(() => {
    const root = document.documentElement;
    const apply = () => {
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      root.classList.toggle("dark", theme === "dark" || (theme === "system" && prefersDark));
    };
    apply();
    if (theme === "system") {
      const mq = window.matchMedia("(prefers-color-scheme: dark)");
      mq.addEventListener("change", apply);
      return () => mq.removeEventListener("change", apply);
    }
  }, [theme]);

  const setTheme = useCallback((t: ThemePref) => {
    if (typeof localStorage !== "undefined") localStorage.setItem(THEME_KEY, t);
    setThemeRaw(t);
  }, []);

  /* ---- events ---- */
  const pushEvent = useCallback((e: Omit<DiagEvent, "id" | "time">) => {
    setEvents((prev) =>
      [
        {
          id:
            typeof crypto !== "undefined" && crypto.randomUUID
              ? crypto.randomUUID()
              : String(Math.random()),
          time: now(),
          ...e,
        },
        ...prev,
      ].slice(0, 200),
    );
  }, []);

  /* ---- snapshot ---- */
  const refresh = useCallback(async () => {
    try {
      const s = await ipc.getSnapshot();
      setSnapshot(s);
    } catch (e) {
      const err = ipc.toIpcError(e);
      pushEvent({ type: "ERROR", description: err.message, status: "warning" });
    }
  }, [pushEvent]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  /* ---- command dispatch ----
   *
   * Every mutating action goes through here. The outcome from the backend
   * decides what the user is told: `applied` is the only success. A command
   * that was transmitted but not confirmed produces a warning, never a
   * success toast — the device not echoing its state back is exactly the
   * situation the user needs to know about.
   */
  const run = useCallback(
    async (
      command: ipc.DeviceCommand,
      describe: { label: string; eventType: string },
    ): Promise<CommandOutcome | null> => {
      setBusy(command.kind);
      try {
        const outcome = await ipc.executeCommand(command);

        switch (outcome.kind) {
          case "applied":
            toast.success(`${describe.label} applied.`, {
              description: "Confirmed by the device.",
            });
            pushEvent({
              type: describe.eventType,
              description: `${describe.label} — confirmed`,
              status: "success",
            });
            break;
          case "sent-unverified":
            toast.warning(`${describe.label}: state could not be verified.`, {
              description: outcome.reason,
            });
            pushEvent({
              type: describe.eventType,
              description: `${describe.label} — sent, unverified`,
              status: "warning",
            });
            break;
          case "rejected":
            toast.error(`${describe.label} rejected.`, { description: outcome.reason });
            pushEvent({
              type: describe.eventType,
              description: `${describe.label} — rejected`,
              status: "warning",
            });
            break;
          case "unsupported":
            toast.warning(`${describe.label} is not available.`, {
              description: outcome.reason,
            });
            pushEvent({
              type: describe.eventType,
              description: `${describe.label} — unsupported`,
              status: "warning",
            });
            break;
        }

        // Never assume the command took effect: re-read and render what the
        // device actually reports.
        await refresh();
        return outcome;
      } catch (e) {
        const err = ipc.toIpcError(e);
        toast.error(`${describe.label} failed.`, { description: err.message });
        pushEvent({
          type: "ERROR",
          description: `${describe.label} — ${err.message}`,
          status: "warning",
        });
        return null;
      } finally {
        setBusy(null);
      }
    },
    [pushEvent, refresh],
  );

  /* ---- derived ---- */
  const connection = connectionFrom(snapshot);
  const simulated = snapshot?.source === "mock";
  const capabilities = useMemo(() => mapCapabilities(snapshot), [snapshot]);

  const battery = snapshot?.battery?.percent ?? null;
  const charging = snapshot?.battery?.charging ?? false;
  const batterySource = snapshot?.battery?.source ?? null;
  const deviceName = snapshot?.identity?.name ?? "No device";

  const noise = (snapshot?.noiseControl?.mode ?? null) as NoiseMode | null;
  const hardwareEq: EqValues = snapshot?.equalizer
    ? {
        bass: snapshot.equalizer.bass,
        mid: snapshot.equalizer.mid,
        treble: snapshot.equalizer.treble,
      }
    : { bass: 0, mid: 0, treble: 0 };

  const volume = snapshot?.windowsAudio?.volumePercent ?? 0;
  const muted = snapshot?.windowsAudio?.muted ?? false;
  const audioEndpointName = snapshot?.windowsAudio?.endpointName ?? null;

  /* ---- actions ---- */
  const setNoise = useCallback(
    (n: NoiseMode) => {
      void run(
        { kind: "setNoiseControl", mode: n },
        { label: `Noise control (${n})`, eventType: "ANC_CHANGE" },
      );
    },
    [run],
  );

  const setHardwareEq = useCallback(
    (e: EqValues) => {
      void run({ kind: "setEqualizer", settings: e }, { label: "Equalizer", eventType: "EQ" });
    },
    [run],
  );

  const setVolume = useCallback(
    (n: number) => {
      void run(
        { kind: "setSystemVolume", percent: Math.round(n) },
        { label: "Windows volume", eventType: "VOLUME" },
      );
    },
    [run],
  );

  const setMuted = useCallback(
    (m: boolean) => {
      void run({ kind: "setSystemMute", muted: m }, { label: "Mute", eventType: "VOLUME" });
    },
    [run],
  );

  const setAncLevel = useCallback(
    (n: number) => {
      setAncLevelRaw(n);
      void run(
        { kind: "setNoiseControlLevel", level: Math.round(n) },
        { label: "Noise control level", eventType: "ANC_LEVEL" },
      );
    },
    [run],
  );

  // No backend command exists for wind block. Rather than silently keeping a
  // local flag that pretends to control hardware, say so.
  const setWindBlock = useCallback(
    (b: boolean) => {
      setWindBlockRaw(b);
      toast.warning("Wind block is not available.", {
        description:
          "No verified interface exposes this on your headphones. The switch reflects your preference only and does not change the device.",
      });
      pushEvent({
        type: "WIND_BLOCK",
        description: "Wind block toggled — no device interface",
        status: "warning",
      });
    },
    [pushEvent],
  );

  // Software EQ would need a Windows DSP/APO pipeline, which does not exist.
  const setSoftwareEq = useCallback(
    (e: EqValues) => {
      setSoftwareEqRaw(e);
      toast.warning("Software EQ is not implemented.", {
        description:
          "This would require a Windows audio processing pipeline. Values are stored locally only and do not affect audio.",
      });
    },
    [],
  );

  const setConnection = useCallback(
    (c: ConnectionState) => {
      if (c === "disconnected") void run({ kind: "disconnect" }, { label: "Disconnect", eventType: "STATE" });
      else if (c === "connected") void run({ kind: "connect" }, { label: "Connect", eventType: "STATE" });
    },
    [run],
  );

  const connect = useCallback(() => {
    void run({ kind: "connect" }, { label: "Connect", eventType: "STATE" });
  }, [run]);

  const reconnect = useCallback(() => {
    void run({ kind: "reconnect" }, { label: "Reconnect", eventType: "STATE" });
  }, [run]);

  const disconnect = useCallback(() => {
    void run({ kind: "disconnect" }, { label: "Disconnect", eventType: "STATE" });
  }, [run]);

  const setSimulated = useCallback(
    async (s: boolean) => {
      setBusy("switch-source");
      try {
        const next = await ipc.setDeviceSource(s ? "mock" : "real");
        setSnapshot(next);
        pushEvent({
          type: "SESSION",
          description: s ? "Switched to simulated device" : "Switched to real hardware",
          status: "info",
        });
      } catch (e) {
        toast.error("Could not switch device source.", {
          description: ipc.toIpcError(e).message,
        });
      } finally {
        setBusy(null);
      }
    },
    [pushEvent],
  );

  /* ---- profiles (local only) ---- */
  const persistProfiles = useCallback((next: Profile[]) => {
    setProfiles(next);
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(PROFILES_KEY, JSON.stringify(next));
    }
  }, []);

  const applyProfile = useCallback(
    async (p: Profile) => {
      pushEvent({ type: "PROFILE", description: `Applying ${p.name}`, status: "user" });

      // A profile only applies settings the device is actually able to accept.
      // Each sub-command reports its own outcome, so a partially-applied
      // profile is visible rather than silently claimed as a success.
      await run(
        { kind: "setNoiseControl", mode: p.noise },
        { label: `${p.name}: noise control`, eventType: "PROFILE" },
      );
      await run(
        { kind: "setEqualizer", settings: p.eq },
        { label: `${p.name}: equalizer`, eventType: "PROFILE" },
      );

      persistProfiles(
        profiles.map((i) => (i.id === p.id ? { ...i, lastUsed: "Just now" } : i)),
      );
    },
    [run, pushEvent, profiles, persistProfiles],
  );

  const toggleCapture = useCallback(() => {
    setCapturing((c) => {
      const next = !c;
      capturingRef.current = next;
      pushEvent({
        type: "SESSION",
        description: next
          ? "Event capture started (application events only — no device event stream exists yet)"
          : "Event capture stopped",
        status: next ? "success" : "warning",
      });
      return next;
    });
  }, [pushEvent]);

  const value: AppState = useMemo(
    () => ({
      connection,
      setConnection,
      simulated: !!simulated,
      setSimulated,
      nativeAvailable,
      battery,
      charging,
      deviceName,
      batterySource,
      capabilities,
      snapshot,
      noise,
      setNoise,
      ancLevel,
      setAncLevel,
      windBlock,
      setWindBlock,
      volume,
      setVolume,
      muted,
      setMuted,
      audioEndpointName,
      hardwareEq,
      setHardwareEq,
      softwareEq,
      setSoftwareEq,
      profiles,
      addProfile: (p) => {
        persistProfiles([...profiles, p]);
        toast.success(`Profile “${p.name}” created.`, {
          description: "Stored locally in this application.",
        });
      },
      updateProfile: (p) => {
        persistProfiles(profiles.map((i) => (i.id === p.id ? p : i)));
        toast.success(`Profile “${p.name}” saved.`);
      },
      deleteProfile: (id) => persistProfiles(profiles.filter((i) => i.id !== id)),
      applyProfile: (p) => void applyProfile(p),
      events,
      capturing,
      toggleCapture,
      clearEvents: () => setEvents([]),
      theme,
      setTheme,
      settings,
      toggleSetting: (k) => {
        if (UNIMPLEMENTED_SETTINGS.has(k)) {
          toast.warning("Not implemented yet.", {
            description:
              "Tray, startup, auto-connect and notification behaviour are not wired up in this build.",
          });
          return;
        }
        setSettings((s) => ({ ...s, [k]: !s[k] }));
      },
      logLevel,
      setLogLevel,
      connect,
      reconnect,
      disconnect,
      refresh,
      busy,
    }),
    [
      connection, setConnection, simulated, setSimulated, nativeAvailable, connect,
      battery, charging, deviceName, batterySource, capabilities, snapshot,
      noise, setNoise, ancLevel, setAncLevel, windBlock, setWindBlock,
      volume, setVolume, muted, setMuted, audioEndpointName,
      hardwareEq, setHardwareEq, softwareEq, setSoftwareEq,
      profiles, persistProfiles, applyProfile, events, capturing, toggleCapture,
      theme, setTheme, settings, logLevel, reconnect, disconnect, refresh, busy,
    ],
  );

  return <AppCtx.Provider value={value}>{children}</AppCtx.Provider>;
}

export function useApp() {
  const ctx = useContext(AppCtx);
  if (!ctx) throw new Error("useApp must be used within AppProvider");
  return ctx;
}

/**
 * Capability metadata.
 *
 * The notes are worded to keep `supported` and `verified` clearly distinct:
 * an interface existing is not the same claim as the device demonstrating it.
 */
export const capabilityMeta: Record<
  Capability,
  { label: string; tone: "success" | "info" | "neutral" | "warning" | "muted"; note: string }
> = {
  verified: {
    label: "Verified",
    tone: "success",
    note: "Confirmed working against your physical headphones.",
  },
  supported: {
    label: "Supported",
    tone: "info",
    note: "A valid interface appears to expose this, but it has not been confirmed on your headphones.",
  },
  unknown: {
    label: "Unknown",
    tone: "neutral",
    note: "Not yet established whether this is accessible on this device.",
  },
  experimental: {
    label: "Experimental",
    tone: "warning",
    note: "Evidence suggests this may work, but verification is incomplete.",
  },
  unsupported: {
    label: "Unsupported",
    tone: "muted",
    note: "Cannot currently be accessed safely through available interfaces.",
  },
};
