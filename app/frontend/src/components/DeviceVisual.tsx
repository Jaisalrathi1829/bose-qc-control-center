import { motion } from "motion/react";
import { cn } from "./ui/utils";

/**
 * Headphone illustration, drawn locally.
 *
 * The Figma export used an Unsplash photo. That is replaced here because this
 * application makes no network requests by design — the CSP in index.html and
 * tauri.conf.json restricts `img-src` to `'self' data:`, so the remote image
 * was blocked outright and rendered as a broken fallback. Drawing it inline
 * also means the visual works with no network at all, which matters for an
 * offline-first desktop utility.
 *
 * It is a generic over-ear illustration, deliberately not a depiction of any
 * specific manufacturer's product.
 */
function HeadphoneMark({ active }: { active: boolean }) {
  return (
    <svg
      viewBox="0 0 200 200"
      className={cn("size-full transition-opacity duration-700", !active && "opacity-60")}
      role="img"
      aria-label="Over-ear headphones"
    >
      <defs>
        <linearGradient id="dv-cup" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="var(--chart-4)" stopOpacity="0.95" />
          <stop offset="100%" stopColor="var(--primary)" stopOpacity="0.75" />
        </linearGradient>
        <linearGradient id="dv-band" x1="0" y1="0" x2="1" y2="0">
          <stop offset="0%" stopColor="var(--primary)" stopOpacity="0.5" />
          <stop offset="50%" stopColor="var(--chart-4)" stopOpacity="0.95" />
          <stop offset="100%" stopColor="var(--primary)" stopOpacity="0.5" />
        </linearGradient>
      </defs>

      {/* headband */}
      <path
        d="M46 116 V96a54 54 0 0 1 108 0v20"
        fill="none"
        stroke="url(#dv-band)"
        strokeWidth="13"
        strokeLinecap="round"
      />
      <path
        d="M46 112 V96a54 54 0 0 1 108 0v16"
        fill="none"
        stroke="white"
        strokeOpacity="0.18"
        strokeWidth="3"
        strokeLinecap="round"
      />

      {/* earcups */}
      {[36, 136].map((x) => (
        <g key={x}>
          <rect x={x} y="104" width="28" height="52" rx="14" fill="url(#dv-cup)" />
          <rect
            x={x + 5}
            y="111"
            width="18"
            height="38"
            rx="9"
            fill="black"
            fillOpacity="0.22"
          />
          <rect
            x={x + 3}
            y="107"
            width="22"
            height="16"
            rx="8"
            fill="white"
            fillOpacity="0.16"
          />
        </g>
      ))}
    </svg>
  );
}

/**
 * DeviceVisual — realistic headphone render with layered, mesmerizing motion.
 * A bright light revolves around the ring, a specular highlight sweeps across
 * the photo, sound rings expand outward and an orbiting light dot circles it.
 */
