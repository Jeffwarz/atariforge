/// Reads and writes Hatari .cfg files.
///
/// Hatari config files are INI-style:
///   [Section]
///   Key = Value
///
/// We convert between that format and our Profile JSON structure.

use serde::{Deserialize, Serialize};

// ── Profile types (mirrors src/types/profile.ts) ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveEntry {
    pub id:  u32,
    pub img: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveConfig {
    pub acsi:   Vec<DriveEntry>,
    pub scsi:   Vec<DriveEntry>,
    pub ide:    Vec<DriveEntry>,
    pub gemdos: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id:           u64,
    pub name:         String,
    pub notes:        String,
    pub machine:      String,
    pub cpu:          String,
    pub ram:          String,
    pub ttram:        String,
    pub tos:          String,
    #[serde(rename = "tosPath")]
    pub tos_path:     String,
    pub monitor:      String,
    #[serde(rename = "floppyA")]
    pub floppy_a:     String,
    #[serde(rename = "floppyB")]
    pub floppy_b:     String,
    pub drives:       DriveConfig,
    #[serde(rename = "bootFromHd")]
    pub boot_from_hd: bool,
    pub blitter:      bool,
    pub rtc:          bool,
    pub dsp:          bool,
    #[serde(rename = "timerD")]
    pub timer_d:      bool,
    pub fullscreen:   bool,
    pub borders:      bool,
    #[serde(rename = "doublePixels")]
    pub double_pixels: bool,
    #[serde(rename = "aspectRatio")]
    pub aspect_ratio: bool,
    pub sound:        bool,
    #[serde(rename = "soundFreq")]
    pub sound_freq:   String,
    pub last:         String,
    #[serde(rename = "createdAt")]
    pub created_at:   u64,
}

// ── Hatari config generation ──────────────────────────────────────────────────

