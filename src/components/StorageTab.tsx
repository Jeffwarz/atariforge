import { Profile } from '../types/profile';
import { pickFile, pickFolder, DISK_IMAGE_FILTERS, FLOPPY_FILTERS } from '../hooks/useFilePicker';

interface Props {
  profile:  Profile;
  onChange: (p: Profile) => void;
}

type Bus = 'acsi' | 'scsi' | 'ide';

const BUS_LABEL: Record<Bus, string> = {
  acsi: 'ACSI',
  scsi: 'SCSI',
  ide:  'IDE',
};

const BUS_NOTE: Record<Bus, string> = {
  acsi: 'ST / STE / MegaSTE / TT — up to ID 7',
  scsi: 'TT / Falcon only',
  ide:  'Master / Slave',
};

const BUS_AVAILABLE: Record<Bus, Profile['machine'][]> = {
  acsi: ['ST', 'STE', 'MegaSTE', 'TT'],
  scsi: ['TT', 'Falcon'],
  ide:  ['ST', 'STE', 'MegaSTE', 'TT', 'Falcon'],
};

const BUS_MAX: Record<Bus, number> = { acsi: 8, scsi: 8, ide: 2 };

export default function StorageTab({ profile: p, onChange }: Props) {

  function setFloppy(drive: 'floppyA' | 'floppyB', val: string) {
    onChange({ ...p, [drive]: val });
  }

  function setGemdos(val: string) {
    onChange({ ...p, drives: { ...p.drives, gemdos: val } });
  }

  function addDrive(bus: Bus) {
    const arr = p.drives[bus];
    if (arr.length >= BUS_MAX[bus]) return;
    const nextId = arr.length > 0 ? Math.max(...arr.map(d => d.id)) + 1 : 0;
    onChange({ ...p, drives: { ...p.drives, [bus]: [...arr, { id: nextId, img: '' }] } });
  }

  function removeDrive(bus: Bus, id: number) {
    onChange({
      ...p,
      drives: { ...p.drives, [bus]: p.drives[bus].filter(d => d.id !== id) },
    });
  }

  function updateDriveImg(bus: Bus, id: number, img: string) {
    onChange({
      ...p,
      drives: {
        ...p.drives,
        [bus]: p.drives[bus].map(d => d.id === id ? { ...d, img } : d),
      },
    });
  }

  async function browseFloppy(drive: 'floppyA' | 'floppyB') {
    const path = await pickFile('Select floppy image', FLOPPY_FILTERS);
    if (path) setFloppy(drive, path);
  }

  async function browseDrive(bus: Bus, id: number) {
    const path = await pickFile(`Select ${BUS_LABEL[bus]} disk image`, DISK_IMAGE_FILTERS);
    if (path) updateDriveImg(bus, id, path);
  }

  async function browseGemdos() {
    const path = await pickFolder('Select GEMDOS folder');
    if (path) setGemdos(path);
  }

  const available = (bus: Bus) => BUS_AVAILABLE[bus].includes(p.machine);

  return (
    <div className="tab-inner">

      {/* Boot options */}
      <div className="section-label">Boot options</div>
      <div className="card">
        <div className="toggle-row" style={{ borderBottom: 'none', padding: '4px 0' }}>
          <div>
            <span className="toggle-label">Boot from hard disk</span>
            <div className="toggle-note">
              Boot directly from an ACSI, SCSI, IDE or GEMDOS drive instead of floppy A:
            </div>
          </div>
          <button
            className={`toggle ${p.bootFromHd ? 'on' : 'off'}`}
            onClick={() => onChange({ ...p, bootFromHd: !p.bootFromHd })}
            aria-pressed={p.bootFromHd}
          />
        </div>
      </div>

      {/* Floppy */}
      <div className="section-label">Floppy drives</div>
      <div className="card">
        <FloppyRow
          label="A:"
          value={p.floppyA}
          onBrowse={() => browseFloppy('floppyA')}
          onEject={() => setFloppy('floppyA', '')}
        />
        <FloppyRow
          label="B:"
          value={p.floppyB}
          onBrowse={() => browseFloppy('floppyB')}
          onEject={() => setFloppy('floppyB', '')}
        />
      </div>

      {/* ACSI / SCSI / IDE */}
      {(['acsi', 'scsi', 'ide'] as Bus[]).map(bus => (
        <div key={bus}>
          <div className="section-label">
            {BUS_LABEL[bus]} drives
            <span className="section-note"> — {BUS_NOTE[bus]}</span>
          </div>
          <div className="card">
            {!available(bus) ? (
              <div className="na-note">
                {BUS_LABEL[bus]} not available on {p.machine === 'MegaSTE' ? 'Mega STE' : p.machine}
              </div>
            ) : (
              <>
                {p.drives[bus].length === 0 && (
                  <div className="na-note">No {BUS_LABEL[bus]} drives configured</div>
                )}
                {p.drives[bus].map((d, i) => (
                  <DriveRow
                    key={d.id}
                    label={bus === 'ide' ? (i === 0 ? 'Master' : 'Slave') : `ID ${d.id}`}
                    img={d.img}
                    onBrowse={() => browseDrive(bus, d.id)}
                    onRemove={() => removeDrive(bus, d.id)}
                  />
                ))}
                {p.drives[bus].length < BUS_MAX[bus] && (
                  <div className="add-drive-row">
                    <button className="btn-sm" onClick={() => addDrive(bus)}>
                      + Add {BUS_LABEL[bus]} drive
                    </button>
                  </div>
                )}
              </>
            )}
          </div>
        </div>
      ))}

      {/* GEMDOS */}
      <div className="section-label">
        GEMDOS drive
        <span className="section-note"> — maps a host folder as C:</span>
      </div>
      <div className="card">
        <div className="floppy-row">
          <span className="drive-label" style={{ width: 'auto', marginRight: 8 }}>Path</span>
          <span className={`floppy-path ${p.drives.gemdos ? '' : 'empty'}`}>
            {p.drives.gemdos || 'No folder selected'}
          </span>
          <button className="btn-sm" onClick={browseGemdos}>Browse</button>
          {p.drives.gemdos && (
            <button className="btn-sm danger" onClick={() => setGemdos('')}>Clear</button>
          )}
        </div>
      </div>

    </div>
  );
}

// ─── Floppy row ───────────────────────────────────────────────────────────────

function FloppyRow({ label, value, onBrowse, onEject }: {
  label: string; value: string; onBrowse: () => void; onEject: () => void;
}) {
  return (
    <div className="floppy-row">
      <span className="drive-label">{label}</span>
      <span className={`floppy-path ${value ? '' : 'empty'}`}>
        {value || 'No image'}
      </span>
      <button className="btn-sm" onClick={onBrowse}>Browse</button>
      {value && (
        <button className="btn-sm danger" onClick={onEject}>Eject</button>
      )}
    </div>
  );
}

// ─── Drive row ────────────────────────────────────────────────────────────────

function DriveRow({ label, img, onBrowse, onRemove }: {
  label: string; img: string; onBrowse: () => void; onRemove: () => void;
}) {
  return (
    <div className="drive-row">
      <span className="drive-id">{label}</span>
      <span className={`drive-img ${img ? '' : 'empty'}`}>
        {img || 'Empty slot'}
      </span>
      <button className="btn-sm" onClick={onBrowse}>Browse</button>
      <button className="btn-sm danger" onClick={onRemove}>Remove</button>
    </div>
  );
}