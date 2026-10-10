<script lang="ts">
  import Modal from './Modal.svelte';
  import licence from '../../../LICENSE?raw';
  import notices from '../../../THIRD-PARTY-NOTICES.md?raw';
  import javascript from '../../../licenses/javascript.md?raw';
  import rust from '../../../licenses/rust.md?raw';

  let { onClose }: { onClose: () => void } = $props();

  const TABS = [
    { id: 'licence', label: 'P12 Librarian', text: licence },
    { id: 'notices', label: 'Themes and fonts', text: notices },
    { id: 'javascript', label: 'JavaScript libraries', text: javascript },
    { id: 'rust', label: 'Rust libraries', text: rust },
  ] as const;

  let tab = $state<(typeof TABS)[number]['id']>('licence');
  const current = $derived(TABS.find((t) => t.id === tab)!);
</script>

<Modal title="Licences" width="820px" {onClose}>
  <p class="muted">
    P12 Librarian is freeware. It includes libraries, fonts and colour themes by other authors under their own
    licences, which this licence doesn't limit.
  </p>
  <div class="tabs" role="tablist">
    {#each TABS as t (t.id)}
      <button role="tab" class:active={tab === t.id} aria-selected={tab === t.id} onclick={() => (tab = t.id)}>
        {t.label}
      </button>
    {/each}
  </div>
  <pre class="text" tabindex="-1">{current.text}</pre>

  {#snippet buttons()}
    <button class="primary" onclick={onClose} title="Close (Esc)">Close</button>
  {/snippet}
</Modal>

<style>
  p {
    margin: 0;
  }
  .muted {
    color: var(--muted);
  }
  .tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .tabs button.active {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--on-accent);
  }
  .text {
    margin: 0;
    height: 52vh;
    overflow: auto;
    padding: 12px 14px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface-2);
    font-family: var(--mono);
    font-size: 12px;
    line-height: 1.45;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
</style>
