import { create } from 'zustand';

export type ThemePreference = 'system' | 'light' | 'dark';
export type PageId = 'dashboard' | 'device' | 'diagnostics' | 'profiles' | 'settings';

const THEME_KEY = 'bose-qc.theme';

function loadTheme(): ThemePreference {
  if (typeof localStorage === 'undefined') return 'system';
  const raw = localStorage.getItem(THEME_KEY);
  return raw === 'light' || raw === 'dark' || raw === 'system' ? raw : 'system';
}

/** Applies the preference to the document, resolving `system` against the OS. */
export function applyTheme(pref: ThemePreference): void {
  if (typeof document === 'undefined') return;
  const root = document.documentElement;
  root.setAttribute('data-theme', pref);

  const prefersDark =
    typeof matchMedia !== 'undefined' && matchMedia('(prefers-color-scheme: dark)').matches;
  root.classList.toggle('sys-dark', pref === 'system' && prefersDark);
}

interface UiStore {
  page: PageId;
  theme: ThemePreference;
  setPage: (page: PageId) => void;
  setTheme: (theme: ThemePreference) => void;
}

export const useUiStore = create<UiStore>((set) => ({
  page: 'dashboard',
  theme: loadTheme(),
  setPage: (page) => set({ page }),
  setTheme: (theme) => {
    if (typeof localStorage !== 'undefined') localStorage.setItem(THEME_KEY, theme);
    applyTheme(theme);
    set({ theme });
  },
}));
