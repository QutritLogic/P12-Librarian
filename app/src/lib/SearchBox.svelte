<script lang="ts">
  interface Props {
    value: string;
    placeholder?: string;
    title?: string;
    input?: HTMLInputElement;
  }

  let { value = $bindable(''), placeholder = 'Search names…', title, input = $bindable() }: Props = $props();
</script>

<div class="search">
  <input
    type="search"
    bind:this={input}
    bind:value
    {placeholder}
    {title}
    spellcheck="false"
    onkeydown={(e) => e.key === 'Escape' && value && (e.stopPropagation(), (value = ''))}
  />
  {#if value}
    <button type="button" onclick={() => ((value = ''), input?.focus())} title="Clear the search (Esc)">×</button>
  {/if}
</div>

<style>
  .search {
    position: relative;
    display: flex;
  }
  input {
    flex: 1;
    /* Shrink to the box's width instead of the field's natural width. */
    min-width: 0;
    padding-right: 26px;
  }
  /* WebKit draws its own clear button on search inputs; we draw ours. */
  input::-webkit-search-cancel-button {
    display: none;
  }
  button {
    position: absolute;
    right: 3px;
    top: 50%;
    transform: translateY(-50%);
    border: none;
    background: none;
    padding: 0 6px;
    color: var(--muted);
    font-size: 16px;
    line-height: 1;
  }
</style>
