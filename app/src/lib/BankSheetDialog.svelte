<script lang="ts">
  import Modal from './Modal.svelte';
  import { PROGRAMS_PER_BANK, USER_BANKS, FACTORY_BANKS } from './api';

  export interface SheetChoice {
    source: 'synth' | 'file';
    banks: string[];
    format: 'pdf' | 'csv';
  }

  interface Props {
    /** How many slots of a bank have been read. */
    readCount: (bank: string) => number;
    /** The File pane's list, or null when it's empty. */
    file: { name: string; count: number } | null;
    onExport: (choice: SheetChoice) => void;
    onCancel: () => void;
  }

  let { readCount, file, onExport, onCancel }: Props = $props();

  const banks = [...USER_BANKS, ...FACTORY_BANKS];
  const isRead = (bank: string) => readCount(bank) === PROGRAMS_PER_BANK;
  // svelte-ignore state_referenced_locally
  let source = $state<'synth' | 'file'>(banks.some(isRead) || !file ? 'synth' : 'file');
  let chosen = $state<string[]>(USER_BANKS.filter(isRead));
  let format = $state<'pdf' | 'csv'>('pdf');

  const problem = $derived(
    source === 'file'
      ? file
        ? null
        : 'The File pane is empty.'
      : !banks.some(isRead)
        ? 'Read a bank from the Prophet 12 first.'
        : chosen.length === 0
          ? 'Choose at least one bank.'
          : null,
  );

  function confirm() {
    if (!problem) onExport({ source, banks: banks.filter((b) => chosen.includes(b)), format });
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), confirm())} />

<Modal title="Export bank sheet" width="520px" onClose={onCancel}>
  <fieldset>
    <legend>Patches</legend>
    <label>
      <input type="radio" bind:group={source} value="synth" />
      Prophet 12 banks
    </label>
    <div class="banks" class:off={source !== 'synth'}>
      {#each banks as b}
        <label title={isRead(b) ? `The ${PROGRAMS_PER_BANK} programs of ${b}` : `Read ${b} first`} class:disabled={!isRead(b)}>
          <input type="checkbox" bind:group={chosen} value={b} disabled={source !== 'synth' || !isRead(b)} />
          {b}
        </label>
      {/each}
    </div>
    <label class:disabled={!file}>
      <input type="radio" bind:group={source} value="file" disabled={!file} />
      The list in the File pane{#if file}: {file.name}, {file.count} {file.count === 1 ? 'patch' : 'patches'}{/if}
    </label>
  </fieldset>

  <fieldset>
    <legend>Format</legend>
    <label title="A4 pages to print: one bank per page, with slot, name, Layer B and favourites">
      <input type="radio" bind:group={format} value="pdf" />
      PDF, to print
    </label>
    <label title="For a spreadsheet: slot, name, Layer B, favourite and tags">
      <input type="radio" bind:group={format} value="csv" />
      CSV, for a spreadsheet
    </label>
  </fieldset>

  {#if problem}<p class="problem">{problem}</p>{/if}

  {#snippet buttons()}
    <button onclick={onCancel} title="Close without exporting (Esc)">Cancel</button>
    <button class="primary" onclick={confirm} disabled={problem !== null} title="Choose where to save it (Enter)"
      >Export…</button
    >
  {/snippet}
</Modal>

<style>
  fieldset {
    border: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  legend {
    font-weight: 600;
    margin-bottom: 6px;
  }
  .banks {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 14px;
    margin-left: 24px;
    font-family: var(--mono);
  }
  .banks.off,
  label.disabled {
    color: var(--muted);
  }
  .problem {
    margin: 0;
    color: var(--danger);
  }
</style>
