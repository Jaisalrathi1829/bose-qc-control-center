import { Info, Waves, Wind } from "lucide-react";
import {
  CapabilityBadge,
  Panel,
  Range,
  SectionLabel,
  SegmentedControl,
  Toggle,
} from "../primitives";
import { useApp, type NoiseMode, capabilityMeta } from "../../store";

const noiseOptions: { value: NoiseMode; label: string; icon: React.ReactNode }[] = [
  { value: "quiet", label: "Quiet", icon: <Waves className="size-4" /> },
  { value: "aware", label: "Aware", icon: <Info className="size-4" /> },
  { value: "custom", label: "Custom", icon: <Wind className="size-4" /> },
];

export function NoisePage() {
  const { noise, setNoise, ancLevel, setAncLevel, windBlock, setWindBlock, capabilities } = useApp();
  const cap = capabilities.noiseControl;
  const disabled = cap === "unknown" || cap === "unsupported";

  const descriptions: Record<NoiseMode, string> = {
    quiet: "Maximum noise cancellation for focus and quiet environments.",
    aware: "Let ambient sound through so you stay aware of your surroundings.",
    custom: "Dial in the exact level of noise cancellation you prefer.",
  };

  return (
    <div className="mx-auto max-w-3xl space-y-5">
      <Panel className="p-6">
        <div className="mb-4 flex items-center justify-between">
          <SectionLabel>Noise Mode</SectionLabel>
          <CapabilityBadge cap={cap} />
        </div>

        <SegmentedControl options={noiseOptions} value={noise} onChange={setNoise} disabled={disabled} className="w-full [&>button]:flex-1" />

        <p className="mt-3 text-sm text-muted-foreground">
          {disabled
            ? capabilityMeta[cap].note
            : noise
              ? descriptions[noise]
              : "The device has not reported its current mode, so none is shown as active."}
        </p>

        {disabled && (
          <div className="mt-4 flex items-start gap-2 rounded-lg border border-border bg-warning-subtle/40 p-3 text-sm">
            <Info className="mt-0.5 size-4 shrink-0 text-warning" />
            <span className="text-muted-foreground">
              This control is not yet verified on your headphones, so changes may not take effect on the device.
            </span>
          </div>
        )}
      </Panel>

      {noise === "custom" && !disabled && (
        <Panel className="space-y-6 p-6">
          <div>
            <SectionLabel>Custom Noise Cancellation</SectionLabel>
            <div className="mt-4">
              <Range label="Noise Cancellation" value={ancLevel} min={0} max={10} leftLabel="0" rightLabel="10"
                displayValue={`${ancLevel}`} onChange={setAncLevel} />
            </div>
          </div>
          <div className="flex items-center justify-between border-t border-border pt-4">
            <div>
              <div className="text-sm">Wind Block</div>
              <div className="text-xs text-muted-foreground">Reduces wind noise in outdoor conditions.</div>
            </div>
            <Toggle checked={windBlock} onChange={setWindBlock} label="Wind Block" />
          </div>
        </Panel>
      )}
    </div>
  );
}
