import { useState } from "react";
import { Plus, Trash2, Waves } from "lucide-react";
import { motion } from "motion/react";
import {
  Button,
  Panel,
  Range,
  SectionLabel,
  SegmentedControl,
  Toggle,
} from "../primitives";
import { Modal } from "../Modal";
import { useApp, type EqValues, type NoiseMode, type Profile } from "../../store";

const noiseOptions: { value: NoiseMode; label: string }[] = [
  { value: "quiet", label: "Quiet" },
  { value: "aware", label: "Aware" },
  { value: "custom", label: "Custom" },
];

const blank = (): Profile => ({
  id: "",
  name: "",
  eq: { bass: 0, mid: 0, treble: 0 },
  noise: "quiet",
  windBlock: false,
  lastUsed: "Never",
});

export function ProfilesPage() {
  const { profiles, addProfile, updateProfile, deleteProfile, applyProfile } = useApp();
  const [editing, setEditing] = useState<Profile | null>(null);
  const [isNew, setIsNew] = useState(false);

  const openNew = () => { setEditing(blank()); setIsNew(true); };
  const openEdit = (p: Profile) => { setEditing({ ...p }); setIsNew(false); };

  const save = () => {
    if (!editing) return;
    if (isNew) addProfile({ ...editing, id: crypto.randomUUID() });
    else updateProfile(editing);
    setEditing(null);
  };

  const fmt = (v: number) => (v > 0 ? `+${v}` : `${v}`);

  return (
    <div>
      <div className="mb-5 flex items-center justify-between">
        <p className="text-sm text-muted-foreground">
          Profiles are local presets. Applying one updates EQ, noise mode and wind block.
        </p>
        <Button icon={<Plus className="size-4" />} onClick={openNew}>New Profile</Button>
      </div>

      {profiles.length === 0 ? (
        <Panel className="flex flex-col items-center justify-center gap-3 py-16 text-center">
          <div className="flex size-14 items-center justify-center rounded-2xl bg-accent text-primary">
            <Waves className="size-6" />
          </div>
          <div>
            <h3>You haven't created any profiles yet.</h3>
            <p className="mt-1 text-sm text-muted-foreground">Create a preset to switch settings in one tap.</p>
          </div>
          <Button icon={<Plus className="size-4" />} onClick={openNew}>Create Profile</Button>
        </Panel>
      ) : (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
          {profiles.map((p) => (
            <motion.div key={p.id} layout initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }}>
              <Panel className="flex h-full flex-col p-5">
                <div className="flex items-start justify-between">
                  <div>
                    <h3>{p.name}</h3>
                    <div className="mt-0.5 text-xs text-muted-foreground">Last used {p.lastUsed}</div>
                  </div>
                  <button onClick={() => deleteProfile(p.id)} className="rounded-md p-1 text-muted-foreground outline-none hover:bg-error-subtle hover:text-error">
                    <Trash2 className="size-4" />
                  </button>
                </div>

                <div className="my-4 grid grid-cols-3 gap-2 text-center">
                  {(["bass", "mid", "treble"] as const).map((b) => (
                    <div key={b} className="rounded-lg bg-surface-2 py-2">
                      <div className="text-sm tabular-nums">{fmt(p.eq[b])}</div>
                      <div className="text-xs uppercase text-muted-foreground">{b}</div>
                    </div>
                  ))}
                </div>

                <div className="mb-4 flex items-center gap-2 text-sm">
                  <span className="text-muted-foreground">Noise</span>
                  <span className="rounded-md bg-accent px-2 py-0.5 text-xs capitalize text-accent-foreground">{p.noise}</span>
                  {p.windBlock && <span className="rounded-md bg-info-subtle px-2 py-0.5 text-xs text-info">Wind Block</span>}
                </div>

                <div className="mt-auto flex gap-2">
                  <Button className="flex-1" onClick={() => applyProfile(p)}>Apply</Button>
                  <Button variant="outline" onClick={() => openEdit(p)}>Edit</Button>
                </div>
              </Panel>
            </motion.div>
          ))}
        </div>
      )}

      <Modal
        open={!!editing}
        onClose={() => setEditing(null)}
        title={isNew ? "Create Profile" : "Edit Profile"}
        description="Configure the preset values."
        width={460}
        footer={
          <>
            <Button variant="outline" onClick={() => setEditing(null)}>Cancel</Button>
            <Button onClick={save} disabled={!editing?.name?.trim()}>Save Profile</Button>
          </>
        }
      >
        {editing && (
          <div className="space-y-5">
            <div>
              <label className="mb-1.5 block">Name</label>
              <input
                value={editing.name}
                onChange={(e) => setEditing({ ...editing, name: e.target.value })}
                placeholder="e.g. Focus"
                className="w-full rounded-lg border border-border bg-input-background px-3 py-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
              />
            </div>
            <div className="space-y-4">
              {(["bass", "mid", "treble"] as const).map((b) => (
                <Range key={b} label={b[0].toUpperCase() + b.slice(1)} value={editing.eq[b]} min={-10} max={10}
                  leftLabel="−10" rightLabel="+10" displayValue={fmt(editing.eq[b])}
                  onChange={(v) => setEditing({ ...editing, eq: { ...editing.eq, [b]: v } as EqValues })} />
              ))}
            </div>
            <div>
              <label className="mb-2 block">Noise Control</label>
              <SegmentedControl options={noiseOptions} value={editing.noise}
                onChange={(n) => setEditing({ ...editing, noise: n })} className="w-full [&>button]:flex-1" />
            </div>
            <div className="flex items-center justify-between">
              <label>Wind Block</label>
              <Toggle checked={editing.windBlock} onChange={(b) => setEditing({ ...editing, windBlock: b })} />
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
}
