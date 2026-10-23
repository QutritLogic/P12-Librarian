<script lang="ts">
  import { onMount } from 'svelte';
  import { SvelteMap } from 'svelte/reactivity';
  import { getVersion } from '@tauri-apps/api/app';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { ask, open, save } from '@tauri-apps/plugin-dialog';
  import AboutDialog from './lib/AboutDialog.svelte';
  import AppearanceDialog from './lib/AppearanceDialog.svelte';
  import PreferencesDialog from './lib/PreferencesDialog.svelte';
  import RestoreDialog from './lib/RestoreDialog.svelte';
  import RearrangeDialog from './lib/RearrangeDialog.svelte';
  import BankSheetDialog, { type SheetChoice } from './lib/BankSheetDialog.svelte';
  import DisclaimerDialog from './lib/DisclaimerDialog.svelte';
  import LicencesDialog from './lib/LicencesDialog.svelte';
  import ContextMenu, { type MenuItem } from './lib/ContextMenu.svelte';
  import InitDialog from './lib/InitDialog.svelte';
  import LayerBDialog from './lib/LayerBDialog.svelte';
  import PatchTable, { type DropInfo, type DropZone, type Move, type Row } from './lib/PatchTable.svelte';
  import RenameDialog from './lib/RenameDialog.svelte';
  import RenumberDialog from './lib/RenumberDialog.svelte';
  import SearchBox from './lib/SearchBox.svelte';
  import TagsDialog from './lib/TagsDialog.svelte';
  import WriteDialog from './lib/WriteDialog.svelte';
  import {
    FACTORY_BANKS,
    PROGRAMS_PER_BANK,
    USER_BANKS,
    api,
    bankSlots,
    errorText,
    isUserSlot,
    matchesSearch,
    matchesTags,
    movePlan,
    noteName,
    onProgress,
    searchFilters,
    searchTags,
    searchTerms,
    swapPlan,
    type Rearrange,
    type SearchFilter,
    type Patch,
    type Ports,
    type Progress,
    type RenameRequest,
    type RestorePoint,
    type AuditionSettings,
    type Settings,
    type Sheet,
    type TagChange,
    type TagEntry,
    type WriteRequest,
  } from './lib/api';

  // Connection
  let ports = $state<Ports>({ inputs: [], outputs: [] });
  let inputIndex = $state(-1);
  let outputIndex = $state(-1);
  let connected = $state(false);
  /** The synth's OS version while connected, e.g. "1.4.2". */
  let osVersion = $state('');
  /** The port names connected to, to notice when they disappear. */
  let connectedPorts: { input: string; output: string } | null = null;
  /** Connect by itself when a Prophet 12 appears; off after Disconnect, on after Connect. */
  let autoConnect = true;
  /** The Prophet 12 pane's share of the width; see the splitter. */
  let split = $state(0.5);
  let mainEl = $state<HTMLElement>();

  // Tags and favourites, by sound (Patch.fingerprint), shared by both panes.
  const tagStore = new SvelteMap<string, TagEntry>();
  const tagsOf = (p: Patch | undefined) => (p ? (tagStore.get(p.fingerprint)?.tags ?? []) : []);
  const isFavourite = (p: Patch | undefined) => !!p && !!tagStore.get(p.fingerprint)?.favourite;
  /** Every tag in use, sorted. */
  const knownTags = $derived(
    [...new Set([...tagStore.values()].flatMap((e) => e.tags ?? []))].sort((a, b) =>
      a.localeCompare(b, undefined, { sensitivity: 'base' }),
    ),
  );

  function setTags(all: Record<string, TagEntry>) {
    tagStore.clear();
    for (const [fingerprint, entry] of Object.entries(all)) tagStore.set(fingerprint, entry);
  }

  // Synth side: patches read from the synth, keyed by slot.
  const synthPatches = new SvelteMap<string, Patch>();
  let bank = $state('U1');
  let synthSelected = $state<string[]>([]);

  // File side
  let filePath = $state<string | null>(null);
  let filePatches = $state<Patch[]>([]);
  let fileSelected = $state<string[]>([]);

  // Activity
  let progress = $state<Progress | null>(null);
  let busy = $state<string | null>(null);
  let cancellable = $state(false);
  /** The status bar message. `undo` is the backup that undoes the change just made. */
  let message = $state<{ text: string; error: boolean; undo?: string } | null>(null);
  let backupDir = $state('');
  let writing = $state<{ patches: Patch[]; start: string | null } | null>(null);
  let dialog = $state<'about' | 'disclaimer' | 'licences' | 'appearance' | 'preferences' | 'restore' | 'bankSheet' | null>(null);
  /** True until the user accepts the disclaimer; nothing touches the synth before that. */
  let mustAccept = $state(false);
  let renaming = $state<{ pane: Pane; title: string; patches: Patch[]; proposed: string[] } | null>(null);
  let initializing = $state<string[] | null>(null);
  let layerCopy = $state<{ source: Patch; target: string } | null>(null);
  /** A drop within the Prophet 12 pane, waiting for the user to choose swap or move. */
  let rearranging = $state<{
    sources: string[];
    target: string;
    swap: [string, string][] | string;
    move: [string, string][] | string;
  } | null>(null);
  /** What the rearrange dialog offers first: the choice made last time. */
  let rearrangeMode = $state<Rearrange>('swap');
  /** Patches whose tags are being edited. */
  let tagging = $state<Patch[] | null>(null);
  let renumbering = $state<{ patches: Patch[]; selectionOnly: boolean } | null>(null);
  /** The File pane's list has changes that aren't saved to its file yet. */
  let fileDirty = $state(false);
  /** Show the Compare column in both panes. */
  let comparing = $state(false);
  let contextMenu = $state<{ x: number; y: number; items: (MenuItem | null)[] } | null>(null);

  type Pane = 'synth' | 'file';
  /** The pane last clicked, for keyboard shortcuts such as F2. */
  let activePane = $state<Pane>('synth');
  let synthTable = $state<PatchTable>();
  let fileTable = $state<PatchTable>();

  // Tooltip suffixes explaining why a button is disabled.
  const OFFLINE = '\n\nConnect to the Prophet 12 first.';
  const offline = $derived(connected ? '' : OFFLINE);
  const needOne = (n: number) => (n === 1 ? '' : '\n\nSelect exactly one patch first.');
  const needSome = (n: number) => (n > 0 ? '' : '\n\nSelect one or more patches first.');

  // Search
  const ALL_BANKS = [...USER_BANKS, ...FACTORY_BANKS];
  let synthSearch = $state('');
  let fileSearch = $state('');
  let synthSearchInput = $state<HTMLInputElement>();
  let fileSearchInput = $state<HTMLInputElement>();
  const synthTerms = $derived(searchTerms(synthSearch));
  const fileTerms = $derived(searchTerms(fileSearch));
  const synthTagTerms = $derived(searchTags(synthSearch));
  const fileTagTerms = $derived(searchTags(fileSearch));
  /** Show only favourites (the ★ button by each search box). */
  let synthFavOnly = $state(false);
  let fileFavOnly = $state(false);
  const synthFilters = $derived(searchFilters(synthSearch));
  const fileFilters = $derived(searchFilters(fileSearch));
  const synthSearching = $derived(
    synthTerms.length > 0 || synthTagTerms.length > 0 || synthFilters.length > 0 || synthFavOnly,
  );
  const fileFiltering = $derived(
    fileTerms.length > 0 || fileTagTerms.length > 0 || fileFilters.length > 0 || fileFavOnly,
  );

  // Compare and duplicates. Patches are matched by sound (fingerprint), which
  // leaves out the Layer A name, so a renamed copy still counts as the same sound.
  const synthByFingerprint = $derived.by(() => {
    const found = new Map<string, string[]>();
    for (const [location, p] of synthPatches) found.set(p.fingerprint, [...(found.get(p.fingerprint) ?? []), location]);
    for (const list of found.values()) list.sort();
    return found;
  });
  /** A file patch's label: its slot, or "edit n" for an edit buffer dump. */
  const fileLabel = (p: Patch, index: number) => p.location ?? `edit ${index + 1}`;
  const fileByFingerprint = $derived.by(() => {
    const found = new Map<string, { id: number; label: string }[]>();
    filePatches.forEach((p, i) =>
      found.set(p.fingerprint, [...(found.get(p.fingerprint) ?? []), { id: p.id, label: fileLabel(p, i) }]),
    );
    return found;
  });
  const fileBySlot = $derived.by(() => {
    const found = new Map<string, Patch>();
    for (const p of filePatches) if (p.location && !found.has(p.location)) found.set(p.location, p);
    return found;
  });
  const allBanksRead = $derived(synthPatches.size === ALL_BANKS.length * PROGRAMS_PER_BANK);

  /** "U1:05, U2:07" for a few places, or the first few and how many more. */
  function places(list: string[]): string {
    return list.length <= 3 ? list.join(', ') : `${list.slice(0, 3).join(', ')} and ${list.length - 3} more`;
  }

  /** How a synth slot compares with the File pane's list. */
  function synthCompare(location: string): Row['compare'] {
    const p = synthPatches.get(location);
    if (!p || filePatches.length === 0) return null;
    const inFile = (fileByFingerprint.get(p.fingerprint) ?? []).map((f) => f.label);
    const there = fileBySlot.get(location);
    if (there?.fingerprint === p.fingerprint) {
      return there.name === p.name
        ? { text: 'same', kind: 'same' }
        : { text: `same sound, named “${there.name}” in the file`, kind: 'renamed' };
    }
    if (there) {
      return { text: inFile.length ? `differs; in the file at ${places(inFile)}` : 'differs from the file', kind: 'differs' };
    }
    return inFile.length
      ? { text: `in the file at ${places(inFile)}`, kind: 'elsewhere' }
      : { text: 'not in the file', kind: 'missing' };
  }

  /** How a patch in the File pane compares with what has been read from the synth. */
  function fileCompare(p: Patch): Row['compare'] {
    const slot = p.location;
    const at = slot ? synthPatches.get(slot) : undefined;
    const elsewhere = (synthByFingerprint.get(p.fingerprint) ?? []).filter((l) => l !== slot);
    if (at?.fingerprint === p.fingerprint) {
      return at.name === p.name
        ? { text: 'same', kind: 'same' }
        : { text: `same sound, named “${at.name}” on the synth`, kind: 'renamed' };
    }
    if (at) {
      return {
        text: elsewhere.length ? `differs; on the synth at ${places(elsewhere)}` : 'differs from the synth',
        kind: 'differs',
      };
    }
    if (elsewhere.length) return { text: `on the synth at ${places(elsewhere)}`, kind: 'elsewhere' };
    if (slot && synthPatches.size > 0) return { text: 'slot not read', kind: 'unknown' };
    return allBanksRead
      ? { text: 'not on the synth', kind: 'missing' }
      : { text: 'not in the banks read', kind: 'missing' };
  }

  function synthDuplicates(location: string, p: Patch): string[] {
    return (synthByFingerprint.get(p.fingerprint) ?? []).filter((l) => l !== location);
  }

  function fileDuplicates(p: Patch): string[] {
    return (fileByFingerprint.get(p.fingerprint) ?? []).filter((f) => f.id !== p.id).map((f) => f.label);
  }

  /** The is:dup, is:same, is:differs and is:missing search words. */
  function passesFilters(filters: SearchFilter[], duplicates: string[], compare: Row['compare']): boolean {
    return filters.every((f) =>
      f === 'dup'
        ? duplicates.length > 0
        : f === 'same'
          ? compare?.kind === 'same' || compare?.kind === 'renamed'
          : compare?.kind === f,
    );
  }

  /** True if a patch passes a pane's search: name words, #tags and the ★ filter. */
  function passes(p: Patch, location: string, terms: string[], tagTerms: string[], favOnly: boolean): boolean {
    return matchesSearch(terms, p.name, location) && matchesTags(tagTerms, tagsOf(p)) && (!favOnly || isFavourite(p));
  }

  function synthRow(location: string): Row {
    const patch = synthPatches.get(location);
    return {
      key: location,
      location,
      name: patch?.name ?? '',
      layerB: patch?.layerB ?? '',
      id: patch?.id ?? null,
      tags: tagsOf(patch),
      favourite: isFavourite(patch),
      compare: comparing ? synthCompare(location) : null,
      duplicates: patch ? synthDuplicates(location, patch) : [],
    };
  }

  function synthMatches(location: string): boolean {
    const patch = synthPatches.get(location);
    return (
      patch !== undefined &&
      passes(patch, location, synthTerms, synthTagTerms, synthFavOnly) &&
      passesFilters(synthFilters, synthDuplicates(location, patch), synthCompare(location))
    );
  }

  /** The open bank, or while searching, the matching patches from every bank that has been read. */
  const synthRows: Row[] = $derived(
    synthSearching
      ? ALL_BANKS.flatMap(bankSlots).filter(synthMatches).map(synthRow)
      : bankSlots(bank).map(synthRow),
  );

  const fileRows: Row[] = $derived(
    filePatches
      .map((p, i) => ({ p, location: fileLabel(p, i), duplicates: fileDuplicates(p), compare: fileCompare(p) }))
      .filter(
        ({ p, location, duplicates, compare }) =>
          passes(p, location, fileTerms, fileTagTerms, fileFavOnly) && passesFilters(fileFilters, duplicates, compare),
      )
      .map(({ p, location, duplicates, compare }) => ({
        key: String(p.id),
        location,
        name: p.name,
        layerB: p.layerB,
        id: p.id,
        tags: tagsOf(p),
        favourite: isFavourite(p),
        compare: comparing ? compare : null,
        duplicates,
      })),
  );

  // Only patches that are visible count as selected, so a search never
  // leaves hidden patches to be acted on.
  const visibleSynth = $derived(new Set(synthRows.map((r) => r.key)));
  const visibleFile = $derived(new Set(fileRows.map((r) => r.key)));
  const selectedSynth = $derived(
    synthSelected
      .filter((k) => visibleSynth.has(k))
      .map((k) => synthPatches.get(k))
      .filter((p) => p !== undefined),
  );
  /** The File pane's rows in the order shown, which a sorted column changes. */
  let fileShownKeys = $state<string[]>([]);
  /** The selected file patches in the order shown, so writing and saving follow a sorted list. */
  const selectedFile = $derived.by(() => {
    const byKey = new Map(filePatches.map((p) => [String(p.id), p]));
    const chosen = new Set(fileSelected);
    return fileShownKeys
      .filter((k) => chosen.has(k) && visibleFile.has(k))
      .map((k) => byKey.get(k))
      .filter((p) => p !== undefined);
  });
  const fileName = $derived(filePath?.split(/[\\/]/).pop() ?? null);

  /** "40 same · 3 differ · 11 not on the synth", for the File pane when comparing. */
  const compareSummary = $derived.by(() => {
    const kinds = filePatches.map((p) => fileCompare(p)?.kind);
    const n = (k: string) => kinds.filter((x) => x === k).length;
    const parts = [
      [n('same') + n('renamed'), 'same'],
      [n('differs'), 'differ'],
      [n('elsewhere'), 'elsewhere on the synth'],
      [n('missing'), allBanksRead ? 'not on the synth' : 'not in the banks read'],
      [n('unknown'), 'slot not read'],
    ] as const;
    return parts.filter(([count]) => count > 0).map(([count, what]) => `${count} ${what}`).join(' · ');
  });

  function readCount(bankName: string): number {
    return bankSlots(bankName).filter((s) => synthPatches.has(s)).length;
  }

  function matchCount(bankName: string): number {
    return bankSlots(bankName).filter(synthMatches).length;
  }

  function tabTitle(b: string): string {
    const kind = isUserSlot(b) ? `User bank ${b.slice(1)}` : `Factory bank ${b.slice(1)}, read-only`;
    if (synthSearching) return `${kind}: ${matchCount(b)} matching. Click to clear the search and open the bank.`;
    return readCount(b) ? `${kind} (${readCount(b)} of 99 read)` : kind;
  }

  const synthEmptyText = $derived.by(() => {
    const read = synthPatches.size;
    if (read === 0) return 'Read a bank to search it.';
    const notAll = read < 792 ? ` Only the ${read} patches read so far are searched.` : '';
    return `${noMatch(synthSearch, synthFavOnly)}${notAll}`;
  });

  /** The empty-list text for a search and the ★ filter. */
  function noMatch(search: string, favOnly: boolean): string {
    const query = search.trim();
    if (!query) return 'No favourites here yet. Click ☆ next to a patch, or press F, to mark one.';
    return `No ${favOnly ? 'favourites' : 'patches'} match “${query}”.`;
  }

  /** Marks the patches as favourites, or unmarks them if they all are already. */
  async function toggleFavourite(patches: Patch[]) {
    if (patches.length === 0) return;
    const favourite = !patches.every(isFavourite);
    try {
      setTags(await api.updateTags(patches.map((p) => ({ fingerprint: p.fingerprint, name: p.name, favourite }))));
    } catch (e) {
      message = { text: errorText(e), error: true };
    }
  }

  async function saveTags(patches: Patch[], add: string[], remove: string[]) {
    tagging = null;
    if (add.length === 0 && remove.length === 0) return;
    try {
      setTags(await api.updateTags(patches.map((p) => ({ fingerprint: p.fingerprint, name: p.name, add, remove }))));
      info(patches.length === 1 ? `Saved the tags for “${patches[0].name}”.` : `Saved the tags for ${patches.length} patches.`);
    } catch (e) {
      message = { text: errorText(e), error: true };
    }
  }

  /** Clicking a tag on a row shows the pane's patches with that tag. */
  function showTag(pane: Pane, tag: string) {
    if (pane === 'synth') synthSearch = `#${tag}`;
    else fileSearch = `#${tag}`;
    activePane = pane;
  }

  function info(text: string) {
    message = { text, error: false };
  }

  /** Reports a change on the synth, offering to undo it from its backup. */
  function changed(text: string, backupPath: string) {
    message = { text: `${text} The previous contents are kept as a restore point.`, error: false, undo: backupPath };
  }

  /** Writes a restore point back to the synth: Undo, or Restore in the Restore points dialog. */
  function restoreFrom(path: string, what: string) {
    dialog = null;
    return run('Restoring', async () => {
      const report = await api.restore(path);
      for (const p of report.written) synthPatches.set(p.location!, p);
      const count = `${report.written.length} ${report.written.length === 1 ? 'slot' : 'slots'}`;
      changed(`${what}: restored and verified ${count}.`, report.backupPath);
    });
  }

  function undo() {
    if (message?.undo && connected && !busy) restoreFrom(message.undo, 'Undone');
  }

  /** Runs one operation at a time and turns errors into the status message. */
  async function run(label: string, job: () => Promise<void>, canCancel = false) {
    if (busy) return;
    busy = label;
    cancellable = canCancel;
    progress = null;
    message = null;
    try {
      await job();
    } catch (e) {
      message = { text: errorText(e), error: true };
    } finally {
      busy = null;
      progress = null;
    }
  }

  /** Port names last seen, to act only when something changes. */
  let seenPorts = '';

  /** Port names change index when devices come and go, so the choice is kept by name. */
  function applyPorts(next: Ports) {
    const inputName = ports.inputs[inputIndex];
    const outputName = ports.outputs[outputIndex];
    ports = next;
    const find = (names: string[], keep: string | undefined) => {
      const kept = keep === undefined ? -1 : names.indexOf(keep);
      return kept >= 0 ? kept : names.findIndex((n) => n.toLowerCase().includes('prophet 12'));
    };
    inputIndex = find(ports.inputs, inputName);
    outputIndex = find(ports.outputs, outputName);
  }

  async function refreshPorts() {
    const next = await api.listPorts();
    seenPorts = JSON.stringify(next);
    applyPorts(next);
  }

  function connect(quiet = false) {
    return run('Connecting', async () => {
      connected = false;
      osVersion = await api.connect(inputIndex, outputIndex);
      connected = true;
      autoConnect = true;
      connectedPorts = { input: ports.inputs[inputIndex], output: ports.outputs[outputIndex] };
      info(quiet ? 'The Prophet 12 is back; connected again.' : 'Connected to the Prophet 12.');
    });
  }

  async function disconnect() {
    await api.disconnect();
    connected = false;
    connectedPorts = null;
    autoConnect = false;
    info('Disconnected.');
  }

  /**
   * Every two seconds: notices when the connected synth is unplugged or switched off,
   * and connects again when a Prophet 12 appears (unless the user chose Disconnect).
   */
  async function watchPorts() {
    if (busy || mustAccept) return;
    let next: Ports;
    try {
      next = await api.listPorts();
    } catch {
      return;
    }
    const names = JSON.stringify(next);
    if (names === seenPorts) return;
    seenPorts = names;
    if (busy) return;
    if (connected && connectedPorts) {
      if (next.inputs.includes(connectedPorts.input) && next.outputs.includes(connectedPorts.output)) {
        applyPorts(next);
        return;
      }
      await api.disconnect();
      connected = false;
      connectedPorts = null;
      applyPorts(next);
      message = {
        text: 'The Prophet 12 is no longer there (unplugged or switched off). It connects again when it comes back.',
        error: true,
      };
      return;
    }
    applyPorts(next);
    const isP12 = (n: string | undefined) => !!n?.toLowerCase().includes('prophet 12');
    if (!connected && autoConnect && isP12(ports.inputs[inputIndex]) && isP12(ports.outputs[outputIndex])) {
      // A freshly plugged-in synth may need a moment before it answers.
      await new Promise((resolve) => setTimeout(resolve, 1500));
      if (!connected && !busy) await connect(true);
    }
  }

  /** "Prophet 12 Keyboard" or "Prophet 12 Module", from the port name where it says so. */
  const model = $derived.by(() => {
    const name = connectedPorts?.input.toLowerCase() ?? '';
    return name.includes('module') ? 'Prophet 12 Module' : name.includes('keyboard') ? 'Prophet 12 Keyboard' : 'Prophet 12';
  });

  /**
   * Reads programs from the synth. Returns whether all were read, the slots that
   * differ from what was read before, and the slots whose tags were kept: a slot
   * that held a tagged or favourite sound and now holds a different one was
   * changed on the synth (changes made in the app update synthPatches directly),
   * so the new sound gets the old one's tags and favourite mark.
   */
  /**
   * The tag change that gives `after` the tags and favourite mark of `before`,
   * when the same slot now holds a different sound; null if there's nothing to carry.
   */
  function carryTags(before: Patch | undefined, after: Patch): TagChange | null {
    if (!before || before.fingerprint === after.fingerprint) return null;
    const entry = tagStore.get(before.fingerprint);
    if (!entry || !(entry.tags?.length || entry.favourite)) return null;
    return {
      fingerprint: after.fingerprint,
      name: after.name,
      add: entry.tags ?? [],
      ...(entry.favourite ? { favourite: true } : {}),
    };
  }

  async function readPrograms(selection: string): Promise<{ complete: boolean; changed: string[]; kept: string[] }> {
    const result = await api.readPrograms(selection);
    const changed: string[] = [];
    const kept: string[] = [];
    const carry: TagChange[] = [];
    for (const p of result.patches) {
      const location = p.location!;
      const before = synthPatches.get(location);
      if (before && (before.fingerprint !== p.fingerprint || before.name !== p.name)) changed.push(location);
      const change = carryTags(before, p);
      if (change) {
        carry.push(change);
        kept.push(location);
      }
      synthPatches.set(location, p);
    }
    if (carry.length > 0) setTags(await api.updateTags(carry));
    return { complete: result.complete, changed, kept };
  }

  /** Slots as a short list, e.g. "U3:14, U4:02" or "5 slots". */
  function slotList(locations: string[]): string {
    return locations.length <= 3 ? locations.join(', ') : `${locations.length} slots`;
  }

  /** Status text for slots whose tags were kept after a change on the synth. */
  function keptNote(kept: string[]): string {
    if (kept.length === 0) return '';
    const whose = kept.length === 1 ? 'its' : 'their';
    return ` ${slotList(kept)} changed on the synth; ${whose} tags and favourite mark were kept.`;
  }

  function readBank() {
    return run(`Reading ${bank}`, async () => {
      const { complete, kept } = await readPrograms(bank);
      info((complete ? `Read bank ${bank}.` : 'Reading cancelled.') + keptNote(kept));
    }, true);
  }

  function readAllUser() {
    return run('Reading all User banks', async () => {
      const { complete, kept } = await readPrograms('user');
      info((complete ? 'Read all 4 User banks (396 programs).' : 'Reading cancelled.') + keptNote(kept));
    }, true);
  }

  /** Reads the selected slots again, e.g. after saving an edit on the synth. */
  function reRead(patches: Patch[]) {
    const locations = patches.map((p) => p.location!);
    if (!connected || locations.length === 0) return;
    return run('Re-reading', async () => {
      const { complete, changed, kept } = await readPrograms(locations.join(','));
      if (!complete) return info('Reading cancelled.');
      const head = `Re-read ${locations.length === 1 ? locations[0] : `${locations.length} slots`}`;
      if (changed.length === 0) return info(`${head}: no changes.`);
      const whose = changed.length === 1 ? 'its' : 'their';
      const tags =
        kept.length === 0
          ? ''
          : kept.length === changed.length
            ? `; ${whose} tags and favourite mark were kept`
            : `; the tags of ${slotList(kept)} were kept`;
      info(`${head}: ${changed.length === 1 && locations.length === 1 ? 'it' : slotList(changed)} had changed on the synth${tags}.`);
    }, true);
  }

  function timestamp(): string {
    const d = new Date();
    const pad = (n: number) => String(n).padStart(2, '0');
    return `${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}-${pad(d.getHours())}${pad(d.getMinutes())}${pad(d.getSeconds())}`;
  }

  const syxFilter = [{ name: 'SysEx', extensions: ['syx'] }];

  async function backUpAll() {
    const path = await save({ defaultPath: `${backupDir}/p12-backup-${timestamp()}.syx`, filters: syxFilter });
    if (!path) return;
    await run('Backing up all User banks', async () => {
      if (!(await readPrograms('user')).complete) {
        info('Backup cancelled; nothing was saved.');
        return;
      }
      const ids = USER_BANKS.flatMap((b) => bankSlots(b).map((s) => synthPatches.get(s)!.id));
      await api.saveSyx(ids, path);
      info(`Saved all 396 User programs to ${path}`);
    }, true);
  }

  /** Saves the chosen banks, or the File pane's list, as a PDF or CSV bank sheet. */
  async function exportSheet({ source, banks, format }: SheetChoice) {
    dialog = null;
    const now = new Date().toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' });
    const row = (slot: string, p: Patch) => ({
      slot,
      name: p.name,
      layerB: p.layerB,
      favourite: isFavourite(p),
      tags: tagsOf(p),
    });
    let sheet: Sheet;
    let suggested: string;
    if (source === 'synth') {
      const synth = connected ? `${model}, OS ${osVersion}` : 'Prophet 12';
      sheet = {
        title: 'Prophet 12',
        note: `${synth} · ${banks.join(', ')} · ${now}`,
        sections: banks.map((b) => ({ bank: b, rows: bankSlots(b).map((s) => row(s, synthPatches.get(s)!)) })),
      };
      suggested = `p12-bank-sheet-${banks.length === ALL_BANKS.length ? 'all' : banks.join('-')}-${timestamp().slice(0, 8)}`;
    } else {
      const name = fileName ?? 'New list';
      const unsaved = fileDirty ? ', with changes not saved yet' : '';
      sheet = {
        title: name,
        note: `${filePatches.length} patches${unsaved} · ${now}`,
        sections: [{ bank: '', rows: filePatches.map((p, i) => row(fileLabel(p, i), p)) }],
      };
      suggested = name.replace(/\.syx$/i, '');
    }
    const filter = format === 'pdf' ? { name: 'PDF', extensions: ['pdf'] } : { name: 'CSV', extensions: ['csv'] };
    let path = await save({ defaultPath: `${backupDir}/${suggested}.${format}`, filters: [filter] });
    if (!path) return;
    if (!path.toLowerCase().endsWith(`.${format}`)) path += `.${format}`;
    await run('Exporting', async () => {
      await api.exportSheet(sheet, path!);
      info(`Saved the bank sheet to ${path}`);
    });
  }

  async function saveSelected(patches: Patch[]) {
    const path = await save({ defaultPath: `${backupDir}/p12-${timestamp()}.syx`, filters: syxFilter });
    if (!path) return;
    await run('Saving', async () => {
      await api.saveSyx(patches.map((p) => p.id), path);
      info(`Saved ${patches.length} ${patches.length === 1 ? 'patch' : 'patches'} to ${path}`);
    });
  }

  async function saveSeparately(patches: Patch[]) {
    const dir = await open({ directory: true, defaultPath: backupDir, title: 'Choose a folder' });
    if (!dir) return;
    await run('Saving', async () => {
      await api.saveSyxSplit(patches.map((p) => p.id), dir);
      info(`Saved ${patches.length} .syx files to ${dir}`);
    });
  }

  /** Asks before throwing away unsaved changes to the File pane's list. */
  async function discardChanges(): Promise<boolean> {
    if (!fileDirty) return true;
    return ask('The list in the File pane has changes that aren\'t saved. Discard them?', {
      title: 'Unsaved changes',
      kind: 'warning',
      okLabel: 'Discard',
      cancelLabel: 'Keep them',
    });
  }

  async function newList() {
    if (!(await discardChanges())) return;
    await api.closeFile();
    filePatches = [];
    filePath = null;
    fileSelected = [];
    fileDirty = false;
    info('New, empty list. Drag patches here from the Prophet 12 pane, or add a .syx file.');
  }

  async function openFile() {
    if (!(await discardChanges())) return;
    const path = await open({ filters: syxFilter, multiple: false });
    if (path) await openPath(path);
  }

  /** Adds a .syx file's patches to the end of the list. */
  async function addFile() {
    const path = await open({ filters: syxFilter, multiple: false });
    if (!path) return;
    await run('Adding', async () => {
      const result = await api.addSyx(path);
      filePatches = [...filePatches, ...result.patches];
      fileDirty = true;
      const name = path.split(/[\\/]/).pop();
      info(`Added ${result.patches.length} patches from ${name} to the list. Not saved yet.`);
    });
  }

  /** Automatic backups (restore points) are never overwritten by Save. */
  function isRestorePoint(path: string): boolean {
    return /[\\/]before-[^\\/]*\.syx$/.test(path);
  }

  async function saveList() {
    if (filePatches.length === 0) return info('The list is empty; there is nothing to save.');
    if (!filePath || isRestorePoint(filePath)) return saveListAs();
    const path = filePath;
    await run('Saving', async () => {
      await api.saveSyx(filePatches.map((p) => p.id), path);
      fileDirty = false;
      info(`Saved ${filePatches.length} patches to ${path}`);
      api.recentFile(path);
    });
  }

  async function saveListAs() {
    if (filePatches.length === 0) return info('The list is empty; there is nothing to save.');
    const suggested = filePath && !isRestorePoint(filePath) ? filePath : `${backupDir}/p12-list-${timestamp()}.syx`;
    const path = await save({ defaultPath: suggested, filters: syxFilter });
    if (!path) return;
    await run('Saving', async () => {
      await api.saveSyx(filePatches.map((p) => p.id), path);
      filePath = path;
      fileDirty = false;
      info(`Saved ${filePatches.length} patches to ${path}`);
      api.recentFile(path);
    });
  }

  async function removeFromList(patches: Patch[]) {
    if (patches.length === 0) return;
    const ids = patches.map((p) => p.id);
    await api.removeFromFile(ids);
    filePatches = filePatches.filter((p) => !ids.includes(p.id));
    fileSelected = [];
    fileDirty = true;
    const count = `${ids.length} ${ids.length === 1 ? 'patch' : 'patches'}`;
    info(`Removed ${count} from the list. The file on disk changes only when you save.`);
  }

  /** Index in the list to insert before: the patch with id `before`, or the end. */
  function insertIndex(before: number | null): number {
    const i = before === null ? -1 : filePatches.findIndex((p) => p.id === before);
    return i === -1 ? filePatches.length : i;
  }

  /** Copies patches (from the synth, or from the list itself) into the list. */
  async function addToList(ids: number[], before: number | null) {
    await run('Adding', async () => {
      const added = await api.copyToFile(ids);
      const at = insertIndex(before);
      filePatches = [...filePatches.slice(0, at), ...added, ...filePatches.slice(at)];
      fileSelected = added.map((p) => String(p.id));
      fileDirty = true;
      const count = `${added.length} ${added.length === 1 ? 'patch' : 'patches'}`;
      info(`Added ${count} to the list. Not saved yet.`);
    });
  }

  /** Moves patches within the list, keeping their order, before the patch with id `before`. */
  function moveInList(ids: number[], before: number | null) {
    if (before !== null && ids.includes(before)) return; // dropped onto itself
    const moving = filePatches.filter((p) => ids.includes(p.id));
    const rest = filePatches.filter((p) => !ids.includes(p.id));
    const i = before === null ? -1 : rest.findIndex((p) => p.id === before);
    const at = i === -1 ? rest.length : i;
    filePatches = [...rest.slice(0, at), ...moving, ...rest.slice(at)];
    fileDirty = true;
  }

  function dropOnFile(row: Row, ids: number[], _zone: DropZone, how: DropInfo) {
    if (how.sameTable && !how.copy) moveInList(ids, row.id);
    else addToList(ids, row.id);
  }

  function dropAtFileEnd(ids: number[], how: DropInfo) {
    if (fileFiltering) return;
    if (how.sameTable && !how.copy) moveInList(ids, null);
    else addToList(ids, null);
  }

  function startRenumber() {
    const selectionOnly = selectedFile.length > 1;
    const patches = selectionOnly ? filePatches.filter((p) => selectedFile.includes(p)) : filePatches;
    if (patches.length > 0) renumbering = { patches, selectionOnly };
  }

  async function confirmRenumber(changes: { id: number; location: string }[]) {
    renumbering = null;
    await run('Renumbering', async () => {
      const updated = new Map((await api.setFileLocations(changes)).map((p) => [p.id, p]));
      filePatches = filePatches.map((p) => updated.get(p.id) ?? p);
      fileDirty = true;
      info(`Renumbered ${changes.length} patches, ${changes[0].location} onwards. Not saved yet.`);
    });
  }

  async function openPath(path: string, fromRecent = false) {
    let opened = false;
    await run('Opening', async () => {
      const result = await api.openSyx(path);
      filePath = path;
      filePatches = result.patches;
      fileSelected = [];
      fileDirty = false;
      opened = true;
      const skipped = result.skipped ? ` (${result.skipped} other SysEx messages skipped)` : '';
      info(`Opened ${result.patches.length} patches${skipped}.`);
    });
    if (opened && !isRestorePoint(path)) api.recentFile(path);
    if (!opened && fromRecent && message) {
      // Moved, renamed or deleted since: it's no use in the menu.
      api.recentFile(path, true);
      message = { ...message, text: `${message.text} It was taken off File → Open recent.` };
    }
  }

  async function openRecent(path: string) {
    if (await discardChanges()) await openPath(path, true);
  }

  function audition(patch: Patch | undefined) {
    if (!patch) return;
    return run('Auditioning', async () => {
      await api.audition(patch.id);
      const note = auditionSettings.play_note ? `, playing ${noteName(auditionSettings.note)}` : '';
      info(`Loaded "${patch.name}" into the edit buffer${note}. Nothing was stored.`);
    });
  }

  function startWrite(patches: Patch[], start: string | null = null) {
    if (patches.length > 0) writing = { patches, start };
  }

  function confirmWrite(writes: WriteRequest[]) {
    writing = null;
    return run('Writing', async () => {
      const report = await api.writePrograms(writes);
      for (const p of report.written) synthPatches.set(p.location!, p);
      changed(`Wrote and verified ${report.written.length} patches.`, report.backupPath);
    });
  }

  /** A patch from either pane, by id. */
  function patchById(id: number): Patch | undefined {
    return filePatches.find((p) => p.id === id) ?? [...synthPatches.values()].find((p) => p.id === id);
  }

  function dropOnSynth(row: Row, ids: number[], zone: DropZone, info: DropInfo) {
    const patches = ids.map(patchById).filter((p) => p !== undefined);
    // Within the Prophet 12 pane a drop rearranges; Ctrl, or a Factory patch, copies.
    if (zone === 'row' && info.sameTable && !info.copy && patches.every((p) => isUserSlot(p.location!))) {
      startRearrange(patches, row.location);
    } else if (zone === 'row') {
      startWrite(patches, row.location);
    } else if (patches.length !== 1) {
      message = { text: 'Drop a single patch onto a Layer B.', error: true };
    } else {
      layerCopy = { source: patches[0], target: row.location };
    }
  }

  /** Asks whether to swap the dragged slots with those from the drop target, or move them there and shift. */
  function startRearrange(patches: Patch[], target: string) {
    const sources = patches.map((p) => p.location!);
    const swap = swapPlan(sources, target);
    const move = movePlan(sources, target);
    if (move.length === 0) return; // dropped back where it was
    if (typeof swap === 'string' && typeof move === 'string') {
      message = { text: rearrangeMode === 'swap' ? swap : move, error: true };
    } else {
      rearranging = { sources, target, swap, move };
    }
  }

  function confirmRearrange(mode: Rearrange, pairs: [string, string][]) {
    rearranging = null;
    if (mode !== rearrangeMode) api.setRearrange(mode);
    rearrangeMode = mode;
    if (mode === 'swap') {
      return run('Swapping', async () => {
        const report = await api.swapPrograms(pairs);
        for (const p of report.written) synthPatches.set(p.location!, p);
        const what = pairs.length === 1 ? `${pairs[0][0]} and ${pairs[0][1]}` : `${pairs.length} pairs of slots`;
        changed(`Swapped and verified ${what}.`, report.backupPath);
      });
    }
    return run('Moving', async () => {
      const report = await api.movePrograms(pairs);
      for (const p of report.written) synthPatches.set(p.location!, p);
      changed(`Moved and verified ${pairs.length} slots.`, report.backupPath);
    });
  }

  // Holding a dragged patch over a bank tab opens that bank, so patches can be dropped in another bank.
  let tabTimer: ReturnType<typeof setTimeout> | undefined;
  function dragOverTab(b: string | null) {
    clearTimeout(tabTimer);
    if (b && !synthSearching && b !== bank) tabTimer = setTimeout(() => (bank = b), 500);
  }

  function canDropOnSynth(row: Row, zone: DropZone): boolean {
    // While searching, a multi-patch drop could fill slots hidden by the search.
    return isUserSlot(row.location) && (zone === 'layerB' || !synthSearching);
  }

  function confirmLayerCopy() {
    const { source, target } = layerCopy!;
    layerCopy = null;
    return run('Copying into Layer B', async () => {
      const before = synthPatches.get(target);
      const report = await api.copyToLayerB(source.id, target);
      for (const p of report.written) synthPatches.set(p.location!, p);
      // The new Layer B changes the program's sound, and so its fingerprint; it's
      // still the same patch, so it keeps its tags and favourite mark.
      const after = report.written.find((p) => p.location === target);
      const change = after && carryTags(before, after);
      if (change) setTags(await api.updateTags([change]));
      changed(`Copied "${source.name}" into Layer B of ${target} and verified it.`, report.backupPath);
    });
  }

  function selectedIn(pane: Pane): Patch[] {
    return pane === 'synth' ? selectedSynth : selectedFile;
  }

  /** Why the selected patches in `pane` can't be renamed, or null if they can. */
  function renameBlocked(pane: Pane): string | null {
    const patches = selectedIn(pane);
    if (patches.length === 0) return 'Select one or more patches first.';
    if (pane === 'synth' && !connected) return 'Connect to the Prophet 12 first.';
    if (pane === 'synth' && patches.some((p) => !isUserSlot(p.location!)))
      return 'Factory banks are read-only. Copy the patch to a User slot to rename it.';
    return null;
  }

  function renameTip(pane: Pane, what: string): string {
    const blocked = renameBlocked(pane);
    return blocked ? `${what}\n\n${blocked}` : what;
  }

  function startRename(pane: Pane) {
    if (busy || renameBlocked(pane)) return;
    const patches = selectedIn(pane);
    const title = patches.length === 1 ? 'Rename patch' : `Rename ${patches.length} patches`;
    renaming = { pane, title, patches, proposed: patches.map((p) => p.name) };
  }

  async function startCleanify(pane: Pane) {
    if (busy || renameBlocked(pane)) return;
    const selected = selectedIn(pane);
    const cleaned = await api.cleanifyNames(selected.map((p) => p.name));
    const changes = selected.flatMap((p, i) => (cleaned[i] === p.name ? [] : [{ patch: p, name: cleaned[i] }]));
    if (changes.length === 0) {
      info(`The selected ${selected.length === 1 ? 'name is' : 'names are'} already clean.`);
      return;
    }
    renaming = {
      pane,
      title: `Cleanify ${changes.length} ${changes.length === 1 ? 'name' : 'names'}`,
      patches: changes.map((c) => c.patch),
      proposed: changes.map((c) => c.name),
    };
  }

  function confirmRename(renames: RenameRequest[]) {
    const pane = renaming!.pane;
    renaming = null;
    return run('Renaming', async () => {
      const report = await api.renamePrograms(renames);
      if (pane === 'synth') {
        for (const p of report.patches) synthPatches.set(p.location!, p);
      } else {
        const renamed = new Map(report.patches.map((p) => [p.id, p]));
        filePatches = filePatches.map((p) => renamed.get(p.id) ?? p);
        fileDirty = true;
      }
      const count = `${report.patches.length} ${report.patches.length === 1 ? 'patch' : 'patches'}`;
      if (report.backupPath) changed(`Renamed and verified ${count} on the synth.`, report.backupPath);
      else info(`Renamed ${count} in the list. Save it (Ctrl+S) to keep the new names.`);
    });
  }

  /** Why the selected synth patches can't be initialized, or null if they can. */
  function initBlocked(): string | null {
    if (selectedSynth.length === 0) return 'Select one or more slots first.';
    if (!connected) return 'Connect to the Prophet 12 first.';
    if (selectedSynth.some((p) => !isUserSlot(p.location!))) return 'Factory banks are read-only.';
    return null;
  }

  function confirmInit(locations: string[]) {
    initializing = null;
    return run('Initializing', async () => {
      const report = await api.initPrograms(locations);
      for (const p of report.written) synthPatches.set(p.location!, p);
      const count = `${report.written.length} ${report.written.length === 1 ? 'slot' : 'slots'}`;
      changed(`Initialized and verified ${count}.`, report.backupPath);
    });
  }

  function openContextMenu(pane: Pane, event: MouseEvent) {
    activePane = pane;
    // The table updates the selection before calling this; read it after that.
    queueMicrotask(() => {
      const patches = selectedIn(pane);
      const one = patches.length === 1;
      const items: (MenuItem | null)[] = [
        {
          label: 'Audition',
          action: () => audition(patches[0]),
          disabled: !connected || !one,
          title: !connected ? 'Connect to the Prophet 12 first' : one ? undefined : 'Select exactly one patch',
        },
        null,
        {
          label: one ? 'Rename…' : `Rename ${patches.length}…`,
          shortcut: 'F2',
          action: () => startRename(pane),
          disabled: renameBlocked(pane) !== null,
          title: renameBlocked(pane) ?? undefined,
        },
        {
          label: 'Cleanify names…',
          action: () => startCleanify(pane),
          disabled: renameBlocked(pane) !== null,
          title: renameBlocked(pane) ?? 'Tidy ALL-CAPS names, e.g. “BOF COOL PAD” → “Bof Cool Pad”, “DISTPAD” → “DistPad”',
        },
        ...(pane === 'synth'
          ? [
              {
                label: 'Re-read from synth',
                shortcut: 'R',
                action: () => reRead(patches),
                disabled: !connected || !!busy,
                title: connected
                  ? 'Read these slots again, e.g. after saving an edit on the synth'
                  : 'Connect to the Prophet 12 first',
              },
              {
                label: one ? 'Initialize…' : `Initialize ${patches.length}…`,
                action: () => (initializing = patches.map((p) => p.location!)),
                disabled: initBlocked() !== null,
                title: initBlocked() ?? 'Replace with the Basic Patch, named “Init Patch”',
              },
            ]
          : [
              {
                label: 'Renumber…',
                action: startRenumber,
                disabled: filePatches.length === 0,
                title: 'Give the list (or the selected patches) consecutive User slots, e.g. to plan a bank',
              },
              {
                label: one ? 'Remove from list' : `Remove ${patches.length} from list`,
                shortcut: 'Del',
                action: () => removeFromList(patches),
                disabled: patches.length === 0,
                title: 'Take them out of this list; the file on disk changes only when you save',
              },
            ]),
        null,
        {
          label: patches.length > 0 && patches.every(isFavourite) ? 'Remove from favourites' : 'Add to favourites',
          shortcut: 'F',
          action: () => toggleFavourite(patches),
          disabled: patches.length === 0,
        },
        {
          label: one ? 'Tags…' : `Tags for ${patches.length}…`,
          shortcut: 'T',
          action: () => (tagging = patches),
          disabled: patches.length === 0,
          title: 'Add or remove tags, to find patches with #tag in the search box',
        },
        null,
        { label: 'Save selected…', action: () => saveSelected(patches) },
        ...(pane === 'synth'
          ? [
              {
                label: 'Add to the File list',
                action: () => addToList(patches.map((p) => p.id), null),
                disabled: patches.length === 0,
                title: 'Copy them into the list in the File pane, to save or compare later',
              },
            ]
          : []),
        pane === 'synth'
          ? { label: 'Save as separate files…', action: () => saveSeparately(patches) }
          : {
              label: 'Write to synth…',
              action: () => startWrite(patches),
              disabled: !connected,
              title: connected ? undefined : 'Connect to the Prophet 12 first',
            },
      ];
      contextMenu = { x: event.clientX, y: event.clientY, items };
    });
  }

  let zoomLevel = 1;
  let appearance = $state({ theme: 'system', font: 'fira' });
  let auditionSettings = $state<AuditionSettings>({
    play_note: true,
    note: 60,
    length_ms: 1000,
    channel: 0,
    on_move: false,
  });

  function applySettings(settings: Settings) {
    appearance = { theme: settings.theme, font: settings.font };
    auditionSettings = settings.audition;
    rearrangeMode = settings.rearrange;
    if (!splitting) split = settings.split;
    document.documentElement.dataset.font = settings.font;
    document.documentElement.dataset.theme = settings.theme;
    if (settings.zoom !== zoomLevel) info(`Zoom ${Math.round(settings.zoom * 100)}%`);
    zoomLevel = settings.zoom;
  }

  /** Zoom keys, read from the typed character so they work on any keyboard layout. */
  function zoomKey(event: KeyboardEvent): boolean {
    if (!event.ctrlKey || event.altKey) return false;
    const direction = { '+': 1, '=': 1, '-': -1, '0': 0 }[event.key];
    if (direction === undefined) return false;
    event.preventDefault();
    api.zoom(direction);
    return true;
  }

  function wheel(event: WheelEvent) {
    if (!event.ctrlKey) return;
    event.preventDefault();
    api.zoom(event.deltaY < 0 ? 1 : -1);
  }

  function keydown(event: KeyboardEvent) {
    if (zoomKey(event)) return;
    if (mustAccept || renaming || renumbering || writing || initializing || layerCopy || rearranging || tagging || dialog || contextMenu)
      return;
    if (event.ctrlKey && event.key.toLowerCase() === 'f') {
      event.preventDefault();
      const input = activePane === 'synth' ? synthSearchInput : fileSearchInput;
      input?.focus();
      input?.select();
    } else if (event.key === 'F2') {
      event.preventDefault();
      startRename(activePane);
    } else if (event.key === 'ArrowDown' && (event.target === synthSearchInput || event.target === fileSearchInput)) {
      // From a search box straight into its results.
      event.preventDefault();
      activePane = event.target === synthSearchInput ? 'synth' : 'file';
      (event.target as HTMLInputElement).blur();
      (activePane === 'synth' ? synthTable : fileTable)?.move('first');
    } else if (!isTextField(event.target)) {
      if (navigationKey(event)) event.preventDefault();
    }
  }

  /** Keys that work on the active pane's list. Returns true if the key was used. */
  function navigationKey(event: KeyboardEvent): boolean {
    const table = activePane === 'synth' ? synthTable : fileTable;
    const selected = activePane === 'synth' ? selectedSynth : selectedFile;
    const moves: Record<string, Move | 'page-up' | 'page-down'> = {
      ArrowUp: -1,
      ArrowDown: 1,
      Home: 'first',
      End: 'last',
      PageUp: 'page-up',
      PageDown: 'page-down',
    };
    const key = event.key;
    if (event.ctrlKey && key.toLowerCase() === 'z') {
      undo();
    } else if (event.ctrlKey && (key === 'PageUp' || key === 'PageDown')) {
      switchBank(key === 'PageUp' ? -1 : 1);
    } else if (key in moves && !event.ctrlKey && !event.altKey) {
      const moved = table?.move(moves[key], event.shiftKey);
      // After the selection updates, audition the patch the cursor landed on.
      if (moved && !event.shiftKey && auditionSettings.on_move && connected && !busy) {
        queueMicrotask(() => audition((activePane === 'synth' ? selectedSynth : selectedFile)[0]));
      }
    } else if (key === 'ArrowLeft' || key === 'ArrowRight') {
      activePane = key === 'ArrowLeft' ? 'synth' : 'file';
    } else if (event.ctrlKey && key.toLowerCase() === 'a') {
      table?.selectAll();
    } else if (key === 'Escape') {
      table?.clearSelection();
    } else if (key === ' ' && !event.ctrlKey) {
      if (connected && !busy) audition(selected[0]);
    } else if (key === 'Enter' && activePane === 'file' && !(event.target instanceof HTMLButtonElement)) {
      if (connected && !busy) startWrite(selectedFile);
    } else if (key.toLowerCase() === 'f' && !event.ctrlKey && !event.altKey) {
      toggleFavourite(selected);
    } else if (key.toLowerCase() === 'r' && !event.ctrlKey && !event.altKey && activePane === 'synth') {
      if (!busy) reRead(selectedSynth);
    } else if (key.toLowerCase() === 't' && !event.ctrlKey && !event.altKey) {
      if (selected.length > 0) tagging = selected;
    } else if (key === 'Delete' && activePane === 'file') {
      removeFromList(selectedFile);
    } else if (key === 'Delete' && activePane === 'synth') {
      if (selectedSynth.length > 0 && initBlocked() === null) initializing = selectedSynth.map((p) => p.location!);
    } else {
      return false;
    }
    return true;
  }

  /** Opens the previous (-1) or next (+1) bank tab. */
  function switchBank(direction: number) {
    const index = ALL_BANKS.indexOf(bank as (typeof ALL_BANKS)[number]);
    bank = ALL_BANKS[(index + direction + ALL_BANKS.length) % ALL_BANKS.length];
    synthSearch = '';
    synthFavOnly = false;
    synthSelected = [];
    activePane = 'synth';
  }

  function isTextField(target: EventTarget | null): boolean {
    return target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement;
  }

  /** Hides the webview's own menu (Reload, Inspect…) except in text fields. */
  function suppressNativeMenu(event: MouseEvent) {
    if (!(event.target instanceof HTMLInputElement)) event.preventDefault();
  }

  function menuAction(id: string) {
    if (mustAccept) return;
    else if (id === 'about' || id === 'disclaimer' || id === 'licences' || id === 'appearance' || id === 'preferences')
      dialog = id;
    else if (id === 'restore_points') dialog = 'restore';
    else if (busy || writing || renaming || initializing || layerCopy) return;
    else if (id === 'open') openFile();
    else if (id === 'new_list') newList();
    else if (id === 'add_file') addFile();
    else if (id === 'save') saveList();
    else if (id === 'save_as') saveListAs();
    else if (id === 'bank_sheet') dialog = 'bankSheet';
    else if (id === 'backup') {
      if (connected) backUpAll();
      else message = { text: 'Connect to the Prophet 12 before backing up.', error: true };
    }
  }

  /** Finds the synth and connects, once the disclaimer is accepted. */
  function start() {
    refreshPorts()
      .then(() => {
        if (inputIndex >= 0 && outputIndex >= 0) connect();
      })
      .catch((e) => (message = { text: errorText(e), error: true }));
  }

  async function acceptDisclaimer() {
    await api.acceptDisclaimer();
    mustAccept = false;
    start();
  }

  let version = $state('');

  // The splitter between the panes. Saved when let go, so dragging doesn't write the settings file.
  let splitting = false;

  function splitAt(clientX: number) {
    if (!mainEl) return;
    const box = mainEl.getBoundingClientRect();
    const style = getComputedStyle(mainEl);
    const left = box.left + parseFloat(style.paddingLeft);
    const width = box.width - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight);
    split = Math.min(0.75, Math.max(0.25, (clientX - left) / width));
  }

  function splitterDown(event: PointerEvent) {
    if (event.button !== 0) return;
    splitting = true;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    event.preventDefault();
  }

  function splitterUp() {
    if (!splitting) return;
    splitting = false;
    api.setSplit(split);
  }

  function splitterKey(event: KeyboardEvent) {
    const step = { ArrowLeft: -0.05, ArrowRight: 0.05 }[event.key];
    if (step === undefined && event.key !== 'Home') return;
    event.preventDefault();
    event.stopPropagation();
    split = event.key === 'Home' ? 0.5 : Math.min(0.75, Math.max(0.25, split + step!));
    api.setSplit(split);
  }

  onMount(() => {
    getVersion().then((v) => (version = v));
    api.getTags().then(setTags, (e) => (message = { text: errorText(e), error: true }));
    const unlisten = onProgress((p) => (progress = p));
    const unlistenMenu = listen<string>('menu', (event) => menuAction(event.payload));
    // Ctrl+wheel must cancel the webview's own handling, so the listener can't be passive.
    window.addEventListener('wheel', wheel, { passive: false });
    const unlistenErrors = listen<string>('app-error', (event) => (message = { text: event.payload, error: true }));
    const unlistenSettings = listen<Settings>('settings', (event) => applySettings(event.payload));
    const unlistenRecent = listen<string>('open-recent', (event) => openRecent(event.payload));
    const portWatch = setInterval(watchPorts, 2000);
    api.getSettings().then((settings) => {
      zoomLevel = settings.zoom; // no "Zoom" message at startup
      applySettings(settings);
    });
    api.backupDir().then((dir) => (backupDir = dir));
    // Closing the window, or File → Quit: ask first if the list has unsaved changes.
    const unlistenClose = getCurrentWindow().onCloseRequested(async (event) => {
      if (!(await discardChanges())) event.preventDefault();
    });
    api.disclaimerAccepted().then((accepted) => {
      if (accepted) start();
      else mustAccept = true;
    });
    return () => {
      unlisten.then((f) => f());
      unlistenMenu.then((f) => f());
      unlistenSettings.then((f) => f());
      unlistenErrors.then((f) => f());
      unlistenClose.then((f) => f());
      unlistenRecent.then((f) => f());
      clearInterval(portWatch);
      window.removeEventListener('wheel', wheel);
    };
  });
