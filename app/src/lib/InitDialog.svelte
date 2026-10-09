<script lang="ts">
  import { onMount } from 'svelte';
  import Modal from './Modal.svelte';
  import { api, errorText } from './api';

  interface Props {
    /** User slots to initialize, e.g. `U1:05`. */
    locations: string[];
    /** Current names of synth slots that have been read. */
    currentName: (location: string) => string | undefined;
    backupDir: string;
    onConfirm: (locations: string[]) => void;
    onCancel: () => void;
  }

  let { locations, currentName, backupDir, onConfirm, onCancel }: Props = $props();

  /** `capture` explains how to provide the template; `confirm` lists the slots. */
  let step = $state<'loading' | 'capture' | 'confirm'>('loading');
  let capturing = $state(false);
  let error = $state<string | null>(null);
  let captured = $state<string | null>(null);

  onMount(async () => {
    try {
      step = (await api.hasInitTemplate()) ? 'confirm' : 'capture';
    } catch (e) {
      error = errorText(e);
      step = 'capture';
    }
  });

  async function capture() {
    capturing = true;
    error = null;
    try {
      captured = await api.captureInitTemplate();
      step = 'confirm';
    } catch (e) {
      error = errorText(e);
    } finally {
      capturing = false;
    }
  }

  const count = $derived(`${locations.length} ${locations.length === 1 ? 'slot' : 'slots'}`);
</script>

<Modal title="Initialize {count}" onClose={onCancel}>
  {#if step === 'capture'}
    <p>
      Initializing uses the Prophet 12's own <b>Basic Patch</b> as a template. The app needs to read it from the
      synth once:
    </p>
    <ol>
      <li>On the Prophet 12, press <b>Global</b> and go to <b>23 Basic Patch</b>.</li>
      <li>
        Press <b>Write Now</b> (soft key 1). This loads the Basic Patch into the edit buffer and replaces any
        unsaved edits on the synth. Nothing is stored.
      </li>
      <li>Click <b>Capture template</b> below.</li>
    </ol>
    <p class="note">The template is kept by the app, so this is only needed once.</p>
  {:else if step === 'confirm'}
    {#if captured}
      <p class="ok">Captured the Basic Patch (“{captured}”) as the init template.</p>
    {/if}
    <p>These slots will be overwritten with the Basic Patch, named <b>Init Patch</b>:</p>
    <div class="list">
      <table>
        <thead><tr><th class="loc">Slot</th><th>Replaces</th></tr></thead>
        <tbody>
          {#each locations as location (location)}
            <tr>
              <td class="loc">{location}</td>
              <td class="muted">{currentName(location) ?? 'not read yet'}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    <p class="note">
      The current contents are saved to <code>{backupDir}</code> first, and every slot is read back and checked.
    </p>
  {/if}
  {#if error}<p class="problem">{error}</p>{/if}

  {#snippet buttons()}
    {#if step === 'confirm' && !captured}
      <button
        class="left"
        onclick={() => ((step = 'capture'), (error = null))}
        title="Read the Basic Patch from the synth again, e.g. after an OS update"
        >Recapture template…</button
      >
    {/if}
    <button onclick={onCancel} title="Close without changing anything (Esc)">Cancel</button>
    {#if step === 'capture'}
      <button
        class="primary"
        disabled={capturing}
        onclick={capture}
        title="Read the synth's edit buffer and keep it as the init template"
        >{capturing ? 'Capturing…' : 'Capture template'}</button
      >
    {:else}
      <button
        class="primary"
        disabled={step !== 'confirm'}
        onclick={() => onConfirm(locations)}
        title="Back up these slots, write the Init Patch to them and verify"
        >Initialize {count}</button
      >
    {/if}
  {/snippet}
</Modal>

<style>
  p,
  ol {
    margin: 0;
  }
  ol {
    padding-left: 20px;
  }
  li + li {
    margin-top: 4px;
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
    padding: 3px 10px;
    text-align: left;
  }
  th {
    background: var(--surface-2);
    position: sticky;
    top: 0;
  }
  .loc {
    font-family: var(--mono);
    width: 4.5em;
  }
  .muted,
  .note {
    color: var(--muted);
  }
  .note {
    font-size: 12px;
  }
  .ok {
    color: var(--ok);
  }
  .problem {
    color: var(--danger);
  }
  .left {
    margin-right: auto;
  }
</style>
