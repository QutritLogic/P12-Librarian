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
  }

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
  }: Props = $props();

  let anchor = $state<number | null>(null);
  let dropIndex = $state<number | null>(null);
  let dropZone = $state<DropZone>('row');
  let selectedSet = $derived(new Set(selected));

  function click(event: MouseEvent, index: number) {
    const row = rows[index];
    if (row.id === null) return;
    if (event.shiftKey && anchor !== null) {
      const [from, to] = anchor < index ? [anchor, index] : [index, anchor];
      selected = rows
        .slice(from, to + 1)
        .filter((r) => r.id !== null)
        .map((r) => r.key);
    } else if (event.ctrlKey || event.metaKey) {
      selected = selectedSet.has(row.key)
        ? selected.filter((k) => k !== row.key)
        : [...selected, row.key];
      anchor = index;
    } else {
      selected = [row.key];
      anchor = index;
    }
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

<div class="table-wrap" role="presentation" onmousedown={() => onFocus?.()}>
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
