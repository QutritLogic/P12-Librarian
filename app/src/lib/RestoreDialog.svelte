<script lang="ts">
  import { onMount } from 'svelte';
  import { revealItemInDir } from '@tauri-apps/plugin-opener';
  import Modal from './Modal.svelte';
  import { api, errorText, type RestorePoint } from './api';

  interface Props {
    connected: boolean;
    /** The name now in a slot on the synth, if that slot has been read. */
    currentName: (location: string) => string | undefined;
    onRestore: (point: RestorePoint) => void;
    onOpen: (point: RestorePoint) => void;
    onClose: () => void;
  }

  let { connected, currentName, onRestore, onOpen, onClose }: Props = $props();

  let points = $state<RestorePoint[] | null>(null);
  let chosen = $state<RestorePoint | null>(null);
  let error = $state('');
  /** The point whose Delete was clicked once; a second click deletes it. */
  let confirmingDelete = $state<string | null>(null);

  async function load() {
    try {
      points = await api.listRestorePoints();
      chosen = points.find((p) => p.path === chosen?.path) ?? points[0] ?? null;
    } catch (e) {
      error = errorText(e);
      points = [];
    }
  }

  onMount(load);

  /** "10 Oct 2026, 20:32": day first and 24-hour, whatever the web view's locale. */
  function when(time: string): string {
    const date = new Date(time.replace(' ', 'T'));
    return isNaN(date.getTime())
      ? time
      : date.toLocaleString('en-GB', { dateStyle: 'medium', timeStyle: 'short' });
  }

  /** "U1:03–U1:07" for a run of slots in one bank, otherwise the first few. */
  function summary(point: RestorePoint): string {
    const slots = point.slots.map((s) => s.location);
    if (slots.length === 1) return slots[0];
    const sameBank = slots.every((s) => s.slice(0, 2) === slots[0].slice(0, 2));
    const numbers = slots.map((s) => Number(s.slice(3)));
    const consecutive = numbers.every((n, i) => i === 0 || n === numbers[i - 1] + 1);
    if (sameBank && consecutive) return `${slots[0]}–${slots[slots.length - 1]}`;
    return slots.length <= 3 ? slots.join(', ') : `${slots.slice(0, 2).join(', ')} and ${slots.length - 2} more`;
  }

  async function remove(point: RestorePoint) {
    if (confirmingDelete !== point.path) {
      confirmingDelete = point.path;
      return;
    }
    confirmingDelete = null;
    try {
      await api.deleteRestorePoint(point.path);
      if (chosen?.path === point.path) chosen = null;
      await load();
    } catch (e) {
      error = errorText(e);
    }
  }

  const slotWord = (n: number) => `${n} ${n === 1 ? 'slot' : 'slots'}`;
</script>

<Modal title="Restore points" width="860px" {onClose}>
  <p class="muted">
    Before every change on the synth, the slots about to change are saved here. Restoring writes them back, with
    the usual backup and verification, so a restore can be undone too.
  </p>
  {#if error}<p class="error">{error}</p>{/if}

  {#if points === null}
    <p class="muted">Loading…</p>
  {:else if points.length === 0}
    <p class="empty">No restore points yet. One is made before every write, rename, initialize or Layer B copy.</p>
  {:else}
    <div class="columns">
      <div class="list" role="listbox" aria-label="Restore points">
        {#each points as point (point.path)}
          <button
            class="point"
            class:active={chosen?.path === point.path}
            role="option"
            aria-selected={chosen?.path === point.path}
            onclick={() => ((chosen = point), (confirmingDelete = null))}
          >
            <span class="when">{when(point.time)}</span>
            <span class="what">Before {point.kind.toLowerCase()} · {summary(point)}</span>
          </button>
        {/each}
      </div>

      {#if chosen}
        <div class="details">
          <div class="slots">
            <table>
              <thead><tr><th>Slot</th><th>Saved</th><th>On the synth now</th></tr></thead>
              <tbody>
                {#each chosen.slots as slot (slot.location)}
                  {@const now = currentName(slot.location)}
                  <tr>
                    <td class="loc">{slot.location}</td>
                    <td>{slot.name}</td>
                    <td class:muted={now === undefined || now === slot.name}>
                      {now === undefined ? 'not read' : now === slot.name ? 'same name' : now}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
          <div class="point-actions">
            <button onclick={() => remove(chosen!)} title="Delete this restore point's file">
              {confirmingDelete === chosen.path ? 'Click again to delete' : 'Delete'}
            </button>
            <button onclick={() => revealItemInDir(chosen!.path)} title={chosen.path}>Show in folder</button>
            <button onclick={() => onOpen(chosen!)} title="Open it in the File pane, to restore only some patches"
              >Open in File pane</button
            >
            <button
              class="primary"
              disabled={!connected}
              onclick={() => onRestore(chosen!)}
              title={connected ? `Write these ${slotWord(chosen.slots.length)} back to the synth` : 'Connect to the Prophet 12 first'}
              >Restore {slotWord(chosen.slots.length)}</button
            >
          </div>
        </div>
      {/if}
    </div>
  {/if}

  {#snippet buttons()}
    <button onclick={onClose} title="Close (Esc)">Close</button>
  {/snippet}
</Modal>

<style>
  p {
    margin: 0;
  }
  .muted {
    color: var(--muted);
  }
  .error {
    color: var(--danger);
  }
  .empty {
    color: var(--muted);
    padding: 2em 0;
    text-align: center;
  }
  .columns {
    display: grid;
    grid-template-columns: minmax(220px, 2fr) 3fr;
    gap: 12px;
    min-height: 0;
    height: 48vh;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px;
    background: var(--surface);
  }
  .point {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 1px;
    border: none;
    text-align: left;
    white-space: normal;
  }
  .point.active,
  .point.active:hover:not(:disabled) {
    background: var(--accent);
    color: var(--on-accent);
  }
  .when {
    font-weight: 600;
  }
  .what {
    font-size: 12px;
    opacity: 0.85;
  }
  .details {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-height: 0;
  }
  .slots {
    flex: 1;
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }
  th {
    position: sticky;
    top: 0;
    background: var(--surface-2);
    text-align: left;
    padding: 5px 10px;
  }
  td {
    padding: 3px 10px;
    white-space: nowrap;
  }
  .loc {
    font-family: var(--mono);
    color: var(--muted);
  }
  .point-actions {
    display: flex;
    justify-content: flex-end;
    flex-wrap: wrap;
    gap: 8px;
  }
</style>
