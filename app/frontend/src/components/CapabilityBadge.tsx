import { CAPABILITY_LABEL, CAPABILITY_MEANING, type CapabilityStatus } from '@/types/capability';

const TONE: Record<CapabilityStatus, string> = {
  verified: 'text-[var(--color-status-verified)] bg-[color-mix(in_oklch,var(--color-status-verified)_14%,transparent)] border-[color-mix(in_oklch,var(--color-status-verified)_35%,transparent)]',
  supported:
    'text-[var(--color-status-supported)] bg-[color-mix(in_oklch,var(--color-status-supported)_14%,transparent)] border-[color-mix(in_oklch,var(--color-status-supported)_35%,transparent)]',
  experimental:
    'text-[var(--color-status-experimental)] bg-[color-mix(in_oklch,var(--color-status-experimental)_14%,transparent)] border-[color-mix(in_oklch,var(--color-status-experimental)_35%,transparent)]',
  unknown:
    'text-[var(--color-status-unknown)] bg-[color-mix(in_oklch,var(--color-status-unknown)_12%,transparent)] border-[color-mix(in_oklch,var(--color-status-unknown)_30%,transparent)]',
  unsupported:
    'text-[var(--color-status-unsupported)] bg-[color-mix(in_oklch,var(--color-status-unsupported)_14%,transparent)] border-[color-mix(in_oklch,var(--color-status-unsupported)_35%,transparent)]',
};

interface Props {
  status: CapabilityStatus;
  /** Shows the hardware-verification marker alongside the status. */
  hardwareVerified?: boolean;
  size?: 'sm' | 'md';
}

/**
 * The status chip used throughout the app.
 *
 * Status and hardware verification are shown as two separate signals, because
 * they are two separate claims — a capability can be `supported` (the interface
 * exists) while never having been confirmed on the user's headphones.
 */
export function CapabilityBadge({ status, hardwareVerified, size = 'md' }: Props) {
  return (
    <span className="inline-flex items-center gap-1.5">
      <span
        title={CAPABILITY_MEANING[status]}
        className={[
          'inline-flex items-center rounded-md border font-medium tracking-wide',
          size === 'sm' ? 'px-1.5 py-0.5 text-[10px]' : 'px-2 py-0.5 text-[11px]',
          TONE[status],
        ].join(' ')}
      >
        {CAPABILITY_LABEL[status]}
      </span>
      {hardwareVerified && (
        <span
          title="Confirmed against your physical headphones."
          className="text-[var(--color-status-verified)]"
          aria-label="Hardware verified"
        >
          <svg width="13" height="13" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path
              d="M8 1.5l1.8 1.3 2.2-.2.7 2.1 1.8 1.3-.9 2 .9 2-1.8 1.3-.7 2.1-2.2-.2L8 14.5l-1.8-1.3-2.2.2-.7-2.1L1.5 10l.9-2-.9-2 1.8-1.3.7-2.1 2.2.2L8 1.5z"
              fill="currentColor"
              opacity="0.18"
            />
            <path
              d="M5.4 8.1l1.9 1.9 3.4-3.7"
              stroke="currentColor"
              strokeWidth="1.7"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </span>
      )}
    </span>
  );
}
