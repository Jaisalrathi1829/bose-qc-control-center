import { create } from 'zustand';
import type { CommandOutcome, DeviceSnapshot, DeviceSource } from '@/types/device';
import type { BluetoothAvailability } from '@/types/bluetooth';
import * as ipc from '@/services/ipc';
import type { DeviceCommand } from '@/services/ipc';

/**
 * A transient message shown after a command.
 *
 * `tone` is driven by the command *outcome*, not by whether the IPC call threw.
 * A command that was transmitted but not confirmed is a caution, never a
 * success — that distinction is the whole point of the outcome type.
 */
export interface Toast {
  id: number;
  tone: 'success' | 'caution' | 'error';
  text: string;
}

interface DeviceStore {
  snapshot: DeviceSnapshot | null;
  bluetooth: BluetoothAvailability | null;
  loading: boolean;
  /** Command currently in flight, by name, so buttons can show a busy state. */
  pending: string | null;
  error: string | null;
  toasts: Toast[];

  refresh: () => Promise<void>;
  loadBluetooth: () => Promise<void>;
  switchSource: (source: DeviceSource) => Promise<void>;
  run: (command: DeviceCommand) => Promise<CommandOutcome | null>;
  dismissToast: (id: number) => void;
}

let toastSeq = 0;

/**
 * Turns an outcome into user-facing text.
 *
 * Note that `sent-unverified` deliberately does not read as success. The user
 * is told plainly that the state could not be confirmed.
 */
function describe(outcome: CommandOutcome): Toast {
  switch (outcome.kind) {
    case 'applied':
      return { id: ++toastSeq, tone: 'success', text: 'Applied and confirmed by the device.' };
    case 'sent-unverified':
      return {
        id: ++toastSeq,
        tone: 'caution',
        text: `Command sent. State could not be verified. ${outcome.reason}`.trim(),
      };
    case 'rejected':
      return {
        id: ++toastSeq,
        tone: 'error',
        text: `Device rejected the command. ${outcome.reason}`.trim(),
      };
    case 'unsupported':
      return { id: ++toastSeq, tone: 'caution', text: outcome.reason };
  }
}

export const useDeviceStore = create<DeviceStore>((set, get) => ({
  snapshot: null,
  bluetooth: null,
  loading: false,
  pending: null,
  error: null,
  toasts: [],

  refresh: async () => {
    set({ loading: true });
    try {
      const snapshot = await ipc.getSnapshot();
      set({ snapshot, error: null });
    } catch (e) {
      set({ error: ipc.toIpcError(e).message });
    } finally {
      set({ loading: false });
    }
  },

  loadBluetooth: async () => {
    try {
      set({ bluetooth: await ipc.getBluetoothAvailability() });
    } catch (e) {
      set({ error: ipc.toIpcError(e).message });
    }
  },

  switchSource: async (source) => {
    set({ pending: 'switch-source' });
    try {
      const snapshot = await ipc.setDeviceSource(source);
      set({ snapshot, error: null });
    } catch (e) {
      const err = ipc.toIpcError(e);
      set((s) => ({
        error: err.message,
        toasts: [...s.toasts, { id: ++toastSeq, tone: 'error', text: err.message }],
      }));
    } finally {
      set({ pending: null });
    }
  },

  run: async (command) => {
    set({ pending: command.kind });
    try {
      const outcome = await ipc.executeCommand(command);
      set((s) => ({ toasts: [...s.toasts, describe(outcome)] }));
      // Always re-read state from the device afterwards. The UI never assumes
      // a command took effect — it re-reads and renders what came back.
      await get().refresh();
      return outcome;
    } catch (e) {
      const err = ipc.toIpcError(e);
      set((s) => ({
        error: err.message,
        toasts: [...s.toasts, { id: ++toastSeq, tone: 'error', text: err.message }],
      }));
      return null;
    } finally {
      set({ pending: null });
    }
  },

  dismissToast: (id) => set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),
}));
