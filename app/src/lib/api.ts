import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface Patch {
  id: number;
  /** `U1:05` style, or null for an edit buffer dump. */
  location: string | null;
  name: string;
  layerB: string;
}

export interface Ports {
  inputs: string[];
  outputs: string[];
}

export interface Progress {
  label: string;
  done: number;
  total: number;
}

export type ThemeId = string;
export type FontId = string;

export interface AuditionSettings {
  play_note: boolean;
  /** MIDI note number; 60 is C3, the Prophet 12's middle C. */
  note: number;
  length_ms: number;
  /** 1-16, or 0 for the channel set on the synth. */
  channel: number;
}

export interface Settings {
  zoom: number;
  font: FontId;
  theme: ThemeId;
  audition: AuditionSettings;
}

/** Note name as the Prophet 12 shows it: C3 is middle C (MIDI note 60). */
export function noteName(note: number): string {
  const names = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
  return `${names[note % 12]}${Math.floor(note / 12) - 2}`;
}

export interface AppearanceOptions {
  themes: { id: ThemeId; name: string }[];
  fonts: { id: FontId; name: string }[];
}

export interface WriteRequest {
  id: number;
  target: string;
}

export interface RenameRequest {
  id: number;
  name: string;
}

export const api = {
  listPorts: () => invoke<Ports>('list_ports'),
  connect: (input: number, output: number) => invoke<number>('connect', { input, output }),
  disconnect: () => invoke<void>('disconnect'),
  readPrograms: (selection: string) =>
    invoke<{ patches: Patch[]; complete: boolean }>('read_programs', { selection }),
  cancel: () => invoke<void>('cancel'),
  openSyx: (path: string) => invoke<{ patches: Patch[]; skipped: number }>('open_syx', { path }),
  saveSyx: (ids: number[], path: string) => invoke<void>('save_syx', { ids, path }),
  saveSyxSplit: (ids: number[], dir: string) => invoke<void>('save_syx_split', { ids, dir }),
  audition: (id: number) => invoke<void>('audition', { id }),
  writePrograms: (writes: WriteRequest[]) =>
    invoke<{ backupPath: string; written: Patch[] }>('write_programs', { writes }),
  cleanifyNames: (names: string[]) => invoke<string[]>('cleanify_names', { names }),
  renamePrograms: (renames: RenameRequest[]) =>
    invoke<{ backupPath: string | null; patches: Patch[] }>('rename_programs', { renames }),
  hasInitTemplate: () => invoke<boolean>('has_init_template'),
  captureInitTemplate: () => invoke<string>('capture_init_template'),
  initPrograms: (locations: string[]) =>
    invoke<{ backupPath: string; written: Patch[] }>('init_programs', { locations }),
  copyToLayerB: (id: number, target: string) =>
    invoke<{ backupPath: string; written: Patch[] }>('copy_to_layer_b', { id, target }),
  backupDir: () => invoke<string>('backup_dir'),
  getSettings: () => invoke<Settings>('get_settings'),
  /** +1 zooms in, -1 out, 0 back to actual size. */
  zoom: (direction: number) => invoke<void>('zoom', { direction }),
  appearanceOptions: () => invoke<AppearanceOptions>('appearance_options'),
  setAppearance: (change: { theme?: ThemeId; font?: FontId }) => invoke<void>('set_appearance', change),
  setAudition: (audition: AuditionSettings) => invoke<void>('set_audition', { audition }),
  disclaimerAccepted: () => invoke<boolean>('disclaimer_accepted'),
  acceptDisclaimer: () => invoke<void>('accept_disclaimer'),
  quit: () => invoke<void>('quit'),
};

export function onProgress(handler: (p: Progress) => void): Promise<UnlistenFn> {
  return listen<Progress>('progress', (event) => handler(event.payload));
}

export const USER_BANKS = ['U1', 'U2', 'U3', 'U4'] as const;
export const FACTORY_BANKS = ['F1', 'F2', 'F3', 'F4'] as const;
export const PROGRAMS_PER_BANK = 99;

export function slot(bank: string, program: number): string {
  return `${bank}:${String(program).padStart(2, '0')}`;
}

export function bankSlots(bank: string): string[] {
  return Array.from({ length: PROGRAMS_PER_BANK }, (_, i) => slot(bank, i + 1));
}

/** The slot `offset` places after `start` in the same bank, or null past program 99. */
export function offsetSlot(start: string, offset: number): string | null {
  const [bank, program] = start.split(':');
  const n = Number(program) + offset;
  return n <= PROGRAMS_PER_BANK ? slot(bank, n) : null;
}

export function isUserSlot(location: string): boolean {
  return location.startsWith('U');
}

export const MAX_NAME_LENGTH = 20;

/** Why a name can't be stored on a Prophet 12 (manual p.82), or null if it can. */
export function nameProblem(name: string): string | null {
  if (name.length > MAX_NAME_LENGTH) return `Longer than ${MAX_NAME_LENGTH} characters`;
  const bad = [...name].find((c) => c < ' ' || c > '}' || '*/:<>?\\|'.includes(c));
  return bad === undefined ? null : `“${bad}” can't be used in a name`;
}

export function errorText(e: unknown): string {
  return typeof e === 'string' ? e : e instanceof Error ? e.message : JSON.stringify(e);
}

/** Lowercase search words from a query; empty when the query is blank. */
export function searchTerms(query: string): string[] {
  return query.toLowerCase().split(/\s+/).filter(Boolean);
}

/** True if every term occurs in the patch name or its slot. */
export function matchesSearch(terms: string[], name: string, location: string): boolean {
  const haystack = `${name} ${location}`.toLowerCase();
  return terms.every((t) => haystack.includes(t));
}

/** Splits `text` into parts, marking those that match any search term. */
export function highlightParts(text: string, terms: string[]): { text: string; match: boolean }[] {
  const lower = text.toLowerCase();
  const marked = new Array<boolean>(text.length).fill(false);
  for (const term of terms) {
    for (let i = lower.indexOf(term); i !== -1; i = lower.indexOf(term, i + 1)) {
      marked.fill(true, i, i + term.length);
    }
  }
  const parts: { text: string; match: boolean }[] = [];
  for (let i = 0; i < text.length; i++) {
    const last = parts.at(-1);
    if (last && last.match === marked[i]) last.text += text[i];
    else parts.push({ text: text[i], match: marked[i] });
  }
  return parts;
}
