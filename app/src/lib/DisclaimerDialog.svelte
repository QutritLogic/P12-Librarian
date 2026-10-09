<script lang="ts">
  import Modal from './Modal.svelte';
  import text from '../../../DISCLAIMER.md?raw';

  interface Props {
    /** Ask the user to accept (first start) instead of just showing the text. */
    accept?: boolean;
    onAccept?: () => void;
    onQuit?: () => void;
    onClose?: () => void;
  }

  let { accept = false, onAccept, onQuit, onClose }: Props = $props();
  let agreed = $state(false);

  // DISCLAIMER.md is plain paragraphs under "#" and "##" headings.
  const blocks = text
    .split(/\n\s*\n/)
    .map((block) => block.trim())
    .filter((block) => block && !block.startsWith('# '))
    .map((block) =>
      block.startsWith('## ')
        ? { heading: true, text: block.slice(3) }
        : { heading: false, text: block.replace(/\s*\n\s*/g, ' ') },
    );
  const title = text.match(/^# (.+)$/m)?.[1] ?? 'Disclaimer';
</script>

<!-- When accepting, Esc and clicks outside do nothing: the user must choose. -->
<Modal {title} width="680px" onClose={accept ? () => {} : (onClose ?? (() => {}))}>
  {#if accept}
    <p class="intro">
      This app writes to your Prophet 12's memory and can overwrite or erase programs. Please read and accept the
      following before using it.
    </p>
  {/if}
  <div class="text" tabindex="-1">
    {#each blocks as block, i (i)}
      {#if block.heading}
        <h3>{block.text}</h3>
      {:else}
        <p>{block.text}</p>
      {/if}
    {/each}
  </div>
  {#if accept}
    <label class="agree">
      <input type="checkbox" bind:checked={agreed} />
      I have read and agree to this disclaimer, and I use the software at my own risk.
    </label>
  {/if}

  {#snippet buttons()}
    {#if accept}
      <button onclick={onQuit} title="Close the app without using it">Quit</button>
      <button class="primary" disabled={!agreed} onclick={onAccept} title={agreed ? undefined : 'Tick the box first'}
        >I agree</button
      >
    {:else}
      <button class="primary" onclick={onClose} title="Close (Esc)">Close</button>
    {/if}
  {/snippet}
</Modal>

<style>
  p {
    margin: 0;
  }
  .intro {
    font-weight: 600;
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 50vh;
    overflow: auto;
    padding: 10px 14px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface-2);
    font-size: 12px;
    line-height: 1.45;
  }
  h3 {
    margin: 6px 0 0;
    font-size: 13px;
  }
  .agree {
    display: flex;
    align-items: flex-start;
    gap: 8px;
  }
  .agree input {
    margin-top: 3px;
  }
</style>
