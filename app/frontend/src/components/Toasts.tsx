import { useEffect } from 'react';
import { useDeviceStore } from '@/stores/deviceStore';

const TONE = {
  success: 'border-[color-mix(in_oklch,var(--color-status-verified)_45%,transparent)] bg-[color-mix(in_oklch,var(--color-status-verified)_12%,var(--surface-panel))]',
  caution:
    'border-[color-mix(in_oklch,var(--color-status-experimental)_45%,transparent)] bg-[color-mix(in_oklch,var(--color-status-experimental)_12%,var(--surface-panel))]',
  error:
    'border-[color-mix(in_oklch,var(--color-status-unsupported)_45%,transparent)] bg-[color-mix(in_oklch,var(--color-status-unsupported)_12%,var(--surface-panel))]',
} as const;

function Toast({ id, tone, text }: { id: number; tone: keyof typeof TONE; text: string }) {
  const dismiss = useDeviceStore((s) => s.dismissToast);

  useEffect(() => {
    // Cautions stay longer — they carry information the user needs to read.
    const ms = tone === 'success' ? 3200 : 7000;
    const t = setTimeout(() => dismiss(id), ms);
    return () => clearTimeout(t);
  }, [id, tone, dismiss]);

  return (
    <div
      role="status"
      className={`pointer-events-auto flex items-start gap-3 rounded-lg border px-4 py-3 shadow-lg backdrop-blur ${TONE[tone]}`}
    >
      <p className="text-[12.5px] leading-snug">{text}</p>
      <button
        type="button"
        onClick={() => dismiss(id)}
        aria-label="Dismiss"
        className="ml-auto shrink-0 text-[var(--text-tertiary)] hover:text-[var(--text-primary)]"
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />
        </svg>
      </button>
    </div>
  );
}

export function Toasts() {
  const toasts = useDeviceStore((s) => s.toasts);
  if (toasts.length === 0) return null;

  return (
    <div className="pointer-events-none fixed bottom-5 right-5 z-50 flex w-[340px] flex-col gap-2">
      {toasts.slice(-4).map((t) => (
        <Toast key={t.id} id={t.id} tone={t.tone} text={t.text} />
      ))}
    </div>
  );
}
