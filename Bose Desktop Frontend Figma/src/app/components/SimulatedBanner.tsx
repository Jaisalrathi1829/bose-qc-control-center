import { FlaskConical, X } from "lucide-react";
import { useApp } from "../store";

/** Unmistakable banner shown whenever the device is simulated. */
export function SimulatedBanner() {
  const { setConnection } = useApp();
  return (
    <div className="flex items-center justify-between gap-3 border-b border-warning/30 bg-warning-subtle px-8 py-2 text-sm">
      <div className="flex items-center gap-2 text-warning">
        <FlaskConical className="size-4" />
        <span className="font-medium">Simulated Device</span>
        <span className="text-muted-foreground">Mock data — does not represent verified hardware behavior.</span>
      </div>
      <button
        onClick={() => setConnection("disconnected")}
        className="rounded-md p-1 text-warning outline-none hover:bg-warning/10"
        aria-label="Exit simulated device"
      >
        <X className="size-4" />
      </button>
    </div>
  );
}
