import { describe, it, expect } from 'vitest';
import { BrowserSimulatedDevice } from './browserDevice';

describe('browser simulated device', () => {
  it('reveals nothing before connecting', async () => {
    const dev = new BrowserSimulatedDevice();
    const snap = await dev.snapshot();
    expect(snap.connection).toBe('disconnected');
    expect(snap.battery).toBeNull();
    expect(snap.identity).toBeNull();
    expect(snap.noiseControl).toBeNull();
    expect(snap.equalizer).toBeNull();
  });

  it('exposes simulated readings once connected', async () => {
    const dev = new BrowserSimulatedDevice();
    await dev.execute({ kind: 'connect' });
    const snap = await dev.snapshot();
    expect(snap.connection).toBe('connected');
    expect(snap.battery?.source).toBe('simulated');
    expect(snap.source).toBe('mock');
  });

  // The defining property: a simulation must never be able to convince the
  // application that a real capability was hardware-verified.
  it('never reports a hardware-verified capability', async () => {
    const dev = new BrowserSimulatedDevice();
    await dev.execute({ kind: 'connect' });
    const snap = await dev.snapshot();
    for (const cap of Object.values(snap.capabilities)) {
      expect(cap.hardwareVerified).toBe(false);
      expect(cap.status).not.toBe('verified');
    }
  });

  it('labels its identity as simulated', async () => {
    const dev = new BrowserSimulatedDevice();
    await dev.execute({ kind: 'connect' });
    const snap = await dev.snapshot();
    expect(snap.identity?.name.toUpperCase()).toContain('SIMULATED');
  });

  it('rejects commands when disconnected', async () => {
    const dev = new BrowserSimulatedDevice();
    const outcome = await dev.execute({ kind: 'setNoiseControl', mode: 'aware' });
    expect(outcome.kind).toBe('rejected');
  });

  it('rejects out-of-range EQ rather than clamping silently', async () => {
    const dev = new BrowserSimulatedDevice();
    await dev.execute({ kind: 'connect' });
    const outcome = await dev.execute({
      kind: 'setEqualizer',
      settings: { bass: 99, mid: 0, treble: 0 },
    });
    expect(outcome.kind).toBe('rejected');
  });

  it('applies and reports noise control changes', async () => {
    const dev = new BrowserSimulatedDevice();
    await dev.execute({ kind: 'connect' });
    const outcome = await dev.execute({ kind: 'setNoiseControl', mode: 'aware' });
    expect(outcome.kind).toBe('applied');
    const snap = await dev.snapshot();
    expect(snap.noiseControl?.mode).toBe('aware');
  });

  // Without this, the UI's "could not verify" path would never run in
  // development and would first be exercised against real hardware.
  it('exercises the unverified path periodically', async () => {
    const dev = new BrowserSimulatedDevice();
    await dev.execute({ kind: 'connect' });
    const kinds = new Set<string>();
    for (let i = 0; i < 14; i++) {
      const outcome = await dev.execute({ kind: 'setNoiseControl', mode: 'quiet' });
      kinds.add(outcome.kind);
    }
    expect(kinds.has('sent-unverified')).toBe(true);
  });

  it('reports windows audio as unsupported in the browser', async () => {
    const dev = new BrowserSimulatedDevice();
    await dev.execute({ kind: 'connect' });
    const outcome = await dev.execute({ kind: 'setSystemVolume', percent: 50 });
    expect(outcome.kind).toBe('unsupported');
  });

  it('clears exposed state on disconnect', async () => {
    const dev = new BrowserSimulatedDevice();
    await dev.execute({ kind: 'connect' });
    await dev.execute({ kind: 'disconnect' });
    const snap = await dev.snapshot();
    expect(snap.battery).toBeNull();
    expect(snap.identity).toBeNull();
  });
});
