<script lang="ts">
  import { onMount } from 'svelte';
  import { SvelteMap } from 'svelte/reactivity';
  import { listen } from '@tauri-apps/api/event';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import AboutDialog from './lib/AboutDialog.svelte';
  import AppearanceDialog from './lib/AppearanceDialog.svelte';
  import PreferencesDialog from './lib/PreferencesDialog.svelte';
  import DisclaimerDialog from './lib/DisclaimerDialog.svelte';
  import ContextMenu, { type MenuItem } from './lib/ContextMenu.svelte';
  import InitDialog from './lib/InitDialog.svelte';
  import LayerBDialog from './lib/LayerBDialog.svelte';
  import PatchTable, { type DropZone, type Row } from './lib/PatchTable.svelte';
  import RenameDialog from './lib/RenameDialog.svelte';
  import SearchBox from './lib/SearchBox.svelte';
  import WriteDialog from './lib/WriteDialog.svelte';
  import {
    FACTORY_BANKS,
    USER_BANKS,
    api,
    bankSlots,
    errorText,
    isUserSlot,
    matchesSearch,
    noteName,
    onProgress,
    searchTerms,
    type Patch,
    type Ports,
    type Progress,
    type RenameRequest,
    type AuditionSettings,
    type Settings,
    type WriteRequest,
  } from './lib/api';

  // Connection
  let ports = $state<Ports>({ inputs: [], outputs: [] });
  let inputIndex = $state(-1);
  let outputIndex = $state(-1);
  let connected = $state(false);

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
  let message = $state<{ text: string; error: boolean } | null>(null);
  let backupDir = $state('');
  let writing = $state<{ patches: Patch[]; start: string | null } | null>(null);
  let dialog = $state<'about' | 'disclaimer' | 'appearance' | 'preferences' | null>(null);
  /** True until the user accepts the disclaimer; nothing touches the synth before that. */
  let mustAccept = $state(false);
  let renaming = $state<{ pane: Pane; title: string; patches: Patch[]; proposed: string[] } | null>(null);
  let initializing = $state<string[] | null>(null);
  let layerCopy = $state<{ source: Patch; target: string } | null>(null);
  let contextMenu = $state<{ x: number; y: number; items: (MenuItem | null)[] } | null>(null);

  type Pane = 'synth' | 'file';
  /** The pane last clicked, for keyboard shortcuts such as F2. */
  let activePane = $state<Pane>('synth');

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
  const synthSearching = $derived(synthTerms.length > 0);

  function synthRow(location: string): Row {
    const patch = synthPatches.get(location);
    return { key: location, location, name: patch?.name ?? '', layerB: patch?.layerB ?? '', id: patch?.id ?? null };
  }

  function synthMatches(location: string): boolean {
    const patch = synthPatches.get(location);
    return patch !== undefined && matchesSearch(synthTerms, patch.name, location);
  }

  /** The open bank, or while searching, the matching patches from every bank that has been read. */
  const synthRows: Row[] = $derived(
    synthSearching
      ? ALL_BANKS.flatMap(bankSlots).filter(synthMatches).map(synthRow)
      : bankSlots(bank).map(synthRow),
  );

  const fileRows: Row[] = $derived(
    filePatches
      .map((p, i) => ({
        key: String(p.id),
        location: p.location ?? `edit ${i + 1}`,
        name: p.name,
        layerB: p.layerB,
        id: p.id,
      }))
      .filter((row) => matchesSearch(fileTerms, row.name, row.location)),
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
  const selectedFile = $derived(
    filePatches.filter((p) => fileSelected.includes(String(p.id)) && visibleFile.has(String(p.id))),
  );
  const fileName = $derived(filePath?.split('/').pop() ?? null);

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
    return `No patches match “${synthSearch.trim()}”.${notAll}`;
  });

  function info(text: string) {
    message = { text, error: false };
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

  async function refreshPorts() {
    ports = await api.listPorts();
    const find = (names: string[]) => names.findIndex((n) => n.toLowerCase().includes('prophet 12'));
    if (inputIndex < 0 || inputIndex >= ports.inputs.length) inputIndex = find(ports.inputs);
    if (outputIndex < 0 || outputIndex >= ports.outputs.length) outputIndex = find(ports.outputs);
  }

  function connect() {
    return run('Connecting', async () => {
      connected = false;
      await api.connect(inputIndex, outputIndex);
      connected = true;
      info('Connected to the Prophet 12.');
    });
  }

  async function disconnect() {
    await api.disconnect();
    connected = false;
    info('Disconnected.');
  }

  async function readPrograms(selection: string): Promise<boolean> {
    const result = await api.readPrograms(selection);
    for (const p of result.patches) synthPatches.set(p.location!, p);
    return result.complete;
  }

  function readBank() {
    return run(`Reading ${bank}`, async () => {
      const complete = await readPrograms(bank);
      info(complete ? `Read bank ${bank}.` : 'Reading cancelled.');
    }, true);
  }

  function readAllUser() {
    return run('Reading all User banks', async () => {
      const complete = await readPrograms('user');
      info(complete ? 'Read all 4 User banks (396 programs).' : 'Reading cancelled.');
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
      if (!(await readPrograms('user'))) {
        info('Backup cancelled; nothing was saved.');
        return;
      }
      const ids = USER_BANKS.flatMap((b) => bankSlots(b).map((s) => synthPatches.get(s)!.id));
      await api.saveSyx(ids, path);
      info(`Saved all 396 User programs to ${path}`);
    }, true);
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

  async function openFile() {
    const path = await open({ filters: syxFilter, multiple: false });
    if (!path) return;
    await run('Opening', async () => {
      const result = await api.openSyx(path);
      filePath = path;
      filePatches = result.patches;
      fileSelected = [];
      const skipped = result.skipped ? ` (${result.skipped} other SysEx messages skipped)` : '';
      info(`Opened ${result.patches.length} patches${skipped}.`);
    });
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
      info(`Wrote and verified ${report.written.length} patches. Previous contents saved to ${report.backupPath}`);
    });
  }

  /** A patch from either pane, by id. */
  function patchById(id: number): Patch | undefined {
    return filePatches.find((p) => p.id === id) ?? [...synthPatches.values()].find((p) => p.id === id);
  }

  function dropOnSynth(row: Row, ids: number[], zone: DropZone) {
    const patches = ids.map(patchById).filter((p) => p !== undefined);
    if (zone === 'row') {
      startWrite(patches, row.location);
    } else if (patches.length !== 1) {
      message = { text: 'Drop a single patch onto a Layer B.', error: true };
    } else {
      layerCopy = { source: patches[0], target: row.location };
    }
  }

  function canDropOnSynth(row: Row, zone: DropZone): boolean {
    // While searching, a multi-patch drop could fill slots hidden by the search.
    return isUserSlot(row.location) && (zone === 'layerB' || !synthSearching);
  }

  function confirmLayerCopy() {
    const { source, target } = layerCopy!;
    layerCopy = null;
    return run('Copying into Layer B', async () => {
      const report = await api.copyToLayerB(source.id, target);
      for (const p of report.written) synthPatches.set(p.location!, p);
      info(`Copied "${source.name}" into Layer B of ${target} and verified it. Previous contents saved to ${report.backupPath}`);
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
      }
      const count = `${report.patches.length} ${report.patches.length === 1 ? 'patch' : 'patches'}`;
      info(
        report.backupPath
          ? `Renamed and verified ${count} on the synth. Previous contents saved to ${report.backupPath}`
          : `Renamed ${count} in this list. Use Save selected… to keep the new names.`,
      );
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
      info(`Initialized and verified ${count}. Previous contents saved to ${report.backupPath}`);
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
                label: one ? 'Initialize…' : `Initialize ${patches.length}…`,
                action: () => (initializing = patches.map((p) => p.location!)),
                disabled: initBlocked() !== null,
                title: initBlocked() ?? 'Replace with the Basic Patch, named “Init Patch”',
              },
            ]
          : []),
        null,
        { label: 'Save selected…', action: () => saveSelected(patches) },
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
  let auditionSettings = $state<AuditionSettings>({ play_note: true, note: 60, length_ms: 1000, channel: 0 });

  function applySettings(settings: Settings) {
    appearance = { theme: settings.theme, font: settings.font };
    auditionSettings = settings.audition;
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
    if (mustAccept || renaming || writing || initializing || layerCopy || dialog) return;
    if (event.ctrlKey && event.key.toLowerCase() === 'f') {
      event.preventDefault();
      const input = activePane === 'synth' ? synthSearchInput : fileSearchInput;
      input?.focus();
      input?.select();
    } else if (event.key === 'F2') {
      event.preventDefault();
      startRename(activePane);
    } else if (event.key === ' ' && !event.ctrlKey && !isTextField(event.target)) {
      // Space auditions the first selected patch in the active pane.
      event.preventDefault();
      if (connected && !busy) audition(activePane === 'synth' ? selectedSynth[0] : selectedFile[0]);
    }
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
    else if (id === 'about' || id === 'disclaimer' || id === 'appearance' || id === 'preferences') dialog = id;
    else if (busy || writing || renaming || initializing || layerCopy) return;
    else if (id === 'open') openFile();
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

  onMount(() => {
    const unlisten = onProgress((p) => (progress = p));
    const unlistenMenu = listen<string>('menu', (event) => menuAction(event.payload));
    // Ctrl+wheel must cancel the webview's own handling, so the listener can't be passive.
    window.addEventListener('wheel', wheel, { passive: false });
    const unlistenErrors = listen<string>('app-error', (event) => (message = { text: event.payload, error: true }));
    const unlistenSettings = listen<Settings>('settings', (event) => applySettings(event.payload));
    api.getSettings().then((settings) => {
      zoomLevel = settings.zoom; // no "Zoom" message at startup
      applySettings(settings);
    });
    api.backupDir().then((dir) => (backupDir = dir));
    api.disclaimerAccepted().then((accepted) => {
      if (accepted) start();
      else mustAccept = true;
    });
    return () => {
      unlisten.then((f) => f());
      unlistenMenu.then((f) => f());
      unlistenSettings.then((f) => f());
      unlistenErrors.then((f) => f());
      window.removeEventListener('wheel', wheel);
    };
  });
</script>

<svelte:window onkeydown={keydown} oncontextmenu={suppressNativeMenu} />

<header>
  <h1>P12 Librarian</h1>
  <div class="ports">
    <label>
      In
      <select bind:value={inputIndex} disabled={!!busy} title="MIDI port the app receives from (the synth's output)">
        <option value={-1}>—</option>
        {#each ports.inputs as name, i}<option value={i}>{name}</option>{/each}
      </select>
    </label>
    <label>
      Out
      <select bind:value={outputIndex} disabled={!!busy} title="MIDI port the app sends to (the synth's input)">
        <option value={-1}>—</option>
        {#each ports.outputs as name, i}<option value={i}>{name}</option>{/each}
      </select>
    </label>
    <button onclick={refreshPorts} disabled={!!busy} title="Rescan MIDI ports, e.g. after plugging in the synth">↻</button>
    {#if connected}
      <button onclick={disconnect} disabled={!!busy} title="Close the MIDI connection">Disconnect</button>
    {:else}
      <button
        class="primary"
        onclick={connect}
        disabled={!!busy || inputIndex < 0 || outputIndex < 0}
        title={inputIndex < 0 || outputIndex < 0
          ? 'Choose an input and an output port first'
          : 'Open the selected ports and check that a Prophet 12 answers'}>Connect</button
      >
    {/if}
    <span
      class="status"
      class:ok={connected}
      title={connected ? 'The Prophet 12 answered and is ready' : 'Not talking to a synth'}>{connected ? '● Connected' : '○ Not connected'}</span>
  </div>
</header>

<main>
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
            onclick={() => ((bank = b), (synthSearch = ''), (synthSelected = []))}
            title={tabTitle(b)}
          >
            {b}
            <small>{(synthSearching ? matchCount(b) : readCount(b)) || ''}</small>
          </button>
        {/each}
      </div>
      <SearchBox
        bind:value={synthSearch}
        bind:input={synthSearchInput}
        title="Show matching patches from every bank that has been read (Ctrl+F). Several words must all match."
      />
    </div>
    <PatchTable
      rows={synthRows}
      bind:selected={synthSelected}
      draggable
      onDrop={dropOnSynth}
      canDropOn={canDropOnSynth}
      highlight={synthTerms}
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

  <section>
    <div class="section-head">
      <h2>File {#if fileName}<span class="muted">— {fileName}</span>{/if}</h2>
      <div class="actions">
        <button onclick={openFile} disabled={!!busy} title="Open a .syx file to see, audition and write its patches (Ctrl+O)">Open .syx…</button>
      </div>
    </div>
    <div class="toolbar">
      <span class="muted">{filePatches.length ? `${filePatches.length} patches` : ''}</span>
      <SearchBox
        bind:value={fileSearch}
        bind:input={fileSearchInput}
        title="Show only matching patches in this file (Ctrl+F). Several words must all match."
      />
    </div>
    <PatchTable
      rows={fileRows}
      bind:selected={fileSelected}
      draggable
      onActivate={(row) => audition(filePatches.find((p) => p.id === row.id))}
      onContextMenu={(e) => openContextMenu('file', e)}
      onFocus={() => (activePane = 'file')}
      highlight={fileTerms}
      emptyText={filePatches.length
        ? `No patches match “${fileSearch.trim()}”.`
        : 'Open a .syx file to see its patches. Drag them onto a slot on the left to write them.'}
    />
    <div class="actions footer">
      <span class="muted">
        {#if fileTerms.length}{fileRows.length} found, {/if}{selectedFile.length} selected
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
    <span class:error={message.error}>{message.text}</span>
  {:else}
    <span class="muted">Double-click a patch, or press Space, to audition it. Right-click for more, e.g. Cleanify names. Press F1 for the user guide.</span>
  {/if}
</footer>

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

{#if contextMenu}
  <ContextMenu {...contextMenu} onClose={() => (contextMenu = null)} />
{/if}

{#if mustAccept}
  <DisclaimerDialog accept onAccept={acceptDisclaimer} onQuit={() => api.quit()} />
{:else if dialog === 'disclaimer'}
  <DisclaimerDialog onClose={() => (dialog = null)} />
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
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .ports select {
    max-width: 260px;
  }
  .status {
    color: var(--muted);
    white-space: nowrap;
  }
  .status.ok {
    color: var(--ok);
  }
  main {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    padding: 16px;
    min-height: 0;
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-height: 0;
  }
  .section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  h2 {
    font-size: 14px;
    margin: 0;
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
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .toolbar :global(.search) {
    width: 220px;
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
  .muted {
    color: var(--muted);
  }
  .error {
    color: var(--danger);
  }
</style>
