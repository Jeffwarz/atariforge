import { Profile, SoundFreq } from '../types/profile';

interface Props {
  profile:  Profile;
  onChange: (p: Profile) => void;
}

function set<K extends keyof Profile>(p: Profile, key: K, val: Profile[K]): Profile {
  return { ...p, [key]: val };
}

interface ToggleProps {
  label:    string;
  value:    boolean;
  onChange: (v: boolean) => void;
  disabled?: boolean;
  note?:    string;
}

function Toggle({ label, value, onChange, disabled, note }: ToggleProps) {
  return (
    <div className={`toggle-row ${disabled ? 'disabled' : ''}`}>
      <div>
        <span className="toggle-label">{label}</span>
        {note && <div className="toggle-note">{note}</div>}
      </div>
      <button
        className={`toggle ${value ? 'on' : 'off'}`}
        onClick={() => !disabled && onChange(!value)}
        aria-pressed={value}
      />
    </div>
  );
}

const FREQ_GROUPS: { label: string; freqs: { value: SoundFreq; label: string }[] }[] = [
  {
    label: 'Standard',
    freqs: [
      { value: '11025', label: '11025 Hz' },
      { value: '22050', label: '22050 Hz' },
      { value: '44100', label: '44100 Hz — recommended' },
      { value: '48000', label: '48000 Hz' },
    ],
  },
  {
    label: 'STE / TT / Falcon DMA (best accuracy)',
    freqs: [
      { value: '6258',  label: '6258 Hz'  },
      { value: '12517', label: '12517 Hz' },
      { value: '25033', label: '25033 Hz' },
      { value: '50066', label: '50066 Hz' },
    ],
  },
];

export default function SoundTab({ profile: p, onChange }: Props) {
  return (
    <div className="tab-inner">

      {/* Audio toggles */}
      <div className="section-label">Audio</div>
      <div className="card">
        <Toggle
          label="Sound enabled"
          value={p.sound}
          onChange={v => onChange(set(p, 'sound', v))}
        />
        <Toggle
          label="YM2149 output"
          value={p.sound}
          onChange={v => onChange(set(p, 'sound', v))}
          note="PSG sound chip used in all Atari ST machines"
        />
        <Toggle
          label="Microphone input"
          value={false}
          onChange={() => {}}
          disabled={p.machine !== 'Falcon'}
          note="Falcon only"
        />
      </div>

      {/* Sample rate */}
      <div className="section-label">Sample rate</div>
      <div className="card">
        {FREQ_GROUPS.map(group => (
          <div key={group.label} className="freq-group">
            <div className="freq-group-label">{group.label}</div>
            <div className="freq-options">
              {group.freqs.map(f => (
                <div
                  key={f.value}
                  className={`freq-opt ${p.soundFreq === f.value ? 'sel' : ''}`}
                  onClick={() => onChange(set(p, 'soundFreq', f.value))}
                >
                  {f.label}
                </div>
              ))}
            </div>
          </div>
        ))}
        <div className="freq-note">
          For STE / TT / Falcon DMA sound, matching the DMA frequency avoids audio artefacts
        </div>
      </div>

    </div>
  );
}