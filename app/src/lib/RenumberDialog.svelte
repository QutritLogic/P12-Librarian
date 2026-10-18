<script lang="ts">
  import Modal from './Modal.svelte';
  import { PROGRAMS_PER_BANK, USER_BANKS, slot, type Patch } from './api';

  interface Props {
    /** The patches to renumber, in list order. */
    patches: Patch[];
    /** True when only the selected patches are renumbered. */
    selectionOnly: boolean;
    onConfirm: (changes: { id: number; location: string }[]) => void;
    onCancel: () => void;
  }

  let { patches, selectionOnly, onConfirm, onCancel }: Props = $props();

  let bank = $state<string>(USER_BANKS[0]);
  let start = $state(1);

  /** Consecutive User slots from the start, continuing into the next bank; null past U4:99. */
  const slots = $derived.by(() => {
    const first = USER_BANKS.indexOf(bank as (typeof USER_BANKS)[number]) * PROGRAMS_PER_BANK + (start - 1);
    const last = first + patches.length - 1;
    if (last >= USER_BANKS.length * PROGRAMS_PER_BANK) return null;
    return patches.map((_, i) => {
      const n = first + i;
      return slot(USER_BANKS[Math.floor(n / PROGRAMS_PER_BANK)], (n % PROGRAMS_PER_BANK) + 1);
    });
  });
  const count = $derived(`${patches.length} ${patches.length === 1 ? 'patch' : 'patches'}`);
</script>

<Modal title="Renumber {count}" width="480px" onClose={onCancel}>
  <p>
    Give {selectionOnly ? 'the selected patches' : 'every patch in the list'} consecutive User slots, in list order.
    Only the list changes; save it to keep the new numbers.
  </p>
  <div class="row">
    <label for="renumber-bank">Start at</label>
    <select id="renumber-bank" bind:value={bank}>
      {#each USER_BANKS as b (b)}<option value={b}>{b}</option>{/each}
    </select>
    <select aria-label="Slot" bind:value={start}>
      {#each Array.from({ length: PROGRAMS_PER_BANK }, (_, i) => i + 1) as n (n)}
        <option value={n}>{String(n).padStart(2, '0')}</option>
      {/each}
    </select>
  </div>
  {#if slots}
    <p class="muted">
      {slots.length === 1 ? slots[0] : `${slots[0]} – ${slots[slots.length - 1]}`}{slots.length > 1 &&
      slots[0].slice(0, 2) !== slots[slots.length - 1].slice(0, 2)
        ? ', continuing into the next bank'
        : ''}
    </p>
  {:else}
    <p class="problem">That runs past U4:99. Start earlier, or renumber fewer patches.</p>
  {/if}

  {#snippet buttons()}
    <button onclick={onCancel} title="Close without changing anything (Esc)">Cancel</button>
    <button
      class="primary"
      disabled={!slots}
      onclick={() => slots && onConfirm(patches.map((p, i) => ({ id: p.id, location: slots[i] })))}
      >Renumber</button
    >
  {/snippet}
</Modal>

<style>
  p {
    margin: 0;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .muted {
    color: var(--muted);
  }
  .problem {
    color: var(--danger);
  }
</style>
