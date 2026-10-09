<script lang="ts">
  import { onMount } from 'svelte';
  import Modal from './Modal.svelte';
  import { api, type AppearanceOptions, type FontId, type ThemeId } from './api';

  interface Props {
    theme: ThemeId;
    font: FontId;
    onClose: () => void;
  }

  let { theme, font, onClose }: Props = $props();
  let options = $state<AppearanceOptions>({ themes: [], fonts: [] });

  onMount(() => {
    api.appearanceOptions().then((o) => (options = o));
  });
</script>

<!-- Choices apply at once; the backend saves them and updates the View menu. -->
<Modal title="Appearance" width="820px" {onClose}>
  <h3>Theme</h3>
  <div class="themes" role="radiogroup" aria-label="Theme">
    {#each options.themes as t (t.id)}
      <button
        class="card"
        class:active={t.id === theme}
        data-theme={t.id}
        role="radio"
        aria-checked={t.id === theme}
        onclick={() => api.setAppearance({ theme: t.id })}
      >
        <div class="mock" aria-hidden="true">
          <div class="bar"><span>Prophet 12</span><span class="ok">●</span></div>
          <div class="row"><span class="slot">U1:01</span>It's a <mark>Proph</mark>et!</div>
          <div class="row selected"><span class="slot">U1:02</span>BigDirtyChoir</div>
          <div class="row"><span class="slot">U1:03</span><span class="muted">TinAttackLead</span></div>
          <div class="buttons"><span class="btn">Read</span><span class="btn primary">Write</span></div>
        </div>
        <div class="name">{t.name}</div>
      </button>
    {/each}
  </div>

  <h3>Font</h3>
  <div class="fonts" role="radiogroup" aria-label="Font">
    {#each options.fonts as f (f.id)}
      <button
        class="font"
        class:active={f.id === font}
        data-font={f.id}
        role="radio"
        aria-checked={f.id === font}
        onclick={() => api.setAppearance({ font: f.id })}
      >
        <span class="font-name">{f.name}</span>
        <span class="sample">U1:05 InstantCuemaker · 0O 1lI {'{}'} =&gt; 0123456789</span>
      </button>
    {/each}
  </div>

  {#snippet buttons()}
    <button class="primary" onclick={onClose} title="Close (Esc)">Close</button>
  {/snippet}
</Modal>

<style>
  h3 {
    margin: 4px 0 0;
    font-size: 13px;
  }
  .themes {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
    gap: 10px;
  }
  /* Each card carries data-theme, so var(--…) here are that theme's colours. */
  .card {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px;
    text-align: left;
    white-space: normal;
    background: var(--bg);
    color: var(--text);
    border: 2px solid var(--border);
    border-radius: 8px;
  }
  .card:hover:not(:disabled) {
    background: var(--bg);
    border-color: var(--accent);
  }
  .card.active {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent);
  }
  .mock {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: var(--surface);
    font-size: 11px;
    line-height: 1.5;
  }
  .bar {
    display: flex;
    justify-content: space-between;
    font-weight: 600;
    padding-bottom: 2px;
    border-bottom: 1px solid var(--border);
  }
  .ok {
    color: var(--ok);
  }
  .row {
    padding: 0 4px;
    border-radius: 3px;
    white-space: nowrap;
    overflow: hidden;
  }
  .row.selected {
    background: var(--accent);
    color: var(--on-accent);
  }
  .slot {
    color: var(--muted);
    margin-right: 6px;
  }
  .selected .slot {
    color: inherit;
    opacity: 0.8;
  }
  .muted {
    color: var(--muted);
  }
  mark {
    background: var(--highlight);
    color: inherit;
    border-radius: 2px;
  }
  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 4px;
    margin-top: 2px;
  }
  .btn {
    padding: 0 6px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--surface-2);
  }
  .btn.primary {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--on-accent);
  }
  .name {
    font-size: 12px;
    font-weight: 600;
    padding: 0 2px;
  }
  .fonts {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  /* Each row carries data-font, so var(--ui-font) is that font. */
  .font {
    display: flex;
    align-items: baseline;
    gap: 16px;
    padding: 6px 10px;
    text-align: left;
    font-family: var(--ui-font);
    border-width: 2px;
  }
  .font.active {
    border-color: var(--accent);
  }
  .font-name {
    min-width: 9.5em;
    font-weight: 600;
  }
  .sample {
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
