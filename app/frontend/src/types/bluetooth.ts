export interface DiscoveredDevice {
  /** Opaque, salted hash. Never the raw Bluetooth address. */
  id: string;
  name: string;
  transport: 'classic' | 'low-energy' | 'unknown';
  connected: boolean | null;
  /** From the Windows PnP battery property, when the device populates it. */
  batteryPercent: number | null;
  /** Name matched a Bose hint. A hint only — never an identification. */
  looksLikeBose: boolean;
}

export interface BluetoothAvailability {
  radioPresent: boolean;
  radioEnabled: boolean;
  detail: string;
}