/// Convert a Profile into a Hatari .cfg file string.
pub fn profile_to_cfg(p: &Profile) -> String {
    let machine_str = match p.machine.as_str() {
        "ST"      => "st",
        "STE"     => "ste",
        "MegaSTE" => "megaste",
        "TT"      => "tt",
        "Falcon"  => "falcon",
        _         => "st",
    };

    let monitor_str = match p.monitor.as_str() {
        "Colour" => "rgb",
        "Mono"   => "mono",
        "VGA"    => "vga",
        "TV"     => "tv",
        _        => "rgb",
    };

    let memsize: u32 = match p.ram.as_str() {
        "512K" => 0,
        "1M"   => 1,
        "2M"   => 2,
        "4M"   => 4,
        "8M"   => 8,
        "10M"  => 10,
        "14M"  => 14,
        _      => 1,
    };

    let ttram_kb: u32 = match p.ttram.as_str() {
        "None"     => 0,
        "4 MiB"    => 4 * 1024,
        "8 MiB"    => 8 * 1024,
        "16 MiB"   => 16 * 1024,
        "32 MiB"   => 32 * 1024,
        "64 MiB"   => 64 * 1024,
        "128 MiB"  => 128 * 1024,
        "256 MiB"  => 256 * 1024,
        "512 MiB"  => 512 * 1024,
        "1024 MiB" => 1024 * 1024,
        _          => 0,
    };

    let cpu_clock: u32 = match p.cpu.as_str() {
        "8"  => 8,
        "16" => 16,
        "32" => 32,
        _    => 8,
    };

    let bool_str = |b: bool| if b { "TRUE" } else { "FALSE" };

    // ACSI — single [ACSI] section, numbered fields per slot
    // Hatari parses bUseDevice0/sDeviceFile0/nBlockSize0/nAcsiVersion0 etc.
    let mut acsi_lines = String::from("[ACSI]\n");
    for i in 0..8_u32 {
        let img = p.drives.acsi.iter().find(|d| d.id == i)
            .map(|d| d.img.clone())
            .unwrap_or_default();
        acsi_lines.push_str(&format!("bUseDevice{} = {}\n", i, bool_str(!img.is_empty())));
        acsi_lines.push_str(&format!("sDeviceFile{} = {}\n", i, img));
        acsi_lines.push_str(&format!("nBlockSize{} = 512\n", i));
        acsi_lines.push_str(&format!("nAcsiVersion{} = 1\n", i));
    }

    // SCSI — same pattern
    let mut scsi_lines = String::from("[SCSI]\n");
    for i in 0..8_u32 {
        let img = p.drives.scsi.iter().find(|d| d.id == i)
            .map(|d| d.img.clone())
            .unwrap_or_default();
        scsi_lines.push_str(&format!("bUseDevice{} = {}\n", i, bool_str(!img.is_empty())));
        scsi_lines.push_str(&format!("sDeviceFile{} = {}\n", i, img));
        scsi_lines.push_str(&format!("nBlockSize{} = 512\n", i));
        scsi_lines.push_str(&format!("nScsiVersion{} = 1\n", i));
    }

    // IDE — single [IDE] section, numbered fields for master (0) and slave (1)
    let ide_master = p.drives.ide.iter().find(|d| d.id == 0)
        .map(|d| d.img.clone()).unwrap_or_default();
    let ide_slave = p.drives.ide.iter().find(|d| d.id == 1)
        .map(|d| d.img.clone()).unwrap_or_default();
    let mut ide_lines = String::from("[IDE]\n");
    ide_lines.push_str(&format!("bUseDevice0 = {}\n", bool_str(!ide_master.is_empty())));
    ide_lines.push_str("nByteSwap0 = 2\n");
    ide_lines.push_str(&format!("sDeviceFile0 = {}\n", ide_master));
    ide_lines.push_str("nBlockSize0 = 512\n");
    ide_lines.push_str("nDeviceType0 = 0\n");
    ide_lines.push_str(&format!("bUseDevice1 = {}\n", bool_str(!ide_slave.is_empty())));
    ide_lines.push_str("nByteSwap1 = 2\n");
    ide_lines.push_str(&format!("sDeviceFile1 = {}\n", ide_slave));
    ide_lines.push_str("nBlockSize1 = 512\n");
    ide_lines.push_str("nDeviceType1 = 0\n");

    let sound_freq: u32 = p.sound_freq.parse().unwrap_or(44100);
    let tos_path = p.tos_path.clone();

    format!(
r#"[Log]
nTextLogLevel = 2
nAlertDlgLogLevel = 2
bConfirmQuit = FALSE

[Screen]
bFullScreen = {fullscreen}
nMonitorType = {monitor}
bShowBorders = {borders}
bZoomLowRes = {double_pixels}
bAspectCorrect = {aspect_ratio}

[Joystick0]
nJoystickMode = 0

[Keyboard]
bDisableKeyRepeat = FALSE

[Sound]
bEnableSound = {sound}
nPlaybackFreq = {sound_freq}

[Memory]
nMemorySize = {memsize}
TTRamSize_KB = {ttram_kb}

[Floppy]
bDriveAEnabled = TRUE
bDriveBEnabled = TRUE
szDiskAFileName = {floppy_a}
szDiskBFileName = {floppy_b}

[HardDisk]
bBootFromHardDisk = {boot_from_hd}
bUseHardDiskDirectories = {use_gemdos}
szHardDiskDirectories = {gemdos}

{ide}
{acsi}
{scsi}
[ROM]
szTosImageFileName = {tos}

[System]
nMachineType = {machine}
nCpuLevel = 0
nCpuFreq = {cpu_clock}
bBlitter = {blitter}
bRealTimeClock = {rtc}
bPatchTimerD = {timer_d}
bDSPEnabled = {dsp}
"#,
        fullscreen         = bool_str(p.fullscreen),
        monitor            = monitor_str,
        borders            = bool_str(p.borders),
        double_pixels      = bool_str(p.double_pixels),
        aspect_ratio       = bool_str(p.aspect_ratio),
        sound              = bool_str(p.sound),
        sound_freq         = sound_freq,
        memsize            = memsize,
        ttram_kb           = ttram_kb,
        floppy_a           = p.floppy_a,
        floppy_b           = p.floppy_b,
        use_gemdos         = bool_str(!p.drives.gemdos.is_empty()),
        gemdos             = p.drives.gemdos,
        boot_from_hd       = bool_str(p.boot_from_hd),
        ide                = ide_lines,
        acsi               = acsi_lines,
        scsi               = scsi_lines,
        tos                = tos_path,
        machine            = machine_str,
        cpu_clock          = cpu_clock,
        blitter            = bool_str(p.blitter),
        rtc                = bool_str(p.rtc),
        timer_d            = bool_str(p.timer_d),
        dsp                = bool_str(p.dsp),
    )
}