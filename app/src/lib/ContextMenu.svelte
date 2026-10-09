<script lang="ts">
  export interface MenuItem {
    label: string;
    action: () => void;
    disabled?: boolean;
    /** Tooltip, e.g. explaining why the item is disabled. */
    title?: string;
    shortcut?: string;
  }

  interface Props {
    x: number;
    y: number;
    /** Items; `null` draws a separator. */
    items: (MenuItem | null)[];
    onClose: () => void;
  }

  let { x, y, items, onClose }: Props = $props();
  let menu = $state<HTMLElement>();

  // Keep the menu inside the window.
  const position = $derived.by(() => {
    const width = menu?.offsetWidth ?? 0;
    const height = menu?.offsetHeight ?? 0;
    return {
      left: Math.max(4, Math.min(x, window.innerWidth - width - 4)),
      top: Math.max(4, Math.min(y, window.innerHeight - height - 4)),
    };
  });

  function choose(item: MenuItem) {
    if (item.disabled) return;
    onClose();
    item.action();
  }
</script>

<svelte:window
  onkeydown={(e) => e.key === 'Escape' && onClose()}
  onblur={onClose}
  onresize={onClose}
/>

<!-- Any click outside closes the menu. -->
<div
  class="catcher"
  role="presentation"
  onmousedown={onClose}
  oncontextmenu={(e) => (e.preventDefault(), onClose())}
></div>

<ul class="menu" role="menu" bind:this={menu} style:left="{position.left}px" style:top="{position.top}px">
  {#each items as item}
    {#if item === null}
      <li class="separator" role="separator"></li>
    {:else}
      <li role="none">
        <button role="menuitem" disabled={item.disabled} title={item.title} onclick={() => choose(item)}>
          <span>{item.label}</span>
          {#if item.shortcut}<kbd>{item.shortcut}</kbd>{/if}
        </button>
      </li>
    {/if}
  {/each}
</ul>

<style>
  .catcher {
    position: fixed;
    inset: 0;
    z-index: 20;
  }
  .menu {
    position: fixed;
    z-index: 21;
    margin: 0;
    padding: 4px;
    list-style: none;
    min-width: 200px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    box-shadow: 0 8px 24px rgb(0 0 0 / 0.3);
    font-size: 13px;
  }
  button {
    width: 100%;
    display: flex;
    justify-content: space-between;
    gap: 24px;
    border: none;
    background: none;
    text-align: left;
    padding: 5px 10px;
    border-radius: 4px;
  }
  button:hover:not(:disabled) {
    background: var(--accent);
    color: var(--on-accent);
  }
  kbd {
    font-family: var(--mono);
    font-size: 11px;
    opacity: 0.7;
  }
  .separator {
    height: 1px;
    margin: 4px 6px;
    background: var(--border);
  }
</style>
