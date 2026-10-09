<script lang="ts">
  import Modal from './Modal.svelte';
  import type { Patch } from './api';

  interface Props {
    /** The patch whose Layer A sound is copied. */
    source: Patch;
    /** The User slot whose Layer B is replaced. */
    target: string;
    /** The target slot, if it has been read. */
    current: Patch | undefined;
    backupDir: string;
    onConfirm: () => void;
    onCancel: () => void;
  }

  let { source, target, current, backupDir, onConfirm, onCancel }: Props = $props();
</script>

<Modal title="Copy into Layer B of {target}" width="560px" onClose={onCancel}>
  <table>
    <tbody>
      <tr>
        <th>Layer A</th>
        <td>{current?.name ?? 'not read yet'}</td>
        <td class="muted">kept as it is</td>
      </tr>
      <tr>
        <th>Layer B</th>
        <td>
          {#if current}<span class="old">{current.layerB}</span> →{/if}
          <b>{source.name}</b>
        </td>
        <td class="muted">sound and name from {source.location ?? 'the file'}</td>
      </tr>
    </tbody>
  </table>

  <p class="note">
    This does the same as the synth's <b>Layers → Copy A -> B</b>, but from another patch. Layer B is only heard
    when the program is set to <b>Split</b> or <b>Stack</b>; otherwise press <b>Edit Layer B</b> on the synth to hear
    it.
  </p>
  <p class="note">
    The slot is read again first so its Layer A stays exactly as it is on the synth. Its current contents are saved
    to <code>{backupDir}</code>, and the result is read back and checked.
  </p>

  {#snippet buttons()}
    <button onclick={onCancel} title="Close without changing anything (Esc)">Cancel</button>
    <button class="primary" onclick={onConfirm} title="Back up {target}, write the new Layer B and verify it">
      Copy into Layer B
    </button>
  {/snippet}
</Modal>

<style>
  table {
    border-collapse: collapse;
    font-size: 13px;
  }
  th {
    text-align: left;
    padding: 4px 16px 4px 0;
    white-space: nowrap;
  }
  td {
    padding: 4px 16px 4px 0;
  }
  .old {
    color: var(--muted);
    text-decoration: line-through;
  }
  .muted,
  .note {
    color: var(--muted);
  }
  .note {
    margin: 0;
    font-size: 12px;
  }
</style>
