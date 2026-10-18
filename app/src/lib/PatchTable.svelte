<script module lang="ts">
  // Ids being dragged. Shared between tables: dataTransfer contents can't be
  // read during dragover, and the drop target needs the count to preview.
  let draggingIds: number[] = [];
  // The table the drag started in, so a drop can tell a rearrange within one
  // table from a copy between tables.
  let dragSource: object | null = null;
</script>

<script lang="ts">
  import { api, highlightParts } from './api';

  /** Where on a row something is dropped: the whole slot, or just its Layer B. */
  export type DropZone = 'row' | 'layerB';

  /** How a drop was made. */
  export interface DropInfo {
    /** The patches came from this same table. */
    sameTable: boolean;
    /** Ctrl was held: copy rather than rearrange. */
    copy: boolean;
  }

  export interface Row {
    key: string;
    location: string;
    name: string;
    layerB: string;
    /** null for a slot that hasn't been read. */
    id: number | null;
    tags: string[];
    favourite: boolean;
    /** How the row compares with the other pane, shown when comparing. */
    compare?: { text: string; kind: CompareKind } | null;
    /** Other places in this pane with the same sound. */
    duplicates?: string[];
  }

  /** same: identical; renamed: same sound, other name; differs: another sound in that
   *  slot; elsewhere: found in another slot; missing: not found; unknown: can't tell. */
  export type CompareKind = 'same' | 'renamed' | 'differs' | 'elsewhere' | 'missing' | 'unknown';

  interface Props {
    rows: Row[];
    selected?: string[];
    /** Rows can be dragged out of this table. */
    draggable?: boolean;
    /** Rows accept drops; called with the target row, the dragged ids, the zone and how. */
    onDrop?: (row: Row, ids: number[], zone: DropZone, info: DropInfo) => void;
    /** Whether dropping here without Ctrl rearranges rather than copies (shows a move cursor). */
    rearranges?: (sameTable: boolean) => boolean;
    /** Whether a row (or its Layer B cell) may receive a drop. */
    canDropOn?: (row: Row, zone: DropZone) => boolean;
    onActivate?: (row: Row) => void;
    /** Right-click on a row; the row is selected first if it wasn't. */
    onContextMenu?: (event: MouseEvent, row: Row) => void;
    /** Any click in the table, so the parent knows which pane is active. */
    onFocus?: () => void;
    emptyText?: string;
    /** Search terms to highlight in names. */
    highlight?: string[];
    /** Searched tags (lowercase, without `#`), to highlight the matching tags. */
    highlightTags?: string[];
    /** Click on a row's star. */
    onToggleFavourite?: (row: Row) => void;
    /** Click on one of a row's tags. */
    onTagClick?: (tag: string) => void;
    /** The pane the keyboard acts on: shows the keyboard cursor. */
    active?: boolean;
    /** Show the Compare column. */
    showCompare?: boolean;
    /** Drops below the last row, or into an empty table; called with the dragged ids and how. */
    onDropEnd?: (ids: number[], info: DropInfo) => void;
  }

  /** Where the keyboard cursor moves: by rows, or to the first or last row. */
  export type Move = number | 'first' | 'last';

  let {
    rows,
    selected = $bindable([]),
    draggable = false,
    onDrop,
    canDropOn = () => true,
    rearranges = () => false,
    onActivate,
    onContextMenu,
    onFocus,
    emptyText = 'Nothing here yet',
    highlight = [],
    highlightTags = [],
    onToggleFavourite,
    onTagClick,
    active = false,
    showCompare = false,
    onDropEnd,
  }: Props = $props();

  const tagMatches = (tag: string) => highlightTags.some((t) => tag.toLowerCase().startsWith(t));

  /** Where a Shift selection starts. */
  let anchor = $state<number | null>(null);
  /** The keyboard cursor: the key of the row last clicked or moved to. */
  let cursorKey = $state<string | null>(null);
  let wrap = $state<HTMLDivElement>();
  /** Identifies this table as a drag source. */
  const self = {};
  let dropIndex = $state<number | null>(null);
  let dropZone = $state<DropZone>('row');
  let selectedSet = $derived(new Set(selected));

  function click(event: MouseEvent, index: number) {
    const row = rows[index];
    if (row.id === null) return;
    if (event.shiftKey && anchor !== null) {
      selectRange(anchor, index);
    } else if (event.ctrlKey || event.metaKey) {
      selected = selectedSet.has(row.key)
        ? selected.filter((k) => k !== row.key)
        : [...selected, row.key];
      anchor = index;
    } else {
      selected = [row.key];
      anchor = index;
    }
    cursorKey = row.key;
  }

  function selectRange(a: number, b: number) {
    const [from, to] = a < b ? [a, b] : [b, a];
    selected = rows
      .slice(from, to + 1)
      .filter((r) => r.id !== null)
      .map((r) => r.key);
  }

  /** Rows that fit in the visible part of the table, for Page Up/Down. */
  function pageSize(): number {
    const row = wrap?.querySelector('tbody tr');
    if (!wrap || !(row instanceof HTMLElement) || row.offsetHeight === 0) return 10;
    return Math.max(1, Math.floor(wrap.clientHeight / row.offsetHeight) - 2);
  }

  /**
   * Moves the keyboard cursor, skipping slots that haven't been read, and
   * selects the row it lands on (or the range from the anchor with `extend`).
   * Returns false if there was nowhere to move.
   */
  export function move(by: Move | 'page-up' | 'page-down', extend = false): boolean {
    const readable = rows.flatMap((r, i) => (r.id === null ? [] : [i]));
    if (readable.length === 0) return false;
    const step = by === 'page-up' ? -pageSize() : by === 'page-down' ? pageSize() : by;
    const last = readable.length - 1;
    // Position of the cursor among the readable rows, -1 if it isn't on one.
    const position = readable.indexOf(rows.findIndex((r) => r.key === cursorKey));
    let next: number;
    if (step === 'first') next = 0;
    else if (step === 'last') next = last;
    else if (position === -1) next = step > 0 ? 0 : last;
    else next = Math.min(last, Math.max(0, position + step));
    const target = readable[next];
    const key = rows[target].key;
    // Already there (at either end): nothing changes, so nothing to audition.
    if (!extend && next === position && selected.length === 1 && selected[0] === key) return false;
    if (extend && anchor !== null) selectRange(anchor, target);
    else {
      selected = [key];
      anchor = target;
    }
    cursorKey = key;
    scrollTo(target);
    return true;
  }

  // A different list (another bank, or a new search) starts at the top.
  let firstKey = $derived(rows[0]?.key);
  $effect(() => {
    void firstKey;
    if (wrap) wrap.scrollTop = 0;
  });

  export function selectAll() {
    selected = rows.filter((r) => r.id !== null).map((r) => r.key);
  }

  export function clearSelection() {
    selected = [];
  }

  function scrollTo(index: number) {
    // After the selection has rendered, so the row is where it will stay.
    requestAnimationFrame(() => {
      wrap?.querySelectorAll('tbody tr')[index]?.scrollIntoView({ block: 'nearest' });
    });
  }

  function contextMenu(event: MouseEvent, index: number) {
    event.preventDefault();
    onFocus?.();
    const row = rows[index];
    if (row.id === null) return;
    if (!selectedSet.has(row.key)) {
      selected = [row.key];
      anchor = index;
    }
    cursorKey = row.key;
    onContextMenu?.(event, row);
  }

  function dragStart(event: DragEvent, row: Row) {
    if (row.id === null) return event.preventDefault();
    if (!selectedSet.has(row.key)) selected = [row.key];
    draggingIds = rows.filter((r) => selected.includes(r.key) && r.id !== null).map((r) => r.id!);
    dragSource = self;
    event.dataTransfer?.setData('text/plain', `p12-ids:${draggingIds.join(',')}`);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = 'copyMove';
  }

  function zoneOf(event: DragEvent): DropZone {
    return event.target instanceof Element && event.target.closest('td.layer-b') ? 'layerB' : 'row';
  }

  function dragOver(event: DragEvent, index: number) {
    const zone = zoneOf(event);
    if (!onDrop || draggingIds.length === 0 || !canDropOn(rows[index], zone)) {
      dropIndex = null;
      return;
    }
    event.preventDefault();
    // Only the cursor; drop() asks whether Ctrl is really held.
    const move = zone === 'row' && !event.ctrlKey && rearranges(dragSource === self);
    if (event.dataTransfer) event.dataTransfer.dropEffect = move ? 'move' : 'copy';
    dropIndex = index;
    dropZone = zone;
  }

  async function drop(event: DragEvent, row: Row) {
    event.preventDefault();
    const ids = draggingIds;
    const zone = dropZone;
    const sameTable = dragSource === self;
    const eventCtrl = event.ctrlKey;
    draggingIds = [];
    dragSource = null;
    dropIndex = null;
    if (!onDrop || ids.length === 0) return;
    // On Linux, ask the windowing system: WebKitGTK can report Ctrl inverted
    // on drag events after a menu shortcut (see ctrl_held in commands.rs).
    const held = await api.ctrlHeld().catch(() => null);
    const copy = held ?? eventCtrl;
    onDrop(row, ids, zone, { sameTable, copy });
  }

  // Below the last row, or in an empty table: drops go to the end of the list.
  let dropAtEnd = $state(false);

  function dragOverEnd(event: DragEvent) {
    if (!onDropEnd || draggingIds.length === 0) return;
    // Over a row, the row's own handler decides.
    if (event.target instanceof Element && event.target.closest('tbody tr')) {
      dropAtEnd = false;
      return;
    }
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = dragSource === self ? 'move' : 'copy';
    dropIndex = null;
    dropAtEnd = true;
  }

  async function dropEnd(event: DragEvent) {
    if (!onDropEnd || !dropAtEnd) return;
    event.preventDefault();
    const ids = draggingIds;
    const sameTable = dragSource === self;
    const eventCtrl = event.ctrlKey;
    draggingIds = [];
    dragSource = null;
    dropAtEnd = false;
    if (ids.length === 0) return;
    const held = await api.ctrlHeld().catch(() => null);
    onDropEnd(ids, { sameTable, copy: held ?? eventCtrl });
  }

  function inDropRange(index: number): boolean {
    return (
      dropZone === 'row' && dropIndex !== null && index >= dropIndex && index < dropIndex + draggingIds.length
    );
  }
