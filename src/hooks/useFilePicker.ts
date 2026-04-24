import { open } from '@tauri-apps/plugin-dialog';

export async function pickFile(title: string, filters?: { name: string; extensions: string[] }[]): Promise<string | null> {
  try {
    const result = await open({
      title,
      multiple: false,
      directory: false,
      filters: filters ?? [],
    });
    if (typeof result === 'string') return result;
    if (Array.isArray(result) && (result as string[]).length > 0) return (result as string[])[0];
    return null;
  } catch {
    return null;
  }
}

export async function pickFolder(title: string): Promise<string | null> {
  try {
    const result = await open({
      title,
      multiple: false,
      directory: true,
    });
    if (typeof result === 'string') return result;
    return null;
  } catch {
    return null;
  }
}

// Preset filters for common Atari file types
export const DISK_IMAGE_FILTERS = [
  { name: 'Disk images', extensions: ['img', 'hd', 'vhd', 'st', 'msa', 'dim', 'stx', 'ipf'] },
  { name: 'All files', extensions: ['*'] },
];

export const FLOPPY_FILTERS = [
  { name: 'Floppy images', extensions: ['st', 'msa', 'dim', 'stx', 'ipf', 'zip', 'gz'] },
  { name: 'All files', extensions: ['*'] },
];

export const ROM_FILTERS = [
  { name: 'ROM images', extensions: ['img', 'rom', 'bin'] },
  { name: 'All files', extensions: ['*'] },
];