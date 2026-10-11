<script lang="ts">
  import Modal from './Modal.svelte';
  import { STARTER_TAGS, cleanTag, type Patch, type TagEntry } from './api';

  interface Props {
    patches: Patch[];
    /** Tags and favourite mark by fingerprint. */
    entryOf: (fingerprint: string) => TagEntry | undefined;
    /** Every tag in use, for suggestions. */
    known: string[];
    onConfirm: (add: string[], remove: string[]) => void;
    onCancel: () => void;
  }

  let { patches, entryOf, known, onConfirm, onCancel }: Props = $props();

  const same = (a: string, b: string) => a.toLowerCase() === b.toLowerCase();

  /** How many of the patches have each tag now. Patches that sound the same count once. */
  const counts: [string, number][] = $derived.by(() => {
    const sounds = [...new Set(patches.map((p) => p.fingerprint))];
    const out: [string, number][] = [];
    for (const fp of sounds) {
      for (const tag of entryOf(fp)?.tags ?? []) {
        const seen = out.find(([t]) => same(t, tag));
        if (seen) seen[1]++;
        else out.push([tag, 1]);
      }
    }
    return out;
  });
  const soundCount = $derived(new Set(patches.map((p) => p.fingerprint)).size);

  let added = $state<string[]>([]);
  let removed = $state<string[]>([]);
  let input = $state('');
  let field = $state<HTMLInputElement>();

  /** The tags shown: current ones not removed, then new ones, with how many patches will have each. */
  const shown = $derived([
    ...counts
      .filter(([t]) => !removed.some((r) => same(r, t)))
      .map(([t, n]): [string, number] => [t, added.some((a) => same(a, t)) ? soundCount : n]),
    ...added.filter((a) => !counts.some(([t]) => same(t, a))).map((a): [string, number] => [a, soundCount]),
  ]);

  const suggestions = $derived(
    [...known, ...STARTER_TAGS.filter((s) => !known.some((k) => same(k, s)))]
      .filter((s) => !shown.some(([t, n]) => same(t, s) && n === soundCount))
      .slice(0, 24),
  );

  function add(raw: string) {
    const tag = cleanTag(raw);
    if (!tag) return;
    removed = removed.filter((r) => !same(r, tag));
    if (!added.some((a) => same(a, tag))) added = [...added, tag];
    input = '';
    field?.focus();
  }

  function remove(tag: string) {
    added = added.filter((a) => !same(a, tag));
    if (counts.some(([t]) => same(t, tag)) && !removed.some((r) => same(r, tag))) removed = [...removed, tag];
  }

  function keydown(event: KeyboardEvent) {
    if (event.key === 'Enter' || event.key === ',') {
      event.preventDefault();
      if (input.trim()) add(input);
      else if (event.key === 'Enter') save();
    }
  }

  function save() {
    if (input.trim()) add(input);
    onConfirm(added, removed);
  }

  const title = $derived(patches.length === 1 ? `Tags for “${patches[0].name}”` : `Tags for ${patches.length} patches`);
</script>

<Modal {title} width="520px" onClose={onCancel}>
  <div class="tags">
    {#each shown as [tag, n] (tag.toLowerCase())}
      <span class="tag" class:partial={n < soundCount}>
        {tag}{#if n < soundCount}<small title="{n} of {soundCount} have this tag">{n}/{soundCount}</small>{/if}
        {#if n < soundCount}<button class="mini" onclick={() => add(tag)} title="Give all of them this tag">+</button>{/if}
        <button class="mini" onclick={() => remove(tag)} title="Remove this tag">×</button>
      </span>
    {:else}
      <span class="muted">No tags yet.</span>
    {/each}
  </div>

  <!-- svelte-ignore a11y_autofocus -->
  <input
    bind:this={field}
    bind:value={input}
    onkeydown={keydown}
    placeholder="Type a tag and press Enter, e.g. Pad"
    maxlength="40"
    autofocus
  />

  {#if suggestions.length}
    <div class="tags suggestions">
      {#each suggestions as tag (tag.toLowerCase())}
        <button class="tag suggestion" onclick={() => add(tag)} title="Add “{tag}”">+ {tag}</button>
      {/each}
    </div>
  {/if}

  <p class="note">
    Tags belong to the sound, not the slot or the name: they stay with a patch when it's moved, renamed or saved to a
    file, and patches that sound the same share them. Search for a tag with <code>#tag</code>.
  </p>

  {#snippet buttons()}
    <button onclick={onCancel} title="Close without changing anything (Esc)">Cancel</button>
    <button class="primary" onclick={save} title="Save the tags (Enter in an empty field)">Save</button>
  {/snippet}
</Modal>

<style>
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    min-height: 26px;
    align-items: center;
  }
  .tag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 4px 2px 10px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--surface-2);
    font-size: 13px;
  }
  .tag.partial {
    border-style: dashed;
  }
  .tag small {
    color: var(--muted);
    font-size: 11px;
  }
  .mini {
    border: none;
    background: none;
    padding: 0 4px;
    font: inherit;
    color: var(--muted);
    cursor: pointer;
    border-radius: 999px;
  }
  .mini:hover {
    color: var(--text);
    background: var(--hover);
  }
  .suggestion {
    padding: 2px 10px;
    color: var(--muted);
    cursor: pointer;
    font: inherit;
    font-size: 12px;
  }
  .suggestion:hover {
    color: var(--text);
    border-color: var(--accent);
  }
  input {
    width: 100%;
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
