<script lang="ts">
  import Modal from './Modal.svelte';

  interface Props {
    /** Pairs of User slots that trade places, e.g. [`U1:04`, `U1:20`]. */
    pairs: [string, string][];
    /** Current names of synth slots that have been read. */
    currentName: (location: string) => string | undefined;
    onConfirm: (pairs: [string, string][]) => void;
    onCancel: () => void;
  }

  let { pairs, currentName, onConfirm, onCancel }: Props = $props();

  const count = $derived(pairs.length === 1 ? 'these two patches' : `${pairs.length} pairs of patches`);
  const name = (location: string) => currentName(location) ?? 'not read yet';
</script>

<svelte:window onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), onConfirm(pairs))} />

<Modal title="Swap {pairs.length === 1 ? 'patches' : `${pairs.length} pairs`}" width="560px" onClose={onCancel}>
  <p>Swap {count} on the synth?</p>
  <div class="list">
    <table>
      <tbody>
        {#each pairs as [a, b] (a)}
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
  <p class="note">
    Both slots are read first and kept as a restore point, the patches are written to each other's slot, and every
    slot is read back and checked. Undo (Ctrl+Z) swaps them back. Hold Ctrl while dropping to copy instead.
  </p>

  {#snippet buttons()}
    <button onclick={onCancel} title="Close without changing anything (Esc)">Cancel</button>
    <button class="primary" onclick={() => onConfirm(pairs)} title="Back up, swap and verify (Enter)">Swap</button>
  {/snippet}
</Modal>

<style>
  p {
    margin: 0;
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
  td {
    padding: 4px 10px;
    white-space: nowrap;
  }
  .loc {
    font-family: var(--mono);
    color: var(--muted);
    width: 4.5em;
  }
  .arrow {
    color: var(--accent);
    font-size: 16px;
    text-align: center;
    width: 2em;
  }
  .note {
    color: var(--muted);
    font-size: 12px;
  }
</style>
