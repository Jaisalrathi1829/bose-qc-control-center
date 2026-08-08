/**
 * The only channel between the UI and the native layer.
 *
 * Note what this module does *not* export: there is no `write(uuid, bytes)`,
 * no characteristic handle, no raw payload. The native side exposes a fixed
 * set of typed commands and nothing else, so the UI is structurally incapable
 * of issuing an arbitrary Bluetooth write.
 *
 * When the app is opened in a plain browser (`npm run dev` without Tauri), the
 * native layer is absent. Rather than showing a broken page, we fall back to a
 * browser-side simulation — which is labelled SIMULATED exactly like the
 * native mock, because it is one.
 */

import type { DeviceSnapshot, DeviceSource, CommandOutcome } from '@/types/device';
import type { DiscoveredDevice, BluetoothAvailability } from '@/types/bluetooth';
import { BrowserSimulatedDevice } from '@/mock/browserDevice';

export type DeviceCommand =
  | { kind: 'refreshSnapshot' }
  | { kind: 'readBattery' }
  | { kind: 'readNoiseControl' }
  | { kind: 'readEqualizer' }
  | { kind: 'readDeviceInfo' }
  | { kind: 'connect' }
  | { kind: 'disconnect' }
  | { kind: 'reconnect' }
  | { kind: 'setSystemVolume'; percent: number }
  | { kind: 'setSystemMute'; muted: boolean }
  | { kind: 'mediaPlayPause' }
  | { kind: 'mediaNext' }
  | { kind: 'mediaPrevious' }
  | { kind: 'setNoiseControl'; mode: 'quiet' | 'aware' | 'custom' | 'off' }
  | { kind: 'setNoiseControlLevel'; level: number }
  | { kind: 'setEqualizer'; settings: { bass: number; mid: number; treble: number } };

export interface IpcError {
  message: string;
  kind: string;
}

/** True when running inside the Tauri shell with a real native layer. */
export function hasNativeLayer(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/**
 * Where the current session's data comes from. Surfaced in the UI so the
 * distinction between "native mock", "browser preview" and "real hardware" is
 * never ambiguous.
 */
export type RuntimeMode = 'native' | 'browser-preview';

export function runtimeMode(): RuntimeMode {
  return hasNativeLayer() ? 'native' : 'browser-preview';
}

const browserFallback = new BrowserSimulatedDevice();

async function invokeNative<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}

export async function getSnapshot(): Promise<DeviceSnapshot> {
  if (!hasNativeLayer()) return browserFallback.snapshot();
  return invokeNative<DeviceSnapshot>('get_snapshot');
}

export async function getDeviceSource(): Promise<DeviceSource> {
  if (!hasNativeLayer()) return 'mock';
  return invokeNative<DeviceSource>('get_device_source');
}

export async function setDeviceSource(source: DeviceSource): Promise<DeviceSnapshot> {
  if (!hasNativeLayer()) {
    // A browser preview cannot reach hardware. Say so rather than pretending
    // the switch worked.
    if (source === 'real') {
      throw {
        kind: 'unsupported',
        message:
          'Real hardware is only reachable from the desktop application, not the browser preview.',
      } satisfies IpcError;
    }
    return browserFallback.snapshot();
  }
  return invokeNative<DeviceSnapshot>('set_device_source', { source });
}

export async function executeCommand(command: DeviceCommand): Promise<CommandOutcome> {
  if (!hasNativeLayer()) return browserFallback.execute(command);
  return invokeNative<CommandOutcome>('execute_command', { command });
}

export async function getBluetoothAvailability(): Promise<BluetoothAvailability> {
  if (!hasNativeLayer()) {
    return {
      radioPresent: false,
      radioEnabled: false,
      detail: 'Browser preview: no access to Bluetooth hardware.',
    };
  }
  return invokeNative<BluetoothAvailability>('get_bluetooth_availability');
}

export async function listBluetoothDevices(): Promise<DiscoveredDevice[]> {
  if (!hasNativeLayer()) return [];
  return invokeNative<DiscoveredDevice[]>('list_bluetooth_devices');
}

/**
 * Opens the Windows Bluetooth settings page.
 *
 * The URI lives in native code and this call takes no argument, so the UI
 * cannot use it to open anything else.
 */
export async function openBluetoothSettings(): Promise<void> {
  if (!hasNativeLayer()) {
    throw {
      kind: 'unsupported',
      message: 'Windows settings can only be opened from the desktop application.',
    } satisfies IpcError;
  }
  return invokeNative<void>('open_bluetooth_settings');
}

/** Normalises anything thrown by the IPC layer into a displayable error. */
export function toIpcError(e: unknown): IpcError {
  if (e && typeof e === 'object' && 'message' in e && 'kind' in e) {
    return e as IpcError;
  }
  return {
    kind: 'unknown',
    message: e instanceof Error ? e.message : String(e),
  };
}
