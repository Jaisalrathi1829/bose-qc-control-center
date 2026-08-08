import {
  Activity,
  LayoutDashboard,
  Settings,
  SlidersHorizontal,
  Speaker,
  Volume2,
  Waves,
} from "lucide-react";
import { motion } from "motion/react";
import { cn } from "./ui/utils";
import { BatteryIndicator, ConnectionBadge } from "./primitives";
import { useApp } from "../store";
import type { PageId } from "../pages";
import wolfLogo from "@/assets/wolf-logo.png";

const nav: { id: PageId; label: string; icon: React.ElementType }[] = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
  { id: "device", label: "Device", icon: Speaker },
  { id: "noise", label: "Noise Control", icon: Waves },
  { id: "equalizer", label: "Equalizer", icon: SlidersHorizontal },
  { id: "profiles", label: "Profiles", icon: Volume2 },
  { id: "diagnostics", label: "Diagnostics", icon: Activity },
  { id: "settings", label: "Settings", icon: Settings },
];

export function Sidebar({
  page,
  onNavigate,
}: {
  page: PageId;
  onNavigate: (p: PageId) => void;
}) {
  const { connection, battery, charging } = useApp();

  return (
    <aside className="flex w-60 shrink-0 flex-col border-r border-sidebar-border bg-sidebar">
      {/* brand */}
      <div className="flex items-center gap-3 px-5 py-5">
        <motion.div
          className="flex size-14 shrink-0 items-center justify-center overflow-hidden rounded-xl bg-white ring-1 ring-border shadow-[var(--shadow-md)]"
          whileHover={{ scale: 1.08, rotate: [0, -6, 6, 0] }}
          transition={{ duration: 0.5 }}
        >
          <img src={wolfLogo} alt="VOID logo" className="size-full object-cover" />
        </motion.div>
        <div className="leading-tight">
          <div className="text-lg font-semibold tracking-[0.2em]">VOID</div>
          <div className="text-xs text-muted-foreground">Control Center</div>
        </div>
      </div>

      {/* nav */}
      <nav className="flex-1 px-3 py-2">
        <ul className="space-y-0.5">
          {nav.map((item) => {
            const active = page === item.id;
            const Icon = item.icon;
            return (
              <motion.li
                key={item.id}
                initial={{ opacity: 0, x: -16 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: 0.04 * nav.indexOf(item), type: "spring", stiffness: 300, damping: 24 }}
              >
                <motion.button
                  onClick={() => onNavigate(item.id)}
                  whileHover={{ x: 4 }}
                  whileTap={{ scale: 0.97 }}
                  className={cn(
                    "group relative flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm outline-none transition-colors",
                    "focus-visible:ring-2 focus-visible:ring-sidebar-ring",
                    active
                      ? "bg-sidebar-accent text-sidebar-accent-foreground"
                      : "text-muted-foreground hover:bg-sidebar-accent/50 hover:text-sidebar-foreground",
                  )}
                >
                  {active && (
                    <motion.span
                      layoutId="nav-active"
                      className="absolute left-0 top-1/2 h-5 w-0.5 -translate-y-1/2 rounded-full bg-primary"
                    />
                  )}
                  <motion.span
                    animate={active ? { scale: [1, 1.25, 1] } : {}}
                    transition={{ duration: 0.4 }}
                    className="inline-flex"
                  >
                    <Icon className="size-4" />
                  </motion.span>
                  {item.label}
                </motion.button>
              </motion.li>
            );
          })}
        </ul>
      </nav>

      {/* status footer */}
      <div className="m-3 space-y-3 rounded-lg border border-sidebar-border bg-card/60 p-3">
        <div className="flex items-center justify-between">
          <span className="text-xs text-muted-foreground">Connection</span>
          <ConnectionBadge state={connection} />
        </div>
        <div className="flex items-center justify-between">
          <span className="text-xs text-muted-foreground">Battery</span>
          <BatteryIndicator level={battery} charging={charging} />
        </div>
      </div>
    </aside>
  );
}