export function DeviceVisual({
  size = 320,
  active = true,
  className,
}: {
  size?: number;
  active?: boolean;
  className?: string;
}) {
  const ring = size * 0.8; // photo diameter

  return (
    <div
      className={cn("relative flex items-center justify-center", className)}
      style={{ width: size, height: size }}
    >
      {/* ambient breathing glow */}
      <motion.div
        className="absolute rounded-full blur-3xl"
        style={{
          width: size * 0.95,
          height: size * 0.95,
          background: "radial-gradient(circle, var(--primary) 0%, transparent 62%)",
        }}
        animate={
          active
            ? { opacity: [0.3, 0.6, 0.3], scale: [0.9, 1.1, 0.9] }
            : { opacity: 0.06, scale: 1 }
        }
        transition={{ duration: 6, repeat: Infinity, ease: "easeInOut" }}
      />

      {/* REVOLVING LIGHT RING — a bright arc that sweeps around the circle.
         Hue cycles through a warm aurora palette as it revolves. */}
      {active && (
        <motion.div
          className="absolute rounded-full"
          style={{
            width: ring * 1.12,
            height: ring * 1.12,
            background:
              "conic-gradient(from 0deg, transparent 0deg, transparent 250deg, rgba(255,138,76,0.5) 320deg, #ffffff 355deg, rgba(180,120,255,0.5) 360deg)",
            WebkitMask:
              "radial-gradient(farthest-side, transparent calc(100% - 7px), #000 calc(100% - 6px))",
            mask: "radial-gradient(farthest-side, transparent calc(100% - 7px), #000 calc(100% - 6px))",
            filter: "drop-shadow(0 0 7px rgba(255,150,90,0.7))",
          }}
          animate={{ rotate: 360, filter: [
            "drop-shadow(0 0 7px rgba(255,150,90,0.75)) hue-rotate(0deg)",
            "drop-shadow(0 0 7px rgba(255,150,90,0.75)) hue-rotate(360deg)",
          ] }}
          transition={{
            rotate: { duration: 9, repeat: Infinity, ease: "linear" },
            filter: { duration: 14, repeat: Infinity, ease: "linear" },
          }}
        />
      )}

      {/* expanding sound rings */}
      {active &&
        [0, 1, 2].map((i) => (
          <motion.span
            key={i}
            className="absolute rounded-full border-2 border-primary/40"
            style={{ width: ring * 1.05, height: ring * 1.05 }}
            initial={{ scale: 0.78, opacity: 0.7 }}
            animate={{ scale: 1.35, opacity: 0 }}
            transition={{ duration: 6, repeat: Infinity, delay: i * 2, ease: "easeOut" }}
          />
        ))}

      {/* slowly counter-rotating dashed halo */}
      {active && (
        <motion.svg
          className="absolute"
          width={size * 0.98}
          height={size * 0.98}
          viewBox="0 0 100 100"
          animate={{ rotate: -360 }}
          transition={{ duration: 40, repeat: Infinity, ease: "linear" }}
        >
          <circle
            cx="50"
            cy="50"
            r="47"
            fill="none"
            stroke="var(--primary)"
            strokeOpacity="0.3"
            strokeWidth="0.6"
            strokeDasharray="1 6"
            strokeLinecap="round"
          />
        </motion.svg>
      )}

      {/* ORBITING LIGHT DOT — a glowing bead that circles the visual */}
      {active && (
        <motion.div
          className="absolute"
          style={{ width: ring * 1.12, height: ring * 1.12 }}
          animate={{ rotate: 360 }}
          transition={{ duration: 12, repeat: Infinity, ease: "linear" }}
        >
          <motion.span
            className="absolute left-1/2 top-0 size-3 -translate-x-1/2 rounded-full bg-white"
            style={{ boxShadow: "0 0 12px 4px rgba(255,150,90,0.9), 0 0 4px 1px #fff" }}
            animate={{ scale: [1, 1.5, 1], opacity: [0.85, 1, 0.85] }}
            transition={{ duration: 2.6, repeat: Infinity, ease: "easeInOut" }}
          />
        </motion.div>
      )}

      {/* floating headphone photo */}
      <motion.div
        className="relative overflow-hidden rounded-full border border-white/20 shadow-[var(--shadow-lg)]"
        style={{ width: ring, height: ring }}
        initial={{ y: 0 }}
        animate={active ? { y: [-8, 8, -8], rotate: [-2, 2, -2] } : { y: 0, rotate: 0 }}
        transition={{ duration: 7, repeat: Infinity, ease: "easeInOut" }}
        whileHover={{ scale: 1.05 }}
      >
        <div className="absolute inset-0 bg-gradient-to-br from-surface-2 to-card" />
        <div className="absolute inset-0 p-[14%]">
          <HeadphoneMark active={active} />
        </div>
        {/* subtle inner sheen */}
        <div className="pointer-events-none absolute inset-0 rounded-full bg-gradient-to-tr from-transparent via-transparent to-white/15" />

        {/* BRIGHT REVOLVING SPECULAR SWEEP across the photo */}
        {active && (
          <motion.div
            className="pointer-events-none absolute inset-[-25%]"
            style={{
              background:
                "conic-gradient(from 0deg, transparent 0deg, transparent 300deg, rgba(255,255,255,0.15) 330deg, rgba(255,255,255,0.75) 352deg, rgba(255,255,255,0.15) 358deg, transparent 360deg)",
              mixBlendMode: "screen",
            }}
            animate={{ rotate: 360 }}
            transition={{ duration: 10, repeat: Infinity, ease: "linear" }}
          />
        )}
      </motion.div>

      {/* soft contact shadow beneath */}
      <motion.div
        className="absolute rounded-full bg-black/40 blur-md"
        style={{ width: size * 0.4, height: size * 0.06, bottom: size * 0.06 }}
        animate={
          active ? { scaleX: [1, 0.8, 1], opacity: [0.42, 0.24, 0.42] } : { scaleX: 1, opacity: 0.3 }
        }
        transition={{ duration: 7, repeat: Infinity, ease: "easeInOut" }}
      />
    </div>
  );
}
