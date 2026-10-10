<script lang="ts">
  import Modal from './Modal.svelte';
  import { api, noteName, type AuditionSettings } from './api';

  interface Props {
    audition: AuditionSettings;
    onClose: () => void;
  }

  let { audition, onClose }: Props = $props();

  // C1 to C6: the Prophet 12 keyboard's range is C1-C6 without transposing.
  const NOTES = Array.from({ length: 61 }, (_, i) => 36 + i);
  const LENGTHS = [250, 500, 1000, 2000, 4000];
  const CHANNELS = Array.from({ length: 16 }, (_, i) => i + 1);

  /** Saves at once; the backend clamps the values and tells the app. */
  function change(update: Partial<AuditionSettings>) {
    api.setAudition({ ...audition, ...update });
  }
</script>

<Modal title="Preferences" width="520px" {onClose}>
  <h3>Audition</h3>
  <p class="muted">
    Auditioning (double-click or Space) loads a patch into the synth's edit buffer without storing it.
  </p>

  <label class="check">
    <input type="checkbox" checked={audition.on_move} onchange={(e) => change({ on_move: e.currentTarget.checked })} />
    Audition each patch as you move with the arrow keys
  </label>

  <label class="check">
    <input type="checkbox" checked={audition.play_note} onchange={(e) => change({ play_note: e.currentTarget.checked })} />
    Play a note after loading the patch
  </label>

  <div class="grid" class:disabled={!audition.play_note}>
    <label for="audition-note">Note</label>
    <select
      id="audition-note"
      disabled={!audition.play_note}
      value={audition.note}
      onchange={(e) => change({ note: Number(e.currentTarget.value) })}
    >
      {#each NOTES as note (note)}
        <option value={note}>{noteName(note)}{note === 60 ? ' (middle C)' : ''}</option>
      {/each}
    </select>

    <label for="audition-length">Length</label>
    <select
      id="audition-length"
      disabled={!audition.play_note}
      value={audition.length_ms}
      onchange={(e) => change({ length_ms: Number(e.currentTarget.value) })}
    >
      {#each LENGTHS as ms (ms)}
        <option value={ms}>{ms / 1000} s</option>
      {/each}
    </select>

    <label for="audition-channel">MIDI channel</label>
    <select
      id="audition-channel"
      disabled={!audition.play_note}
      value={audition.channel}
      onchange={(e) => change({ channel: Number(e.currentTarget.value) })}
    >
      <option value={0}>Same as the synth (automatic)</option>
      {#each CHANNELS as channel (channel)}
        <option value={channel}>{channel}</option>
      {/each}
    </select>
  </div>
  <p class="muted small">
    Automatic reads the channel from the Prophet 12's Global settings each time, and uses channel 1 when the synth
    is set to All. The note stops early when you audition another patch or the app reads or writes.
  </p>

  {#snippet buttons()}
    <button class="primary" onclick={onClose} title="Close (Esc)">Close</button>
  {/snippet}
</Modal>

<style>
  h3 {
    margin: 0;
    font-size: 13px;
  }
  p {
    margin: 0;
  }
  .muted {
    color: var(--muted);
  }
  .small {
    font-size: 12px;
  }
  .check {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .grid {
    display: grid;
    grid-template-columns: auto 1fr;
    align-items: center;
    gap: 8px 16px;
    padding-left: 24px;
  }
  .grid.disabled label {
    opacity: 0.5;
  }
</style>
