import { motion } from "motion/react";
import { ImageWithFallback } from "./figma/ImageWithFallback";
import { cn } from "./ui/utils";

const HEADPHONE_IMG =
  "https://images.unsplash.com/photo-1693621947585-7b7d94149af4?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&q=80&w=1080";

// The source photo is sage-toned; this filter shifts it to the
// moonstone-blue finish of the reference product shot.
const MOONSTONE_FILTER = "hue-rotate(120deg) saturate(0.85) brightness(1.06) contrast(1.02)";

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
        <ImageWithFallback
          src={HEADPHONE_IMG}
          alt="Premium moonstone-blue over-ear headphones"
          style={{ filter: active ? MOONSTONE_FILTER : `${MOONSTONE_FILTER} grayscale(0.6)` }}
          className={cn(
            "size-full scale-110 object-cover transition-all duration-700",
            !active && "opacity-70",
          )}
        />
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