</script>

<svelte:window onkeydown={keydown} oncontextmenu={suppressNativeMenu} />

<header>
  <h1>P12 Librarian</h1>
  <div class="ports">
    <label>
      <span>In</span>
      <select bind:value={inputIndex} disabled={!!busy} title="MIDI port the app receives from (the synth's output)">
        <option value={-1}>—</option>
        {#each ports.inputs as name, i}<option value={i}>{name}</option>{/each}
      </select>
    </label>
    <label>
      <span>Out</span>
      <select bind:value={outputIndex} disabled={!!busy} title="MIDI port the app sends to (the synth's input)">
        <option value={-1}>—</option>
        {#each ports.outputs as name, i}<option value={i}>{name}</option>{/each}
      </select>
    </label>
    <button onclick={refreshPorts} disabled={!!busy} title="Rescan MIDI ports, e.g. after plugging in the synth">↻</button>
    {#if connected}
      <button
        onclick={disconnect}
        disabled={!!busy}
        title="Close the MIDI connection. The app then won't connect by itself until you click Connect.">Disconnect</button
      >
    {:else}
      <button
        class="primary"
        onclick={() => connect()}
        disabled={!!busy || inputIndex < 0 || outputIndex < 0}
        title={inputIndex < 0 || outputIndex < 0
          ? 'Choose an input and an output port first'
          : 'Open the selected ports and check that a Prophet 12 answers'}>Connect</button
      >
    {/if}
    <span
      class="status"
      class:ok={connected}
      title={connected
        ? `${model}, OS ${osVersion}, answered and is ready. The Global screen shows the full OS version.`
        : 'Not talking to a synth. A Prophet 12 connects by itself when it is plugged in or switched on.'}
      >{#if connected}● <span class="model">{model}&nbsp;·&nbsp;</span><span class="connected">Connected&nbsp;·&nbsp;</span>OS {osVersion}{:else}○
        Not connected{/if}</span
    >
  </div>
</header>

<main bind:this={mainEl} style:grid-template-columns="minmax(0, {split}fr) 16px minmax(0, {1 - split}fr)">
  <section>
    <div class="section-head">
      <h2>Prophet 12</h2>
      <div class="actions">
        <button onclick={readBank} disabled={!connected || !!busy} title="Read all 99 programs in bank {bank} from the synth{offline}">Read {bank}</button>
        <button onclick={readAllUser} disabled={!connected || !!busy} title="Read all 396 programs in User banks U1–U4{offline}">Read all User banks</button>
        <button
          class="primary"
          onclick={backUpAll}
          disabled={!connected || !!busy}
          title="Read all User banks and save them to one .syx file (Ctrl+B){offline}">Back up all…</button
        >
      </div>
    </div>
    <div class="toolbar">
      <div class="tabs">
        {#each ALL_BANKS as b}
          <button
            class="tab"
            class:active={!synthSearching && bank === b}
            class:factory={!isUserSlot(b)}
            onclick={() => ((bank = b), (synthSearch = ''), (synthFavOnly = false), (synthSelected = []))}
            ondragenter={() => dragOverTab(b)}
            ondragleave={(e) => !(e.currentTarget as Element).contains(e.relatedTarget as Node) && dragOverTab(null)}
            ondrop={() => dragOverTab(null)}
            title={tabTitle(b)}
          >
            {b}
            <small>{(synthSearching ? matchCount(b) : readCount(b)) || ''}</small>
          </button>
        {/each}
      </div>
      <div class="find">
        <button
          class="fav-filter"
          class:on={synthFavOnly}
          aria-pressed={synthFavOnly}
          onclick={() => (synthFavOnly = !synthFavOnly)}
          title="Show only favourites, from every bank that has been read">★</button
        >
        <SearchBox
          bind:value={synthSearch}
          bind:input={synthSearchInput}
          title="Show matching patches from every bank that has been read (Ctrl+F). Several words must all match; #pad finds patches tagged Pad."
        />
      </div>
    </div>
    <PatchTable
      bind:this={synthTable}
      active={activePane === 'synth'}
      rows={synthRows}
      bind:selected={synthSelected}
      draggable
      onDrop={dropOnSynth}
      canDropOn={canDropOnSynth}
      rearranges={(sameTable) => sameTable}
      showCompare={comparing}
      highlight={synthTerms}
      highlightTags={synthTagTerms}
      onToggleFavourite={(row) => toggleFavourite([synthPatches.get(row.location)!])}
      onTagClick={(tag) => showTag('synth', tag)}
      emptyText={synthEmptyText}
      onActivate={(row) => audition(synthPatches.get(row.location))}
      onContextMenu={(e) => openContextMenu('synth', e)}
      onFocus={() => (activePane = 'synth')}
    />
    <div class="actions footer">
      <span class="muted">
        {#if synthSearching}{synthRows.length} found, {/if}{selectedSynth.length} selected
      </span>
      <button
        onclick={() => audition(selectedSynth[0])}
        disabled={!connected || !!busy || selectedSynth.length !== 1}
        title="Play the selected patch from the synth's edit buffer without storing it. Replaces unsaved edits on the synth.{offline}{needOne(selectedSynth.length)}"
        >Audition</button
      >
      <button
        onclick={() => startRename('synth')}
        disabled={!!busy || renameBlocked('synth') !== null}
        title={renameTip('synth', 'Rename the selected patches on the synth (F2). Right-click for Cleanify names.')}
        >Rename…</button
      >
      <button
        onclick={() => saveSelected(selectedSynth)}
        disabled={!!busy || selectedSynth.length === 0}
        title="Save the selected patches into one .syx file{needSome(selectedSynth.length)}">Save selected…</button
      >
      <button
        onclick={() => saveSeparately(selectedSynth)}
        disabled={!!busy || selectedSynth.length === 0}
        title="Save each selected patch as its own .syx file in a folder you choose{needSome(selectedSynth.length)}"
        >Save as separate files…</button
      >
    </div>
  </section>

  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <div
    class="splitter"
    role="separator"
    aria-orientation="vertical"
    aria-valuenow={Math.round(split * 100)}
    aria-valuemin={25}
    aria-valuemax={75}
    tabindex="0"
    title="Drag to resize the panes. Double-click for equal widths."
    onpointerdown={splitterDown}
    onpointermove={(e) => splitting && splitAt(e.clientX)}
    onpointerup={splitterUp}
    onpointercancel={splitterUp}
    ondblclick={() => ((split = 0.5), api.setSplit(0.5))}
    onkeydown={splitterKey}
  ></div>

  <section>
    <div class="section-head">
      <h2>
        File {#if fileName}<span class="muted">— {fileName}{fileDirty ? ' *' : ''}</span>{:else if fileDirty}<span
            class="muted">— new list *</span
          >{/if}
      </h2>
      <div class="actions">
        <button onclick={openFile} disabled={!!busy} title="Open a .syx file to see, audition and write its patches (Ctrl+O)">Open .syx…</button>
        <button
          onclick={saveList}
          disabled={!!busy || filePatches.length === 0}
          title={fileDirty ? 'Save the list, with your changes (Ctrl+S). File menu: Save list as…' : 'Save the list (Ctrl+S). File menu: Save list as…'}
          >Save</button
        >
      </div>
    </div>
    <div class="toolbar">
      <span class="muted">
        <button
          class="compare"
          class:on={comparing}
          aria-pressed={comparing}
          disabled={filePatches.length === 0 || synthPatches.size === 0}
          onclick={() => (comparing = !comparing)}
          title="Show how the synth and this list differ, matched by sound (also finds renamed copies). Search is:same, is:differs, is:missing or is:dup to filter."
          >Compare</button
        >
        {#if comparing && filePatches.length && synthPatches.size}{compareSummary}{:else}{filePatches.length
            ? `${filePatches.length} patches`
            : ''}{/if}
      </span>
      <div class="find">
        <button
          class="fav-filter"
          class:on={fileFavOnly}
          aria-pressed={fileFavOnly}
          onclick={() => (fileFavOnly = !fileFavOnly)}
          title="Show only favourites in this file">★</button
        >
        <SearchBox
          bind:value={fileSearch}
          bind:input={fileSearchInput}
          title="Show only matching patches in this file (Ctrl+F). Several words must all match; #pad finds patches tagged Pad."
        />
      </div>
    </div>
    <PatchTable
      bind:this={fileTable}
      active={activePane === 'file'}
      rows={fileRows}
      bind:selected={fileSelected}
      draggable
      onDrop={dropOnFile}
      canDropOn={(_row, zone) => zone === 'row' && !fileFiltering}
      rearranges={(sameTable) => sameTable}
      onDropEnd={dropAtFileEnd}
      bind:shownKeys={fileShownKeys}
      showCompare={comparing && synthPatches.size > 0}
      onActivate={(row) => audition(filePatches.find((p) => p.id === row.id))}
      onContextMenu={(e) => openContextMenu('file', e)}
      onFocus={() => (activePane = 'file')}
      highlight={fileTerms}
      highlightTags={fileTagTerms}
      onToggleFavourite={(row) => toggleFavourite(filePatches.filter((p) => p.id === row.id))}
      onTagClick={(tag) => showTag('file', tag)}
      emptyText={filePatches.length
        ? noMatch(fileSearch, fileFavOnly)
        : 'Open a .syx file, or drag patches here from the Prophet 12 pane to start a new list.'}
    />
    <div class="actions footer">
      <span class="muted">
        {#if fileFiltering}{fileRows.length} found, {/if}{selectedFile.length} selected
      </span>
      <button
        onclick={() => audition(selectedFile[0])}
        disabled={!connected || !!busy || selectedFile.length !== 1}
        title="Play the selected patch from the synth's edit buffer without storing it. Replaces unsaved edits on the synth.{offline}{needOne(selectedFile.length)}"
        >Audition</button
      >
      <button
        onclick={() => startRename('file')}
        disabled={!!busy || renameBlocked('file') !== null}
        title={renameTip('file', 'Rename the selected patches in this list (F2). Right-click for Cleanify names.')}
        >Rename…</button
      >
      <button
        onclick={() => saveSelected(selectedFile)}
        disabled={!!busy || selectedFile.length === 0}
        title="Save the selected patches into a new .syx file{needSome(selectedFile.length)}">Save selected…</button
      >
      <button
        class="primary"
        onclick={() => startWrite(selectedFile)}
        disabled={!connected || !!busy || selectedFile.length === 0}
        title="Choose slots and write the selected patches to the synth. You confirm before anything is written.{offline}{needSome(selectedFile.length)}"
        >Write to synth…</button
      >
    </div>
  </section>
</main>

<footer>
  {#if busy}
    {#if progress}
      <progress max={progress.total} value={progress.done}></progress>
      <span class="muted count">{progress.done}/{progress.total}</span>
    {/if}
    <span class="label">{progress?.label ?? `${busy}…`}</span>
    {#if cancellable}<button class="cancel" onclick={() => api.cancel()} title="Stop reading. Programs read so far are kept.">Cancel</button>{/if}
  {:else if message}
    <span class="label" class:error={message.error} title={message.text}>{message.text}</span>
    {#if message.undo}
      <button
        class="cancel"
        onclick={undo}
        disabled={!connected}
        title={connected ? `Write back the previous contents (Ctrl+Z). Saved in ${message.undo}` : 'Connect to the Prophet 12 first'}
        >Undo</button
      >
    {/if}
  {:else}
    <span class="muted label">Double-click a patch, or press Space, to audition it. Right-click for more, e.g. Cleanify names. Press F1 for the user guide.</span>
  {/if}
  {#if version}<span class="version" title="P12 Librarian {version}">v{version}</span>{/if}
</footer>

{#if rearranging}
  <RearrangeDialog
    {...rearranging}
    initial={rearrangeMode}
    currentName={(location) => synthPatches.get(location)?.name}
    onConfirm={confirmRearrange}
    onCancel={() => (rearranging = null)}
  />
{/if}

{#if writing}
  <WriteDialog
    patches={writing.patches}
    start={writing.start}
    currentName={(location) => synthPatches.get(location)?.name}
    {backupDir}
    onConfirm={confirmWrite}
    onCancel={() => (writing = null)}
  />
{/if}

{#if renaming}
  <RenameDialog
    title={renaming.title}
    patches={renaming.patches}
    proposed={renaming.proposed}
    onSynth={renaming.pane === 'synth'}
    {backupDir}
    onConfirm={confirmRename}
    onCancel={() => (renaming = null)}
  />
{/if}

{#if layerCopy}
  <LayerBDialog
    source={layerCopy.source}
    target={layerCopy.target}
    current={synthPatches.get(layerCopy.target)}
    {backupDir}
    onConfirm={confirmLayerCopy}
    onCancel={() => (layerCopy = null)}
  />
{/if}

{#if initializing}
  <InitDialog
    locations={initializing}
    currentName={(location) => synthPatches.get(location)?.name}
    {backupDir}
    onConfirm={confirmInit}
    onCancel={() => (initializing = null)}
  />
{/if}

{#if tagging}
  <TagsDialog
    patches={tagging}
    entryOf={(fingerprint) => tagStore.get(fingerprint)}
    known={knownTags}
    onConfirm={(add, remove) => saveTags(tagging!, add, remove)}
    onCancel={() => (tagging = null)}
  />
{/if}

{#if renumbering}
  <RenumberDialog {...renumbering} onConfirm={confirmRenumber} onCancel={() => (renumbering = null)} />
{/if}

{#if contextMenu}
  <ContextMenu {...contextMenu} onClose={() => (contextMenu = null)} />
{/if}

{#if mustAccept}
  <DisclaimerDialog accept onAccept={acceptDisclaimer} onQuit={() => api.quit()} />
{:else if dialog === 'licences'}
  <LicencesDialog onClose={() => (dialog = null)} />
{:else if dialog === 'disclaimer'}
  <DisclaimerDialog onClose={() => (dialog = null)} />
{:else if dialog === 'bankSheet'}
  <BankSheetDialog
    {readCount}
    file={filePatches.length > 0
      ? { name: fileName ?? 'New list', count: filePatches.length }
      : null}
    onExport={exportSheet}
    onCancel={() => (dialog = null)}
  />
{:else if dialog === 'restore'}
  <RestoreDialog
    {connected}
    currentName={(location) => synthPatches.get(location)?.name}
    onRestore={(point: RestorePoint) => restoreFrom(point.path, `Restore point of ${point.time}`)}
    onOpen={async (point: RestorePoint) => {
      if (!(await discardChanges())) return;
      dialog = null;
      openPath(point.path);
    }}
    onClose={() => (dialog = null)}
  />
{:else if dialog === 'preferences'}
  <PreferencesDialog audition={auditionSettings} onClose={() => (dialog = null)} />
{:else if dialog === 'appearance'}
  <AppearanceDialog theme={appearance.theme} font={appearance.font} onClose={() => (dialog = null)} />
{:else if dialog === 'about'}
  <AboutDialog onClose={() => (dialog = null)} />
{/if}

<style>
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 10px 16px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }
  h1 {
    font-size: 16px;
    margin: 0;
    white-space: nowrap;
  }
  .ports {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .ports label {
    display: flex;
    align-items: center;
    gap: 6px;
    white-space: nowrap;
    min-width: 0;
  }
  .ports select {
    max-width: 220px;
    min-width: 6em;
  }
  .status {
    color: var(--muted);
    white-space: nowrap;
  }
  .status.ok {
    color: var(--ok);
  }
  /* The model where there's room; "Connected" where there isn't. */
  .status .connected {
    display: none;
  }
  @media (max-width: 1500px) {
    .status .model {
      display: none;
    }
    .status .connected {
      display: inline;
    }
  }
  main {
    flex: 1;
    display: grid;
    gap: 0;
    padding: 16px;
    min-height: 0;
  }
  .splitter {
    cursor: col-resize;
    position: relative;
    outline: none;
  }
  /* A thin line in the middle of the gap, stronger on hover and when focused. */
  .splitter::after {
    content: '';
    position: absolute;
    inset: 0 7px;
    border-radius: 1px;
    background: transparent;
  }
  .splitter:hover::after,
  .splitter:focus-visible::after {
    background: var(--accent);
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-height: 0;
    min-width: 0;
  }
  .section-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .section-head .actions {
    flex-wrap: wrap;
    justify-content: flex-end;
    margin-left: auto;
  }
  h2 {
    font-size: 14px;
    margin: 0;
    white-space: nowrap;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .footer {
    justify-content: flex-end;
    flex-wrap: wrap;
  }
  .footer .muted {
    margin-right: auto;
  }
  .compare {
    padding: 2px 8px;
    margin-right: 6px;
  }
  .compare.on {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--on-accent);
  }
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  /* With the ★ button beside it, as wide as the search box alone used to be. */
  .toolbar :global(.search) {
    width: 180px;
  }
  .find {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .fav-filter {
    padding: 4px 9px;
    color: var(--muted);
  }
  .fav-filter.on {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--on-accent);
  }
  .tabs {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }
  .tab {
    padding: 4px 10px;
    font-family: var(--mono);
  }
  .tab.factory {
    font-style: italic;
  }
  .tab.active {
    background: var(--accent);
    color: var(--on-accent);
    border-color: var(--accent);
  }
  .tab small {
    opacity: 0.7;
  }
  footer {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 16px;
    border-top: 1px solid var(--border);
    background: var(--surface);
    min-height: 40px;
    font-size: 13px;
  }
  footer progress {
    flex: none;
    width: 240px;
  }
  /* Fixed width so the label after it doesn't move as the count grows. */
  footer .count {
    flex: none;
    min-width: 7ch;
    font-variant-numeric: tabular-nums;
  }
  footer .label {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  footer .cancel {
    margin-left: auto;
  }
  /* Always at the right end, after Cancel or Undo when they're shown. */
  footer .version {
    flex: none;
    margin-left: auto;
    padding-left: 12px;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  footer .cancel + .version {
    margin-left: 0;
  }
  .muted {
    color: var(--muted);
  }
  .error {
    color: var(--danger);
  }
</style>
