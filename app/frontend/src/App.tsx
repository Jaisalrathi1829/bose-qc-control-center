import { useEffect } from 'react';
import { AppShell } from '@/layouts/AppShell';
import { Dashboard } from '@/pages/Dashboard';
import { DevicePage } from '@/pages/DevicePage';
import { Diagnostics } from '@/pages/Diagnostics';
import { Profiles } from '@/pages/Profiles';
import { Settings } from '@/pages/Settings';
import { SimulatedBanner } from '@/components/primitives';
import { Toasts } from '@/components/Toasts';
import { useUiStore, applyTheme } from '@/stores/uiStore';
import { useDeviceStore } from '@/stores/deviceStore';
import { runtimeMode } from '@/services/ipc';

export default function App() {
  const page = useUiStore((s) => s.page);
  const theme = useUiStore((s) => s.theme);
  const refresh = useDeviceStore((s) => s.refresh);
  const snapshot = useDeviceStore((s) => s.snapshot);

  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  // Re-resolve `system` when the OS theme changes.
  useEffect(() => {
    if (typeof matchMedia === 'undefined') return;
    const mq = matchMedia('(prefers-color-scheme: dark)');
    const onChange = () => applyTheme(useUiStore.getState().theme);
    mq.addEventListener('change', onChange);
    return () => mq.removeEventListener('change', onChange);
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const simulated = snapshot?.source === 'mock';
  const browserPreview = runtimeMode() === 'browser-preview';

  return (
    <AppShell>
      {simulated && (
        <div className="mb-4">
          <SimulatedBanner
            detail={
              browserPreview
                ? 'Browser preview with no native layer. Every value on screen is fabricated — no headphones are involved.'
                : 'The simulated device is active. Every value on screen is fabricated, not read from hardware.'
            }
          />
        </div>
      )}

      {page === 'dashboard' && <Dashboard />}
      {page === 'device' && <DevicePage />}
      {page === 'diagnostics' && <Diagnostics />}
      {page === 'profiles' && <Profiles />}
      {page === 'settings' && <Settings />}

      <Toasts />
    </AppShell>
  );
}
