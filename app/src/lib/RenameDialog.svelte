<script lang="ts">
  import Modal from './Modal.svelte';
  import { MAX_NAME_LENGTH, nameProblem, type Patch, type RenameRequest } from './api';

  interface Props {
    title: string;
    patches: Patch[];
    /** Proposed new names, in the same order as `patches`. */
    proposed: string[];
    /** True when the patches are on the synth, so renaming writes to it. */
    onSynth: boolean;
    backupDir: string;
    onConfirm: (renames: RenameRequest[]) => void;
    onCancel: () => void;
  }

  let { title, patches, proposed, onSynth, backupDir, onConfirm, onCancel }: Props = $props();

  // Editable copies of the proposals; the dialog is created fresh each time.
  // svelte-ignore state_referenced_locally
  let names = $state([...proposed]);

  const problems = $derived(names.map(nameProblem));
  const changed = $derived(patches.filter((p, i) => names[i] !== p.name));
  const invalid = $derived(problems.some((p) => p !== null));

  function focusFirst(input: HTMLInputElement, index: number) {
    if (index === 0) input.select();
  }

  function confirm() {
    if (invalid || changed.length === 0) return;
    onConfirm(
      patches.flatMap((p, i) => (names[i] === p.name ? [] : [{ id: p.id, name: names[i] }])),
    );
  }

  const confirmLabel = $derived(
    changed.length === 0
      ? 'Rename'
      : onSynth
        ? `Rename ${changed.length} on synth`
        : `Rename ${changed.length}`,
  );
</script>

<Modal {title} onClose={onCancel}>
  <div class="list">
    <table>
      <thead><tr><th class="loc">Slot</th><th>Current name</th><th>New name</th></tr></thead>
      <tbody>
        {#each patches as patch, i (patch.id)}
          <tr class:unchanged={names[i] === patch.name}>
            <td class="loc">{patch.location ?? 'edit'}</td>
            <td class="old">{patch.name}</td>
            <td>
              <input
                type="text"
                maxlength={MAX_NAME_LENGTH}
                spellcheck="false"
                bind:value={names[i]}
                class:bad={problems[i] !== null}
                title={problems[i] ?? `Up to ${MAX_NAME_LENGTH} characters. Not allowed: * / : < > ? \\ | ~`}
                use:focusFirst={i}
                onkeydown={(e) => e.key === 'Enter' && confirm()}
              />
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  {#if onSynth}
    <p class="note">
      Each slot is read again from the synth and only its name is changed, so nothing else in the patch is
      touched. The current contents are saved to <code>{backupDir}</code> first, and every renamed patch is
      read back and checked.
    </p>
  {:else}
    <p class="note">
      This renames the patches in this list only. Use <b>Save selected…</b> to save them to a file, or write them
      to the synth.
    </p>
  {/if}
  {#if invalid}
    <p class="problem">{problems.find((p) => p !== null)}</p>
  {/if}

  {#snippet buttons()}
    <button onclick={onCancel} title="Close without renaming anything (Esc)">Cancel</button>
    <button
      class="primary"
      disabled={invalid || changed.length === 0}
      onclick={confirm}
      title={invalid
        ? 'Fix the highlighted names first'
        : changed.length === 0
          ? 'No names have been changed'
          : onSynth
            ? 'Back up the slots, store the new names on the synth and verify them (Enter)'
            : 'Apply the new names (Enter)'}
    >
      {confirmLabel}
    </button>
  {/snippet}
</Modal>

<style>
  .list {
    overflow: auto;
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
    padding: 3px 10px;
    text-align: left;
    white-space: nowrap;
  }
  th {
    background: var(--surface-2);
    position: sticky;
    top: 0;
    z-index: 1;
  }
  .loc {
    font-family: var(--mono);
    width: 4.5em;
  }
  .old {
    color: var(--muted);
  }
  input {
    width: 100%;
    font-family: var(--mono);
    padding: 2px 6px;
  }
  input.bad {
    border-color: var(--danger);
    outline-color: var(--danger);
  }
  tr.unchanged input {
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
