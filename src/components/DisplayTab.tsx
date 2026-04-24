import { Profile, MonitorType } from '../types/profile';

interface Props {
  profile:  Profile;
  onChange: (p: Profile) => void;
}

function set<K extends keyof Profile>(p: Profile, key: K, val: Profile[K]): Profile {
  return { ...p, [key]: val };
}

const MONITOR_OPTIONS: { value: MonitorType; label: string; note: string }[] = [
  { value: 'Colour', label: 'Colour (RGB)',  note: 'ST / STE / MegaSTE'    },
  { value: 'Mono',   label: 'Monochrome',    note: 'SM124 / SM125 monitor'  },
  { value: 'VGA',    label: 'VGA',           note: 'Falcon / TT'            },
  { value: 'TV',     label: 'TV (RF)',        note: 'ST / STE composite out' },
];

interface ToggleProps {
  label:    string;
  value:    boolean;
  onChange: (v: boolean) => void;
  note?:    string;
}

function Toggle({ label, value, onChange, note }: ToggleProps) {
  return (
    <div className="toggle-row">
      <div>
        <span className="toggle-label">{label}</span>
        {note && <div className="toggle-note">{note}</div>}
      </div>
      <button
        className={`toggle ${value ? 'on' : 'off'}`}
        onClick={() => onChange(!value)}
        aria-pressed={value}
      />
    </div>
  );
}

export default function DisplayTab({ profile: p, onChange }: Props) {
  return (
    <div className="tab-inner">

      {/* Monitor type */}
      <div className="section-label">Monitor type</div>
      <div className="card">
        <div className="monitor-options">
          {MONITOR_OPTIONS.map(opt => (
            <div
              key={opt.value}
              className={`monitor-opt ${p.monitor === opt.value ? 'sel' : ''}`}
              onClick={() => onChange(set(p, 'monitor', opt.value))}
            >
              <div className="monitor-opt-label">{opt.label}</div>
              <div className="monitor-opt-note">{opt.note}</div>
            </div>
          ))}
        </div>
      </div>

      {/* Window options */}
      <div className="section-label">Window</div>
      <div className="card">
        <Toggle
          label="Fullscreen on launch"
          value={p.fullscreen}
          onChange={v => onChange(set(p, 'fullscreen', v))}
        />
        <Toggle
          label="Show screen borders"
          value={p.borders}
          onChange={v => onChange(set(p, 'borders', v))}
          note="Enables overscan demo effects on ST / STE"
        />
        <Toggle
          label="Double pixels (low resolution)"
          value={p.doublePixels}
          onChange={v => onChange(set(p, 'doublePixels', v))}
          note="Scales ST-low resolution to a more comfortable size"
        />
        <Toggle
          label="Aspect ratio correction"
          value={p.aspectRatio}
          onChange={v => onChange(set(p, 'aspectRatio', v))}
          note="Corrects monitor aspect ratio (Falcon / TT)"
        />
      </div>

    </div>
  );
}