</script>

<div
  class="table-wrap"
  class:active
  class:drop-end={dropAtEnd}
  bind:this={wrap}
  role="presentation"
  onmousedown={() => onFocus?.()}
  ondragover={dragOverEnd}
  ondragleave={(e) => e.currentTarget === e.target && (dropAtEnd = false)}
  ondrop={dropEnd}
>
  {#if rows.length === 0}
    <p class="empty">{emptyText}</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th class="fav" title="Favourites">★</th><th class="loc">Slot</th><th>Name</th><th>Layer B</th>
          {#if showCompare}<th class="cmp">Compare</th>{/if}
        </tr>
      </thead>
      <tbody ondragleave={() => (dropIndex = null)}>
        {#each rows as row, index (row.key)}
          <tr
            class:selected={selectedSet.has(row.key)}
            class:cursor={active && cursorKey === row.key}
            class:unread={row.id === null}
            class:drop={inDropRange(index)}
            draggable={draggable && row.id !== null}
            onclick={(e) => click(e, index)}
            ondblclick={() => row.id !== null && onActivate?.(row)}
            oncontextmenu={(e) => contextMenu(e, index)}
            ondragstart={(e) => dragStart(e, row)}
            ondragend={() => ((draggingIds = []), (dragSource = null), (dropIndex = null))}
            ondragover={(e) => dragOver(e, index)}
            ondrop={(e) => drop(e, row)}
          >
            <td class="fav">
              {#if row.id !== null}
                <!-- Not focusable, so the keyboard stays on the list. -->
                <button
                  class="star"
                  class:on={row.favourite}
                  tabindex="-1"
                  aria-pressed={row.favourite}
                  aria-label="Favourite"
                  title={row.favourite ? 'Favourite. Click to remove (F).' : 'Click to mark as a favourite (F)'}
                  onmousedown={(e) => e.preventDefault()}
                  onclick={(e) => (e.stopPropagation(), onToggleFavourite?.(row))}
                  ondblclick={(e) => e.stopPropagation()}>{row.favourite ? '★' : '☆'}</button
                >
              {/if}
            </td>
            <td class="loc">{row.location}</td>
            <td>
              {#if row.id === null}—{:else}{#each highlightParts(row.name, highlight) as part}{#if part.match}<mark
                    >{part.text}</mark
                  >{:else}{part.text}{/if}{/each}{#each row.tags as tag (tag)}<button
                    class="tag"
                    class:match={tagMatches(tag)}
                    tabindex="-1"
                    title="Show patches tagged “{tag}”"
                    onmousedown={(e) => e.preventDefault()}
                    onclick={(e) => (e.stopPropagation(), onTagClick?.(tag))}
                    ondblclick={(e) => e.stopPropagation()}>{tag}</button
                  >{/each}{#if row.duplicates?.length}<span
                    class="dup"
                    title="The same sound is also at {row.duplicates.join(', ')}">×{row.duplicates.length + 1}</span
                  >{/if}{/if}
            </td>
            <td class="layer-b" class:drop-cell={dropZone === 'layerB' && dropIndex === index}>{row.layerB}</td>
            {#if showCompare}
              <td class="cmp {row.compare?.kind ?? ''}">{row.compare?.text ?? ''}</td>
            {/if}
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  /* Drop target below the last row. */
  .table-wrap.drop-end {
    box-shadow: inset 0 -3px 0 var(--accent);
  }
  .dup {
    margin-left: 6px;
    padding: 0 5px;
    border-radius: 4px;
    font-size: 11px;
    color: var(--muted);
    border: 1px solid var(--border);
    cursor: help;
  }
  th.cmp,
  td.cmp {
    white-space: nowrap;
    font-size: 12px;
  }
  td.cmp.same,
  td.cmp.renamed {
    color: var(--ok);
  }
  td.cmp.differs {
    color: var(--danger);
  }
  td.cmp.elsewhere,
  td.cmp.missing,
  td.cmp.unknown {
    color: var(--muted);
  }
  tr.selected td.cmp {
    color: inherit;
  }
  .table-wrap {
    flex: 1;
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
  }
  /* The pane the keyboard acts on. */
  .table-wrap.active {
    border-color: var(--accent);
  }
  /* The keyboard cursor: a frame in the text colour, which stands out both on
     plain rows and on selected (accent-coloured) ones. */
  tr.cursor {
    --cursor: var(--text);
    outline: 2px solid var(--cursor);
    outline-offset: -2px;
  }
  tr.cursor.selected {
    --cursor: var(--on-accent);
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }
  thead th {
    position: sticky;
    top: 0;
    background: var(--surface-2);
    text-align: left;
    font-weight: 600;
    padding: 6px 10px;
    border-bottom: 1px solid var(--border);
  }
  td {
    padding: 4px 10px;
    white-space: nowrap;
    user-select: none;
  }
  tbody tr {
    cursor: default;
  }
  tbody tr:hover {
    background: var(--hover);
  }
  tr.selected,
  tr.selected:hover {
    background: var(--accent);
    color: var(--on-accent);
  }
  tr.unread {
    color: var(--muted);
  }
  td.drop-cell {
    outline: 2px dashed var(--accent);
    outline-offset: -2px;
    color: var(--text);
  }
  tr.drop {
    outline: 2px dashed var(--accent);
    outline-offset: -2px;
  }
  .loc {
    width: 4.5em;
    font-family: var(--mono);
  }
  .layer-b {
    color: var(--muted);
  }
  .fav {
    width: 1.6em;
    padding-right: 0;
    text-align: center;
  }
  .star {
    border: none;
    background: none;
    padding: 0 2px;
    font: inherit;
    line-height: 1;
    color: var(--muted);
    opacity: 0.45;
    cursor: pointer;
  }
  .star:hover {
    opacity: 1;
  }
  .star.on {
    color: var(--accent);
    opacity: 1;
  }
  .tag {
    margin-left: 6px;
    padding: 0 6px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--surface-2);
    color: var(--muted);
    font: inherit;
    font-size: 11px;
    line-height: 1.5;
    vertical-align: 1px;
    cursor: pointer;
  }
  .tag:hover {
    color: var(--text);
    border-color: var(--accent);
  }
  .tag.match {
    background: var(--highlight);
    color: var(--text);
  }
  tr.selected .star,
  tr.selected .star.on,
  tr.selected .tag {
    color: inherit;
  }
  tr.selected .tag {
    background: transparent;
    border-color: currentColor;
  }
  tr.selected .layer-b {
    color: inherit;
    opacity: 0.8;
  }
  mark {
    background: var(--highlight);
    color: inherit;
    border-radius: 2px;
  }
  tr.selected mark {
    background: rgb(255 255 255 / 0.3);
  }
  .empty {
    color: var(--muted);
    text-align: center;
    margin-top: 3em;
  }
</style>
