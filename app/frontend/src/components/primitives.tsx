import type { ReactNode, ButtonHTMLAttributes } from 'react';

export function Panel({
  title,
  subtitle,
  actions,
  children,
  className = '',
}: {
  title?: ReactNode;
  subtitle?: ReactNode;
  actions?: ReactNode;
  children?: ReactNode;
  className?: string;
}) {
  return (
    <section className={`panel p-5 ${className}`}>
      {(title || actions) && (
        <header className="mb-4 flex items-start justify-between gap-4">
          <div className="min-w-0">
            {title && (
              <h2 className="font-[var(--font-display)] text-[15px] font-semibold tracking-tight">
                {title}
              </h2>
            )}
            {subtitle && (
              <p className="mt-0.5 text-[12.5px] leading-relaxed text-[var(--text-secondary)]">
                {subtitle}
              </p>
            )}
          </div>
          {actions && <div className="shrink-0">{actions}</div>}
        </header>
      )}
      {children}
    </section>
  );
}

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
  busy?: boolean;
};

export function Button({
  variant = 'secondary',
  busy = false,
  className = '',
  children,
  disabled,
  ...rest
}: ButtonProps) {
  const base =
    'inline-flex items-center justify-center gap-2 rounded-lg px-3.5 py-2 text-[13px] font-medium transition-all duration-150 disabled:opacity-45 disabled:cursor-not-allowed';
  const variants = {
    primary:
      'bg-[var(--color-accent-600)] text-white hover:bg-[var(--color-accent-500)] active:scale-[0.98]',
    secondary:
      'border border-[var(--border-strong)] bg-[var(--surface-inset)] hover:bg-[var(--surface-panel)] active:scale-[0.98]',
    ghost: 'hover:bg-[var(--surface-inset)] active:scale-[0.98]',
    danger:
      'border border-[color-mix(in_oklch,var(--color-status-unsupported)_40%,transparent)] text-[var(--color-status-unsupported)] hover:bg-[color-mix(in_oklch,var(--color-status-unsupported)_10%,transparent)]',
  };
  return (
    <button
      className={`${base} ${variants[variant]} ${className}`}
      disabled={disabled || busy}
      {...rest}
    >
      {busy && (
        <svg className="animate-spin" width="13" height="13" viewBox="0 0 16 16" aria-hidden="true">
          <circle
            cx="8"
            cy="8"
            r="6"
            stroke="currentColor"
            strokeWidth="2"
            fill="none"
            opacity="0.25"
          />
          <path
            d="M14 8a6 6 0 0 0-6-6"
            stroke="currentColor"
            strokeWidth="2"
            fill="none"
            strokeLinecap="round"
          />
        </svg>
      )}
      {children}
    </button>
  );
}

/**
 * A prominent, non-dismissible notice that the data on screen is simulated.
 *
 * This is intentionally hard to miss and cannot be turned off while a mock
 * backend is active.
 */
export function SimulatedBanner({ detail }: { detail: string }) {
  return (
    <div
      role="status"
      className="flex items-center gap-3 rounded-lg border border-[color-mix(in_oklch,var(--color-status-experimental)_40%,transparent)] bg-[color-mix(in_oklch,var(--color-status-experimental)_12%,transparent)] px-4 py-2.5"
    >
      <span className="rounded border border-[color-mix(in_oklch,var(--color-status-experimental)_50%,transparent)] px-1.5 py-0.5 text-[10px] font-bold tracking-widest text-[var(--color-status-experimental)]">
        SIMULATED
      </span>
      <p className="text-[12.5px] leading-snug text-[var(--text-secondary)]">{detail}</p>
    </div>
  );
}

export function StatusDot({ state }: { state: 'on' | 'off' | 'warn' | 'busy' }) {
  const color = {
    on: 'var(--color-status-verified)',
    off: 'var(--color-status-unknown)',
    warn: 'var(--color-status-experimental)',
    busy: 'var(--color-status-supported)',
  }[state];
  return (
    <span className="relative inline-flex h-2 w-2 shrink-0" aria-hidden="true">
      {state === 'busy' && (
        <span
          className="absolute inline-flex h-full w-full animate-ping rounded-full opacity-60"
          style={{ background: color }}
        />
      )}
      <span
        className="relative inline-flex h-2 w-2 rounded-full"
        style={{ background: color }}
      />
    </span>
  );
}

/** A labelled key/value row used on the device and diagnostics pages. */
export function Field({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="flex items-baseline justify-between gap-4 border-b border-[var(--border-subtle)] py-2.5 last:border-0">
      <dt className="text-[12.5px] text-[var(--text-secondary)]">{label}</dt>
      <dd className="selectable text-right text-[13px] font-medium">{value}</dd>
    </div>
  );
}

export function EmptyState({ title, detail }: { title: string; detail: ReactNode }) {
  return (
    <div className="rounded-lg border border-dashed border-[var(--border-strong)] px-5 py-8 text-center">
      <p className="text-[13.5px] font-medium">{title}</p>
      <p className="mx-auto mt-1.5 max-w-md text-[12.5px] leading-relaxed text-[var(--text-secondary)]">
        {detail}
      </p>
    </div>
  );
}
