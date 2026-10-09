<script lang="ts">
  import Modal from './Modal.svelte';
  import { USER_BANKS, isUserSlot, offsetSlot, slot, type Patch, type WriteRequest } from './api';

  interface Props {
    patches: Patch[];
    /** Preset start slot, e.g. from a drop. */
    start?: string | null;
    /** Current names of synth slots that have been read. */
    currentName: (location: string) => string | undefined;
    backupDir: string;
    onConfirm: (writes: WriteRequest[]) => void;
    onCancel: () => void;
  }

  let { patches, start = null, currentName, backupDir, onConfirm, onCancel }: Props = $props();

  const canKeepLocations = $derived(patches.every((p) => p.location !== null && isUserSlot(p.location)));
  let mode = $state<'keep' | 'start'>('start');
  let bank = $state('U1');
  let program = $state(1);

  // Initialise from the props once; afterwards the user's choice wins.
  $effect.pre(() => {
    if (start) {
      [bank, program] = [start.split(':')[0], Number(start.split(':')[1])];
    } else if (canKeepLocations) {
      mode = 'keep';
    }
  });

  interface Line {
    patch: Patch;
    target: string | null;
  }

  const plan: Line[] = $derived(
    patches.map((patch, i) => ({
      patch,
      target: mode === 'keep' ? patch.location : offsetSlot(slot(bank, program), i),
    })),
  );

  const problem = $derived.by(() => {
    if (mode === 'start' && !(program >= 1 && program <= 99)) return 'Program must be 1–99.';
    if (plan.some((l) => l.target === null)) return `Not enough slots after ${slot(bank, program)}.`;
    if (plan.some((l) => !isUserSlot(l.target!))) return 'Factory banks are read-only.';
    return null;
  });

  function confirm() {
    if (problem) return;
    onConfirm(plan.map((l) => ({ id: l.patch.id, target: l.target! })));
  }
</script>

<Modal title="Write {patches.length} {patches.length === 1 ? 'patch' : 'patches'} to the Prophet 12" onClose={onCancel}>
  <div class="mode">
    <label
      class:disabled={!canKeepLocations}
      title={canKeepLocations
        ? 'Write each patch back to the slot it was saved from'
        : 'Only possible when every patch comes from a User slot'}
    >
      <input type="radio" bind:group={mode} value="keep" disabled={!canKeepLocations} />
      Original locations
    </label>
    <label title="Write the patches one after another, starting at the slot chosen here">
      <input type="radio" bind:group={mode} value="start" />
      Consecutive slots starting at
    </label>
    <select bind:value={bank} disabled={mode !== 'start'} title="User bank to write to">
      {#each USER_BANKS as b}<option value={b}>{b}</option>{/each}
    </select>
    <input
      type="number"
      min="1"
      max="99"
      bind:value={program}
      disabled={mode !== 'start'}
      title="Program number (1–99) for the first patch"
    />
  </div>

  <div class="plan">
    <table>
      <thead><tr><th>Patch</th><th></th><th>Slot</th><th>Replaces</th></tr></thead>
      <tbody>
        {#each plan as line (line.patch.id)}
          <tr>
            <td>{line.patch.name}</td>
            <td class="arrow">→</td>
            <td class="loc">{line.target ?? '—'}</td>
            <td class="muted">
              {line.target ? (currentName(line.target) ?? 'not read yet') : ''}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  <p class="note">
    The current contents of these slots are read and saved to <code>{backupDir}</code> first. Each written
    patch is read back and checked.
  </p>
  {#if problem}<p class="problem">{problem}</p>{/if}

  {#snippet buttons()}
    <button onclick={onCancel} title="Close without writing anything (Esc)">Cancel</button>
    <button
      class="primary"
      disabled={problem !== null}
      onclick={confirm}
      title={problem ?? 'Back up the target slots, write the patches and verify them'}
    >
      Write
    </button>
  {/snippet}
</Modal>

<style>
  .mode {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }
  .mode label.disabled {
    color: var(--muted);
  }
  .mode input[type='number'] {
    width: 4.5em;
  }
  .plan {
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: 6px;
    min-height: 80px;
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
  }
  .arrow,
  .muted {
    color: var(--muted);
  }
  .note {
    margin: 0;
    font-size: 12px;
    color: var(--muted);
  }
  .problem {
    margin: 0;
    color: var(--danger);
  }
</style>
