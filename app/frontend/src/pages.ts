export type PageId =
  | "dashboard"
  | "device"
  | "noise"
  | "equalizer"
  | "profiles"
  | "diagnostics"
  | "settings";

export const pageTitles: Record<PageId, { title: string; subtitle: string }> = {
  dashboard: { title: "Dashboard", subtitle: "Overview and quick controls" },
  device: { title: "Device", subtitle: "Connection and hardware details" },
  noise: { title: "Noise Control", subtitle: "Manage ambient noise and awareness" },
  equalizer: { title: "Equalizer", subtitle: "Hardware and software audio shaping" },
  profiles: { title: "Profiles", subtitle: "Saved control presets" },
  diagnostics: { title: "Diagnostics", subtitle: "Device capabilities and event capture" },
  settings: { title: "Settings", subtitle: "Application preferences" },
};
