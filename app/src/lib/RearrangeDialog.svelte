<script lang="ts">
  import Modal from './Modal.svelte';
  import type { Rearrange } from './api';

  interface Props {
    /** The dragged slots. */
    sources: string[];
    /** Where they were dropped. */
    target: string;
    /** [slot, slot] pairs that trade places, or why they can't. */
    swap: [string, string][] | string;
    /** [from, to] pairs for every slot that changes, or why it can't be done. */
    move: [string, string][] | string;
    /** The choice made last time. */
    initial: Rearrange;
    /** Current names of synth slots that have been read. */
    currentName: (location: string) => string | undefined;
    onConfirm: (mode: Rearrange, pairs: [string, string][]) => void;
    onCancel: () => void;
  }

  let { sources, target, swap, move, initial, currentName, onConfirm, onCancel }: Props = $props();

  // Start with the last choice, unless it isn't possible this time.
  // svelte-ignore state_referenced_locally
  let mode = $state<Rearrange>(
    (initial === 'swap' ? typeof swap !== 'string' : typeof move === 'string') ? 'swap' : 'move',
  );

  const pairs = $derived(mode === 'swap' ? swap : move);
  const problem = $derived(typeof pairs === 'string' ? pairs : null);
  const name = (location: string) => currentName(location) ?? 'not read yet';
  const patches = $derived(sources.length === 1 ? 'this patch' : `${sources.length} patches`);
  const shifted = $derived(typeof move === 'string' ? 0 : move.length - sources.length);

  /** Rough time for `slots` slots: each is read, written and read back. */
  function duration(slots: number): string {
    const seconds = Math.ceil(slots * 0.25);
    if (seconds < 5) return 'a few seconds';
    if (seconds < 90) return `about ${seconds} seconds`;
    return `about ${Math.round(seconds / 60)} minutes`;
  }

  function confirm() {
    if (typeof pairs !== 'string') onConfirm(mode, pairs);
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), confirm())} />

<Modal title="Rearrange on the Prophet 12" width="600px" onClose={onCancel}>
  <div class="mode" role="radiogroup" aria-label="How to rearrange">
    <label
      class:disabled={typeof swap === 'string'}
      title="The dragged patches and the ones in the slots from {target} on trade places"
    >
      <input type="radio" bind:group={mode} value="swap" disabled={typeof swap === 'string'} />
      Swap
    </label>
    <label
      class:disabled={typeof move === 'string'}
      title="Put the dragged patches at {target}; the patches in between move up or down to close the gap"
    >
      <input type="radio" bind:group={mode} value="move" disabled={typeof move === 'string'} />
      Move and shift
    </label>
  </div>

  {#if problem}
    <p class="problem">{problem}</p>
  {:else if mode === 'swap' && typeof swap !== 'string'}
    <p>Swap {swap.length === 1 ? 'these two patches' : `${swap.length} pairs of patches`} on the synth?</p>
    <div class="list">
      <table>
        <tbody>
          {#each swap as [a, b] (a)}
            <tr>
              <td class="loc">{a}</td>
              <td>{name(a)}</td>
              <td class="arrow" aria-label="swaps with">⇄</td>
              <td class="loc">{b}</td>
              <td>{name(b)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else if typeof move !== 'string'}
    <p>
      Move {patches} to {target}{#if shifted > 0}, shifting {shifted === 1 ? 'one other patch' : `${shifted} others`}
        to close the gap{/if}?
    </p>
    <div class="list">
      <table>
        <thead><tr><th>Slot</th><th>Gets</th><th>From</th></tr></thead>
        <tbody>
          {#each move as [from, to] (to)}
            <tr class:dragged={sources.includes(from)}>
              <td class="loc">{to}</td>
              <td>{name(from)}</td>
              <td class="loc muted">{from}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

  {#if !problem}
    <p class="note">
      {#if typeof pairs !== 'string'}
        {@const slots = mode === 'swap' ? pairs.length * 2 : pairs.length}
        {slots} slots are rewritten, which takes {duration(slots)}.
      {/if}
      They are read first and kept as a restore point, and every slot is read back and checked afterwards. Undo
      (Ctrl+Z) puts them back. Hold Ctrl while dropping to copy instead.
    </p>
  {/if}

  {#snippet buttons()}
    <button onclick={onCancel} title="Close without changing anything (Esc)">Cancel</button>
    <button class="primary" onclick={confirm} disabled={problem !== null} title="Back up, rearrange and verify (Enter)"
      >{mode === 'swap' ? 'Swap' : 'Move'}</button
    >
  {/snippet}
</Modal>

<style>
  p {
    margin: 0;
  }
  .mode {
    display: flex;
    gap: 16px;
  }
  .mode label.disabled {
    color: var(--muted);
  }
  .list {
    overflow: auto;
    max-height: 40vh;
    border: 1px solid var(--border);
    border-radius: 6px;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }
  th,
  td {
    padding: 4px 10px;
    text-align: left;
    white-space: nowrap;
  }
  th {
    background: var(--surface-2);
    position: sticky;
    top: 0;
  }
  .loc {
    font-family: var(--mono);
    color: var(--muted);
    width: 4.5em;
  }
  .dragged td:not(.muted) {
    color: var(--accent);
  }
  .arrow {
    color: var(--accent);
    font-size: 16px;
    text-align: center;
    width: 2em;
  }
  .muted {
    color: var(--muted);
  }
  .note {
    color: var(--muted);
    font-size: 12px;
  }
  .problem {
    color: var(--danger);
  }
</style>
