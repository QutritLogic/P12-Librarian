<script module lang="ts">
  // Ids being dragged. Shared between tables: dataTransfer contents can't be
  // read during dragover, and the drop target needs the count to preview.
  let draggingIds: number[] = [];
</script>

<script lang="ts">
  import { highlightParts } from './api';

  /** Where on a row something is dropped: the whole slot, or just its Layer B. */
  export type DropZone = 'row' | 'layerB';

  export interface Row {
    key: string;
    location: string;
    name: string;
    layerB: string;
    /** null for a slot that hasn't been read. */
    id: number | null;
  }

  interface Props {
    rows: Row[];
    selected?: string[];
    /** Rows can be dragged out of this table. */
    draggable?: boolean;
    /** Rows accept drops; called with the target row, the dragged ids and the zone. */
    onDrop?: (row: Row, ids: number[], zone: DropZone) => void;
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
    /** The pane the keyboard acts on: shows the keyboard cursor. */
    active?: boolean;
  }

  /** Where the keyboard cursor moves: by rows, or to the first or last row. */
  export type Move = number | 'first' | 'last';

  let {
    rows,
    selected = $bindable([]),
    draggable = false,
    onDrop,
    canDropOn = () => true,
    onActivate,
    onContextMenu,
    onFocus,
    emptyText = 'Nothing here yet',
    highlight = [],
    active = false,
  }: Props = $props();

  /** Where a Shift selection starts. */
  let anchor = $state<number | null>(null);
  /** The keyboard cursor: the key of the row last clicked or moved to. */
  let cursorKey = $state<string | null>(null);
  let wrap = $state<HTMLDivElement>();
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
    event.dataTransfer?.setData('text/plain', `p12-ids:${draggingIds.join(',')}`);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = 'copy';
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
    if (event.dataTransfer) event.dataTransfer.dropEffect = 'copy';
    dropIndex = index;
    dropZone = zone;
  }

  function drop(event: DragEvent, row: Row) {
    event.preventDefault();
    const ids = draggingIds;
    const zone = dropZone;
    draggingIds = [];
    dropIndex = null;
    if (onDrop && ids.length > 0) onDrop(row, ids, zone);
  }

  function inDropRange(index: number): boolean {
    return (
      dropZone === 'row' && dropIndex !== null && index >= dropIndex && index < dropIndex + draggingIds.length
    );
  }
</script>

<div class="table-wrap" class:active bind:this={wrap} role="presentation" onmousedown={() => onFocus?.()}>
  {#if rows.length === 0}
    <p class="empty">{emptyText}</p>
  {:else}
    <table>
      <thead>
        <tr><th class="loc">Slot</th><th>Name</th><th>Layer B</th></tr>
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
            ondragend={() => ((draggingIds = []), (dropIndex = null))}
            ondragover={(e) => dragOver(e, index)}
            ondrop={(e) => drop(e, row)}
          >
            <td class="loc">{row.location}</td>
            <td>
              {#if row.id === null}—{:else}{#each highlightParts(row.name, highlight) as part}{#if part.match}<mark
                    >{part.text}</mark
                  >{:else}{part.text}{/if}{/each}{/if}
            </td>
            <td class="layer-b" class:drop-cell={dropZone === 'layerB' && dropIndex === index}>{row.layerB}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
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
