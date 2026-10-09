<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    width?: string;
    onClose: () => void;
    children: Snippet;
    buttons?: Snippet;
  }

  let { title, width = '640px', onClose, children, buttons }: Props = $props();
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onClose()} />

<div class="backdrop" role="presentation" onclick={onClose}>
  <!-- Clicks inside must not reach the backdrop; Esc is handled on the window. -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    class="dialog"
    role="dialog"
    aria-modal="true"
    aria-label={title}
    tabindex="-1"
    style:width="min({width}, 92vw)"
    onclick={(e) => e.stopPropagation()}
  >
    <h2>{title}</h2>
    <div class="body">{@render children()}</div>
    {#if buttons}<div class="buttons">{@render buttons()}</div>{/if}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgb(0 0 0 / 0.45);
    display: grid;
    place-items: center;
    z-index: 10;
  }
  .dialog {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 20px 24px;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    gap: 12px;
    box-shadow: 0 12px 40px rgb(0 0 0 / 0.35);
  }
  h2 {
    margin: 0;
    font-size: 16px;
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow: auto;
    min-height: 0;
  }
  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
