<script lang="ts">
  import AppIcon from './AppIcon.svelte';
  import { onMount, untrack, mount, unmount } from 'svelte';
  import { Editor, Node } from '@tiptap/core';
  import { NodeSelection, TextSelection } from '@tiptap/pm/state';
  import { closeHistory } from '@tiptap/pm/history';
  import StarterKit from '@tiptap/starter-kit';
  import UniqueID from '@tiptap/extension-unique-id';
  import TaskList from '@tiptap/extension-task-list';
  import TaskItem from '@tiptap/extension-task-item';
  import { toEditor, fromEditor } from './editor-adapter';
  import { SourceLocator, sourceKey } from './source-locator';
  import { DocumentTableView } from './document-table-view';
  import { operate } from './api';
  import EmbeddedCollectionView from './EmbeddedCollectionView.svelte';
  import { ReadQueue, ReadLifetime } from './embedded-reads';
  import { slashQuery, matchingBlocks, topLevelBlock, blockById, copyWithNewIds, movePageBlock, popoverPosition } from './editor-page-controls';
  import type { Attachment, AttachmentData, Block, Document, Entity, Collection, CollectionBundle, CollectionView, Contract } from './contracts.generated';

  let { initial, disabled = false, workspace, focusBlock, label = "Note content", placeholder = '', showStarters = false, starterContext = 'task', pageStyle = false, onchange, onerror, oninvalid, onopen }: {
    initial: Document; disabled?: boolean; workspace: string;
    focusBlock?: string;
    label?: string; placeholder?: string; showStarters?: boolean; starterContext?: 'task' | 'note';
    pageStyle?: boolean;
    onchange: (document: Document) => void; onerror: (message: string) => void;
    oninvalid: (raw: string, message: string) => void;
    onopen: (entity: Entity) => Promise<void>;
  } = $props();
  let element: HTMLDivElement;
  let editor = $state.raw<Editor>();
  let shell: HTMLDivElement;
  let bodyEmpty = $state(false);
  let starterChosen = $state(false);
  function canStartBody(current: Editor) {
    const doc = current.state.doc;
    // Empty list items, headings and other existing structures are content.
    // Starter choices may replace only the single blank paragraph.
    return doc.childCount === 1 && doc.firstChild?.type.name === 'paragraph' && doc.firstChild.content.size === 0;
  }
  export function focusStart() {
    if (disabled || !editor) return;
    starterChosen = true;
    editor.commands.focus('start');
    if (pageStyle) editor.view.focus();
  }
  function startBody(kind: 'empty' | 'checklist' | 'notes') {
    if (disabled || !editor || !canStartBody(editor)) return;
    if (kind === 'empty') { focusStart(); return; }
    const paragraph = (): Block => ({ id: crypto.randomUUID(), type: 'paragraph', content: [] });
    const heading = (text: string): Block => ({ id: crypto.randomUUID(), type: 'heading', attrs: { level: 2 }, content: [{ type: 'text', text }] });
    const blocks: Block[] = kind === 'checklist' ? [{ id: crypto.randomUUID(), type: 'task_list', content: [], children: [0, 1].map(() => ({ id: crypto.randomUUID(), type: 'task_item', attrs: { checked: false }, content: [], children: [paragraph()] })) }]
      : [heading("Context"), paragraph(), heading("Notes"), paragraph()];
    starterChosen = true;
    editor.chain().command(({ tr }) => { closeHistory(tr); return true; }).setContent(toEditor({ schema_version: 2, blocks }), { emitUpdate: true, errorOnInvalidContent: true }).focus('start').run();
  }
  let slash = $state(false);
  let slashStart = 0;
  let pageMenu = $state<'slash' | 'insert' | 'turn' | null>(null);
  let menuQuery = $state(''), menuIndex = $state(0);
  let menuPosition = $state({ left: 0, top: 0 });
  let bubble = $state<{ left: number; top: number } | null>(null);
  let handle = $state<{ id: string; top: number; index: number; convertible: boolean } | null>(null);
  let blockMenu = $state(false);
  let panel = $state<'link' | 'attachment' | null>(null);
  let panelPosition = $state({ left: 0, top: 0 });
  let panelError = $state('');
  let panelInput = $state<HTMLInputElement>();
  let menuElement = $state<HTMLDivElement>();
  let blockButton = $state<HTMLButtonElement>();
  let dismissedSlash: number | null = null;
  let savedRange: { from: number; to: number } | null = null;
  let insertionRange: { from: number; to: number } | null = null;
  let draggingId = $state<string | null>(null);
  let dragCandidate: string | null = null;
  let pointerBlockId = $state<string | null>(null);
  let lastPointer: { x: number; y: number } | null = null;
  let dropTarget = $state<{ index: number; top: number } | null>(null);
  const menuItems = $derived(matchingBlocks(menuQuery, pageMenu === 'turn'));
  const menuId = $props.id();
  const activeMarks = $derived.by(() => { serial; return Object.fromEntries(['bold', 'italic', 'strike', 'code', 'link'].map(mark => [mark, editor?.isActive(mark) ?? false])); });
  const currentBlock = $derived.by(() => { serial; if (!editor) return 'Text'; for (const level of [1, 2, 3]) if (editor.isActive('heading', { level })) return `Heading ${level}`; return editor.isActive('codeBlock') ? 'Code' : editor.isActive('bulletList') ? 'Bulleted list' : editor.isActive('orderedList') ? 'Numbered list' : editor.isActive('taskList') ? 'To-do list' : editor.isActive('blockquote') ? 'Quote' : 'Text'; });
  let attachmentPath = $state('');
  let link = $state('');
  let importing = $state(false);
  let generation = 0;
  let serial = $state(0);
  let notificationQueued = false;
  function notifyTransaction() {
    if (notificationQueued) return;
    notificationQueued = true;
    // Blur can fire while Svelte replaces a view. Notify outside that render;
    // document updates themselves remain synchronous in onUpdate below.
    queueMicrotask(() => {
      notificationQueued = false;
      if (editor && !editor.isDestroyed) { serial++; syncPageUI(); }
    });
  }
  let documentVersion = 2;
  const reads = new ReadQueue(2);
  let picker = $state(false), pickerBusy = $state(false), pickerError = $state('');
  let pickerCollections = $state<Collection[]>([]), pickerCursor = $state<string | null>(null);
  let pickerCollection = $state(''), pickerView = $state(''), pickerViews = $state<CollectionView[]>([]);
  let pickerRange: { from: number; to: number } | null = null;
  const viewSelected = $derived.by(() => { serial; return editor?.state.selection instanceof NodeSelection && editor.state.selection.node.type.name === 'collectionView'; });

  const blocks = [
    { label: "Paragraph", command: 'paragraph' }, { label: "Heading", command: 'heading' },
    { label: "List", command: 'bullet' }, { label: "Numbered list", command: 'ordered' },
    { label: 'Checklist', command: 'task' }, { label: "Quote", command: 'quote' },
    { label: "Code", command: 'code' }, { label: "Divider", command: 'rule' },
    { label: "Embedded database", command: 'view' },
  ];
  function command(name: string, fromSlash = false) {
    if (!editor || disabled) return;
    if (name === 'view') { void openViewPicker(fromSlash); return; }
    if (name === 'attachment') { openPanel('attachment', fromSlash); return; }
    let c = pageStyle ? editor.chain() : editor.chain().focus();
    if (fromSlash) c = c.deleteRange({ from: slashStart, to: editor.state.selection.from });
    slash = false;
    pageMenu = null; blockMenu = false; dismissedSlash = slashStart;
    switch (name) {
      case 'paragraph': c.setParagraph().run(); break;
      case 'heading': c.toggleHeading({ level: 2 }).run(); break;
      case 'heading1': c.setHeading({ level: 1 }).run(); break;
      case 'heading2': c.setHeading({ level: 2 }).run(); break;
      case 'heading3': c.setHeading({ level: 3 }).run(); break;
      case 'bullet': c.toggleBulletList().run(); break;
      case 'ordered': c.toggleOrderedList().run(); break;
      case 'task': c.toggleTaskList().run(); break;
      case 'quote': c.toggleBlockquote().run(); break;
      case 'code': c.toggleCodeBlock().run(); break;
      case 'rule': c.setHorizontalRule().run(); break;
    }
    if (pageStyle) editor.view.focus();
  }
  function selectionRect() {
    if (!editor) return null;
    const { from, to } = editor.state.selection;
    const start = editor.view.coordsAtPos(from), end = editor.view.coordsAtPos(to);
    return { left: Math.min(start.left, end.left), right: Math.max(start.right, end.right), top: start.top, bottom: end.bottom };
  }
  function placeHandle(position: number) {
    if (!editor || !shell) return;
    const block = topLevelBlock(editor.state.doc, position);
    if (!block) return;
    const dom = editor.view.nodeDOM(block.pos);
    if (!(dom instanceof HTMLElement)) return;
    handle = { id: block.node.attrs.id, index: block.index, top: dom.getBoundingClientRect().top - shell.getBoundingClientRect().top + (block.node.type.name === 'heading' ? 5 : 1), convertible: block.node.isTextblock };
  }
  function syncPageUI() {
    if (!pageStyle || !editor || !shell) return;
    const current = editor;
    if (disabled || !shell.getClientRects().length) { bubble = null; pointerBlockId = null; return; }
    updateHoveredBlock();
    if (current.isFocused && !panel && !picker && !blockMenu && pageMenu !== 'insert' && pageMenu !== 'turn') {
      const candidate = slashQuery(current);
      if (!candidate) { dismissedSlash = null; if (pageMenu === 'slash') pageMenu = null; }
      else if (candidate.from !== dismissedSlash) {
        if (menuQuery !== candidate.query || pageMenu !== 'slash') menuIndex = 0;
        slashStart = candidate.from; menuQuery = candidate.query; pageMenu = 'slash';
      }
    }
    const rect = selectionRect();
    if (pageMenu && rect) menuPosition = popoverPosition(rect, 320, Math.min(420, menuItems.length * 51 + 72));
    const formattingFocus = current.isFocused || shell.querySelector('.selection-toolbar')?.contains(document.activeElement);
    bubble = formattingFocus && !current.state.selection.empty && !(current.state.selection instanceof NodeSelection) && !pageMenu && !panel && !picker && !blockMenu && rect
      ? popoverPosition(rect, 362, 40, true) : null;
  }
  function closeMenus(focus = true) {
    dismissedSlash = editor ? slashQuery(editor)?.from ?? null : null;
    pageMenu = null; blockMenu = false; panel = null; panelError = ''; insertionRange = null;
    if (focus && !disabled) editor?.view.focus();
  }
  function openCommands(mode: 'insert' | 'turn', anchor?: HTMLElement) {
    if (!editor || disabled) return;
    pageMenu = mode; menuQuery = ''; menuIndex = 0; blockMenu = false; bubble = null;
    if (mode === 'insert') editor.view.focus();
    const rect = anchor?.getBoundingClientRect() ?? selectionRect();
    if (rect) menuPosition = popoverPosition(rect, 320, 420);
    queueMicrotask(() => menuElement?.focus());
  }
  function addBlockBelow() {
    if (!editor || disabled || !handle) return;
    const block = blockById(editor.state.doc, handle.id);
    if (!block) return;
    const pos = block.pos + block.node.nodeSize;
    editor.chain().command(({ tr }) => { closeHistory(tr); return true; }).insertContentAt(pos, { type: 'paragraph' }).setTextSelection(pos + 1).run();
    openCommands('insert');
  }
  function chooseCommand(name: string) {
    const fromSlash = pageMenu === 'slash';
    command(name, fromSlash);
  }
  function commandKeydown(event: KeyboardEvent) {
    if (!pageMenu || event.isComposing) return false;
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      menuIndex = (menuIndex + (event.key === 'ArrowDown' ? 1 : -1) + menuItems.length) % Math.max(1, menuItems.length);
      queueMicrotask(() => menuElement?.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: 'nearest' }));
      return true;
    }
    if (event.key === 'Enter') { event.preventDefault(); if (menuItems[menuIndex]) chooseCommand(menuItems[menuIndex].command); return true; }
    if (event.key === 'Escape' || event.key === 'Tab') { closeMenus(); return event.key === 'Escape'; }
    if (pageMenu !== 'slash' && event.key.length === 1 && !event.metaKey && !event.ctrlKey && event.key !== ' ') { event.preventDefault(); menuQuery += event.key; menuIndex = 0; return true; }
    if (pageMenu !== 'slash' && event.key === 'Backspace') { event.preventDefault(); menuQuery = menuQuery.slice(0, -1); menuIndex = 0; return true; }
    return false;
  }
  function clearHover() {
    lastPointer = null; pointerBlockId = null;
  }
  function updateHoveredBlock() {
    if (!pageStyle || !editor || disabled || draggingId || dragCandidate) return;
    const target = lastPointer && document.elementFromPoint(lastPointer.x, lastPointer.y);
    // Keep the path from the block into its controls usable, without treating
    // the editor's empty minimum-height area as a hovered block.
    if (target && shell.querySelector('.block-handle-group')?.contains(target)) return;
    if (!lastPointer || !target || !editor.view.dom.contains(target)) { pointerBlockId = null; return; }
    const hit = editor.view.posAtCoords({ left: lastPointer.x, top: lastPointer.y });
    const block = hit && topLevelBlock(editor.state.doc, hit.pos);
    const dom = block && editor.view.nodeDOM(block.pos);
    const rect = dom instanceof HTMLElement ? dom.getBoundingClientRect() : null;
    if (!block || !rect || lastPointer.y < rect.top || lastPointer.y > rect.bottom || lastPointer.x < rect.left || lastPointer.x > rect.right) { pointerBlockId = null; return; }
    if (blockMenu && block.node.attrs.id !== handle?.id) { pointerBlockId = null; return; }
    pointerBlockId = block.node.attrs.id;
    if (!blockMenu) placeHandle(block.pos);
  }
  function hoverBlock(event: PointerEvent) {
    if (!pageStyle) return;
    if (event.pointerType === 'touch') { clearHover(); return; }
    lastPointer = { x: event.clientX, y: event.clientY };
    updateHoveredBlock();
  }
  function selectHandle() {
    if (!editor || !handle) return;
    const block = blockById(editor.state.doc, handle.id);
    if (block) editor.commands.setNodeSelection(block.pos);
  }
  function openBlockMenu() {
    if (disabled || !editor || !handle) return;
    selectHandle(); pageMenu = null; panel = null; bubble = null; blockMenu = !blockMenu;
    const rect = blockButton?.getBoundingClientRect() ?? selectionRect();
    if (rect) panelPosition = popoverPosition(rect, 240, 275);
    if (blockMenu) queueMicrotask(() => shell.querySelector<HTMLButtonElement>('.page-block-menu button:not(:disabled)')?.focus());
  }
  function turnBlock() {
    if (!editor || !handle) return;
    const block = blockById(editor.state.doc, handle.id);
    if (!block?.node.isTextblock) return;
    editor.commands.setTextSelection(block.pos + 1);
    openCommands('turn', blockButton);
  }
  function blockAction(action: 'duplicate' | 'delete' | 'up' | 'down') {
    if (!editor || !handle || disabled) return;
    const current = editor, block = blockById(current.state.doc, handle.id);
    if (!block) return;
    blockMenu = false;
    if (action === 'up' || action === 'down') { movePageBlock(current, handle.id, block.index + (action === 'up' ? -1 : 2)); return; }
    const tr = closeHistory(current.state.tr);
    if (action === 'duplicate') {
      const pos = block.pos + block.node.nodeSize;
      tr.insert(pos, current.schema.nodeFromJSON(copyWithNewIds(block.node.toJSON())));
      tr.setSelection(NodeSelection.create(tr.doc, pos));
    } else {
      tr.delete(block.pos, block.pos + block.node.nodeSize);
      tr.setSelection(TextSelection.near(tr.doc.resolve(Math.min(block.pos, tr.doc.content.size))));
    }
    current.view.dispatch(tr.scrollIntoView()); current.view.focus();
  }
  function startDrag(event: DragEvent) {
    if (!editor || disabled || !handle || !event.dataTransfer) { event.preventDefault(); return; }
    draggingId = dragCandidate ?? handle.id; blockMenu = false; pageMenu = null; bubble = null;
    event.dataTransfer.setData('application/x-noteka-block', draggingId); event.dataTransfer.effectAllowed = 'move';
    const block = blockById(editor.state.doc, draggingId), dom = block && editor.view.nodeDOM(block.pos);
    if (dom instanceof HTMLElement) event.dataTransfer.setDragImage(dom, 0, 0);
  }
  function dragOver(event: DragEvent) {
    if (!editor || !draggingId || disabled) return;
    lastPointer = { x: event.clientX, y: event.clientY };
    const hit = editor.view.posAtCoords({ left: event.clientX, top: event.clientY });
    if (!hit) return;
    const block = topLevelBlock(editor.state.doc, hit.pos), dom = block && editor.view.nodeDOM(block.pos);
    if (!block || !(dom instanceof HTMLElement)) return;
    event.preventDefault(); if (event.dataTransfer) event.dataTransfer.dropEffect = 'move';
    const rect = dom.getBoundingClientRect(), after = event.clientY > (rect.top + rect.bottom) / 2;
    dropTarget = { index: block.index + Number(after), top: (after ? rect.bottom : rect.top) - shell.getBoundingClientRect().top };
  }
  function dropBlock(event: DragEvent) {
    if (!editor || !draggingId || !dropTarget) return false;
    event.preventDefault(); event.stopPropagation();
    if (!disabled) movePageBlock(editor, draggingId, dropTarget.index);
    draggingId = null; dropTarget = null; return true;
  }
  function openPanel(kind: 'link' | 'attachment', fromSlash = false) {
    if (!editor || disabled) return;
    savedRange = { from: editor.state.selection.from, to: editor.state.selection.to };
    insertionRange = fromSlash ? { from: slashStart, to: editor.state.selection.from } : null;
    link = kind === 'link' ? editor.getAttributes('link').href ?? '' : link;
    const rect = selectionRect();
    if (rect) panelPosition = popoverPosition(rect, 330, 180);
    pageMenu = null; blockMenu = false; bubble = null; panelError = ''; panel = kind;
    queueMicrotask(() => { panelInput?.focus(); panelInput?.select(); });
  }
  function applyPageLink() {
    if (!editor || disabled || !savedRange) return;
    const value = link.trim();
    let valid = !value;
    if (value && !/[\x00-\x1f\x7f]/.test(value)) {
      try { const url = new URL(value); valid = (['http:', 'https:'].includes(url.protocol) && !!url.hostname) || (url.protocol === 'mailto:' && !!url.pathname); } catch { valid = false; }
    }
    if (!valid) { panelError = 'Use a complete http, https or mailto link.'; return; }
    editor.commands.setTextSelection(savedRange);
    link = value; setLink(); closeMenus();
  }
  function pageKeydown(event: KeyboardEvent) {
    if (event.key !== 'Escape') return;
    if (pageMenu || blockMenu || panel || picker) {
      event.preventDefault(); event.stopPropagation(); picker = false; pickerRange = null; closeMenus();
    }
  }
  function blockMenuKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); blockMenu = false; blockButton?.focus(); return; }
    if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) return;
    event.preventDefault();
    const buttons = [...shell.querySelectorAll<HTMLButtonElement>('.page-block-menu button:not(:disabled)')];
    let index = buttons.indexOf(document.activeElement as HTMLButtonElement);
    index = event.key === 'Home' ? 0 : event.key === 'End' ? buttons.length - 1 : (index + (event.key === 'ArrowDown' ? 1 : -1) + buttons.length) % buttons.length;
    buttons[index]?.focus();
  }
  async function listPicker(cursor?: string) {
    const token = generation;
    pickerBusy = true; pickerError = '';
    try {
      const result = await operate<Contract['collections']>({ operation: 'collection.list', input: { limit: 50, cursor } }, workspace);
      if (token !== generation) return;
      pickerCollections = cursor ? [...pickerCollections, ...result.items] : result.items; pickerCursor = result.next_cursor ?? null;
    } catch (e) { if (token === generation) pickerError = String(e); }
    finally { if (token === generation) pickerBusy = false; }
  }
  async function openViewPicker(fromSlash = false) {
    if (!editor || disabled) return;
    const rect = pageStyle ? selectionRect() : null;
    if (rect) menuPosition = popoverPosition(rect, 320, 380);
    pickerRange = fromSlash ? { from: slashStart, to: editor.state.selection.from } : null;
    slash = false; pageMenu = null; bubble = null; picker = true; pickerCollection = ''; pickerView = ''; pickerViews = [];
    await listPicker();
  }
  async function loadPickerViews() {
    const token = generation; pickerBusy = true; pickerViews = []; pickerView = ''; pickerError = '';
    try {
      if (!pickerCollection) return;
      const bundle = await operate<CollectionBundle>({ operation: 'collection.get', input: { id: pickerCollection } }, workspace);
      if (token === generation) pickerViews = bundle.views;
    } catch (e) { if (token === generation) pickerError = String(e); }
    finally { if (token === generation) pickerBusy = false; }
  }
  function insertView() {
    if (!editor || disabled || pickerBusy || !pickerCollection || !pickerView) return;
    const chain = pageStyle ? editor.chain() : editor.chain().focus();
    chain.command(({ tr }) => { closeHistory(tr); return true; });
    if (pickerRange) chain.deleteRange(pickerRange);
    chain.insertContent({ type: 'collectionView', attrs: { collection_id: pickerCollection, view_id: pickerView } }).run();
    picker = false; pickerRange = null;
    if (pageStyle) editor.view.focus();
  }
  function removeView() {
    if (!editor || disabled || !viewSelected) return;
    editor.chain().focus().command(({ tr }) => { closeHistory(tr); return true; }).deleteSelection().run();
  }
  function collectionNode() {
    return Node.create({
      name: 'collectionView', group: 'block', atom: true, draggable: true,
      addAttributes() { return { collection_id: { default: null }, view_id: { default: null } }; },
      renderHTML({ node }) { return ['div', { 'data-collection-view': node.attrs.view_id }, "Local view reference"]; },
      addNodeView() { return ({ node, editor: current, getPos }) => {
        const dom = document.createElement('div'); dom.className = 'collection-view-node'; dom.contentEditable = 'false';
        const select = document.createElement('button'); select.type = 'button'; select.textContent = "Select embedded view";
        select.addEventListener('click', () => { const pos = getPos(); if (typeof pos === 'number' && !disabled) current.chain().setNodeSelection(pos).focus().run(); });
        dom.append(select); const target = document.createElement('div'); dom.append(target);
        const lifetime = new ReadLifetime();
        const component = mount(EmbeddedCollectionView, { target, props: { workspace, collectionId: node.attrs.collection_id, viewId: node.attrs.view_id, queue: reads, lifetime, onopen: (entity: Entity) => onopen(entity) } });
        return {
          dom,
          update(next) { return next.type === node.type && next.attrs.id === node.attrs.id && next.attrs.collection_id === node.attrs.collection_id && next.attrs.view_id === node.attrs.view_id; },
          stopEvent(event) { return event.target instanceof HTMLElement && !!event.target.closest('button,input,select,textarea,a'); },
          ignoreMutation(record) { return record.type !== 'selection'; },
          destroy() { lifetime.dispose(); void unmount(component); },
        };
      }; },
    });
  }
  function attachmentNode(name: 'image' | 'file') {
    return Node.create({
      name, group: 'block', atom: true, draggable: true,
      addAttributes() { return { attachment_id: { default: null }, ...(name === 'image' ? { alt: { default: '' } } : { name: { default: '' } }) }; },
      // No parseHTML: el portapapeles no puede convertir rutas/URLs externas en originales.
      renderHTML({ node }) { return ['figure', { 'data-attachment': node.attrs.attachment_id }, name === 'image' ? node.attrs.alt || 'Imagen local' : node.attrs.name || "Local file"]; },
      addNodeView() { return ({ node }) => {
        const dom = document.createElement('figure'); dom.className = 'managed-attachment'; dom.contentEditable = 'false';
        const caption = document.createElement('figcaption'); caption.textContent = name === 'image' ? node.attrs.alt || 'Imagen local' : `▧ ${node.attrs.name || "Local file"}`; dom.append(caption);
        let disposed = false;
        if (name === 'image') void operate<AttachmentData>({ operation: 'attachment.read', input: { id: node.attrs.attachment_id } }, workspace).then(data => {
          if (disposed) return;
          if (!['image/png', 'image/jpeg', 'image/gif', 'image/webp'].includes(data.attachment.mime)) throw new Error("Unsupported image format.");
          const img = document.createElement('img'); img.alt = node.attrs.alt || data.attachment.name; img.src = `data:${data.attachment.mime};base64,${data.base64}`; dom.prepend(img);
        }).catch(e => { if (!disposed) caption.textContent = `Original unavailable: ${String(e)}`; });
        return { dom, destroy() { disposed = true; } };
      }; },
    });
  }
  async function attach() {
    if (!editor || disabled || importing) return;
    const current = editor, token = generation;
    importing = true;
    try {
      const a = await operate<Attachment>({ operation: 'attachment.import', input: { source_path: attachmentPath.trim() } }, workspace);
      if (token !== generation || current.isDestroyed) return;
      const image = ['image/png', 'image/jpeg', 'image/gif', 'image/webp'].includes(a.mime);
      const selection = current.state.selection;
      const position = pageStyle && panel === 'attachment' && savedRange ? insertionRange ?? savedRange : selection instanceof NodeSelection ? selection.to : { from: selection.from, to: selection.to };
      current.chain().focus().command(({ tr }) => { closeHistory(tr); return true; }).insertContentAt(position, { type: image ? 'image' : 'file', attrs: { attachment_id: a.id, ...(image ? { alt: a.name } : { name: a.name }) } }).run();
      attachmentPath = '';
      if (pageStyle) { panel = null; insertionRange = null; current.view.focus(); }
    } catch (e) { if (pageStyle) panelError = e instanceof Error ? e.message : String(e); else onerror(e instanceof Error ? e.message : String(e)); }
    finally { importing = false; }
  }
  function setLink() {
    if (!editor) return;
    if (!link) editor.chain().focus().extendMarkRange('link').unsetLink().run();
    else if (/^(https?:\/\/|mailto:)/.test(link) && !/[\x00-\x1f\x7f]/.test(link)) editor.chain().focus().extendMarkRange('link').setLink({ href: link }).run();
    else onerror("Use an http, https or mailto link; links do not open while editing.");
  }
  onMount(() => {
    documentVersion = initial.schema_version;
    const visibleDocument = showStarters && initial.blocks.length === 0
      ? { ...initial, blocks: [{ id: crypto.randomUUID(), type: 'paragraph' as const, content: [] }] }
      : initial;
    editor = new Editor({
      element, content: toEditor(visibleDocument), editable: !disabled,
      extensions: [
        StarterKit.configure({ heading: { levels: [1, 2, 3] }, link: { openOnClick: false, autolink: false, linkOnPaste: false, protocols: ['http', 'https', 'mailto'] }, underline: false, trailingNode: false }),
        TaskList, TaskItem.configure({ nested: true }), attachmentNode('image'), attachmentNode('file'), collectionNode(), SourceLocator, DocumentTableView,
        UniqueID.configure({ types: ['paragraph', 'heading', 'bulletList', 'orderedList', 'listItem', 'taskList', 'taskItem', 'blockquote', 'horizontalRule', 'codeBlock', 'image', 'file', 'collectionView'], generateID: () => crypto.randomUUID() }),
      ],
      editorProps: {
        attributes: { class: 'noteka-document', role: 'textbox', 'aria-label': label, 'aria-multiline': 'true', ...(placeholder ? { 'aria-placeholder': placeholder } : {}) },
        handleKeyDown(view, e) {
          if (pageStyle) {
            if (e.isComposing) return false;
            if (pageMenu && commandKeydown(e)) return true;
            if (e.altKey && e.shiftKey && e.code === 'KeyB') { e.preventDefault(); placeHandle(view.state.selection.from); openBlockMenu(); return true; }
            if (e.altKey && e.shiftKey && e.code === 'KeyF' && !view.state.selection.empty) { e.preventDefault(); queueMicrotask(() => shell.querySelector<HTMLButtonElement>('.selection-toolbar button')?.focus()); return true; }
            if (e.key === 'Escape' && (pageMenu || blockMenu || panel)) { closeMenus(); return true; }
            return false;
          }
          if (e.key === '/' && !e.metaKey && !e.ctrlKey && view.state.selection.$from.parent.type.name !== 'codeBlock') { slashStart = view.state.selection.from; slash = true; }
          else if (e.key === 'Escape') { slash = false; }
          return false;
        },
        handlePaste(_view, event) {
          // Texto plano evita imágenes externas, IDs forjados y esquemas HTML no soportados.
          const plain = event.clipboardData?.getData('text/plain');
          if (plain !== undefined) { editor?.commands.insertContent(plain.split(/\r?\n/).map(text => ({ type: 'paragraph', content: text ? [{ type: 'text', text }] : [] }))); return true; }
          return true;
        },
        handleDrop(_view, event, _slice, moved) { if (pageStyle && draggingId) { dropBlock(event); draggingId = null; dropTarget = null; return true; } if (moved) return false; event.preventDefault(); onerror("Import files using an explicit local path; URLs and clipboard images are not downloaded."); return true; },
      },
      onFocus() { if (showStarters) starterChosen = true; notifyTransaction(); },
      onBlur: notifyTransaction,
      onUpdate({ editor }) { bodyEmpty = canStartBody(editor); try { const document = fromEditor(editor.getJSON(), documentVersion); documentVersion = document.schema_version; onchange(document); } catch (e) { oninvalid(JSON.stringify(editor.getJSON(), null, 2), String(e)); } },
      onTransaction({ transaction }) {
        if (transaction.docChanged) {
          const map = (range: { from: number; to: number } | null) => range ? { from: transaction.mapping.map(range.from), to: transaction.mapping.map(range.to) } : null;
          savedRange = map(savedRange); insertionRange = map(insertionRange); pickerRange = map(pickerRange);
        }
        notifyTransaction();
      },
    });
    bodyEmpty = canStartBody(editor);
    function viewportChanged(event: Event) {
      if (event.target instanceof Element && event.target.closest('.editor-popover')) return;
      if (pageStyle) { if (panel || blockMenu || pageMenu === 'insert' || pageMenu === 'turn') closeMenus(false); syncPageUI(); }
    }
    document.addEventListener('scroll', viewportChanged, true);
    window.addEventListener('resize', viewportChanged);
    document.addEventListener('pointermove', hoverBlock);
    document.addEventListener('pointerleave', clearHover);
    window.addEventListener('blur', clearHover);
    shell.addEventListener('dragover', dragOver);
    shell.addEventListener('drop', dropBlock);
    return () => { generation++; document.removeEventListener('scroll', viewportChanged, true); window.removeEventListener('resize', viewportChanged); document.removeEventListener('pointermove', hoverBlock); document.removeEventListener('pointerleave', clearHover); window.removeEventListener('blur', clearHover); shell.removeEventListener('dragover', dragOver); shell.removeEventListener('drop', dropBlock); editor?.destroy(); };
  });
  $effect(() => { editor?.setEditable(!disabled, false); if (disabled) { bubble = null; pageMenu = null; blockMenu = false; panel = null; handle = null; } });
  $effect(() => {
    if (!editor || !pageStyle) return;
    const dom = editor.view.dom;
    if (pageMenu === 'slash') {
      dom.setAttribute('aria-controls', menuId);
      if (menuItems[menuIndex]) dom.setAttribute('aria-activedescendant', `${menuId}-option-${menuIndex}`);
      else dom.removeAttribute('aria-activedescendant');
    } else { dom.removeAttribute('aria-controls'); dom.removeAttribute('aria-activedescendant'); }
  });
  $effect(() => {
    const current = editor, id = focusBlock;
    if (!current || !id) return;
    untrack(() => {
      current.state.doc.descendants((node, pos) => {
        if (node.attrs.id !== id) return;
        current.view.dispatch(current.state.tr.setMeta(sourceKey, id));
        if (node.isTextblock) current.chain().setTextSelection(pos + 1).focus().run();
        else current.chain().setNodeSelection(pos).focus().run();
        const dom = current.view.nodeDOM(pos);
        if (dom instanceof HTMLElement) dom.scrollIntoView({ block: 'center' });
      });
    });
  });
</script>

<svelte:window onpointerup={() => { if (!draggingId) dragCandidate = null; }} onpointerdown={(event) => { if (pageStyle && shell && !shell.contains(event.target as globalThis.Node)) { closeMenus(false); picker = false; } }} onkeydown={(event) => { if (pageStyle && shell?.contains(event.target as globalThis.Node)) pageKeydown(event); }} />

<div bind:this={shell} class="block-editor" class:page-editor={pageStyle} data-transaction={serial} data-empty={bodyEmpty} data-started={starterChosen}>
  {#if !pageStyle}
  <details class="format-controls"><summary>Formatting and blocks</summary>
  <div class="format-toolbar" role="toolbar" aria-label="Formatting and blocks">
    <button disabled={disabled} title="Negrita · ⌘B" onclick={() => editor?.chain().focus().toggleBold().run()}><b>B</b></button>
    <button disabled={disabled} title="Cursiva · ⌘I" onclick={() => editor?.chain().focus().toggleItalic().run()}><i>I</i></button>
    <button disabled={disabled} title="Strikethrough" onclick={() => editor?.chain().focus().toggleStrike().run()}>S̶</button>
    {#each blocks as block}<button disabled={disabled} onclick={() => command(block.command)}>{block.label}</button>{/each}
    <button disabled={disabled} onclick={() => editor?.commands.undo()}>Undo</button><button disabled={disabled} onclick={() => editor?.commands.redo()}>Redo</button>
    <button disabled={disabled || !viewSelected} onclick={removeView}>Remove selected view</button>
  </div>
  <details class="editor-tools"><summary>Insert link or attachment</summary>
    <label>Link for selected text<input aria-label="Link" bind:value={link} placeholder="https://…" disabled={disabled} /></label><button disabled={disabled} onclick={setLink}>Aplicar enlace</button>
    <label>Original local<input aria-label="Attachment path" bind:value={attachmentPath} placeholder="/path/to/file" disabled={disabled || importing} /></label><button disabled={disabled || importing || !attachmentPath.trim()} onclick={() => void attach()}>{importing ? 'Copiando…' : "Insert attachment"}</button>
  </details>
  </details>
  {/if}
  {#if pageStyle && handle && !disabled}
    <div class="block-handle-group" class:is-visible={pointerBlockId === handle.id || !!draggingId} data-block-id={handle.id} style:top={`${handle.top}px`} role="group" aria-label="Block controls">
      <button type="button" class="block-add" aria-label="Add block below" title="Add a block below" onmousedown={event => event.preventDefault()} onclick={addBlockBelow}><AppIcon name="plus" size={18} /></button>
      <button bind:this={blockButton} type="button" class="block-grip" aria-label="Block actions" aria-haspopup="menu" aria-expanded={blockMenu} title="Drag to move · Click for actions · Alt+Shift+B" draggable="true" onpointerdown={() => { dragCandidate = handle?.id ?? null; }} ondragstart={startDrag} ondragend={() => { draggingId = null; dragCandidate = null; dropTarget = null; }} onclick={openBlockMenu}><AppIcon name="grip" size={18} /></button>
    </div>
  {/if}
  <div class="editor-surface" data-empty={bodyEmpty} data-placeholder={placeholder}><div bind:this={element}></div></div>
  {#if showStarters && bodyEmpty && !starterChosen}<div class="task-body-starters" role="group" aria-label={starterContext === 'note' ? "Start note content" : "Start task content"}><p>Press Enter in the title to begin, or choose a structure.</p><button type="button" disabled={disabled} onclick={() => startBody('empty')}><AppIcon name="file-text" size={18} />Empty page</button><button type="button" disabled={disabled} onclick={() => startBody('checklist')}><AppIcon name="list-checks" size={18} />{starterContext === 'note' ? 'Checklist' : "Checklist"}</button><button type="button" disabled={disabled} onclick={() => startBody('notes')}><AppIcon name="notebook" size={18} />{starterContext === 'note' ? "Note with sections" : "Task notes"}</button></div>{/if}
  {#if pageStyle && bubble && !disabled}
    <div class="selection-toolbar editor-popover" role="toolbar" aria-label="Text formatting" tabindex="-1" style:left={`${bubble.left}px`} style:top={`${bubble.top}px`} onmousedown={event => event.preventDefault()}>
      <button type="button" class="selection-block-type" aria-haspopup="listbox" onclick={() => openCommands('turn')} title="Turn into">{currentBlock}<AppIcon name="chevron-down" size={13} /></button>
      <span class="toolbar-separator"></span>
      <button type="button" aria-label="Bold" aria-pressed={activeMarks.bold} title="Bold · ⌘B / Ctrl+B" onclick={() => editor?.chain().focus().toggleBold().run()}><b>B</b></button>
      <button type="button" aria-label="Italic" aria-pressed={activeMarks.italic} title="Italic · ⌘I / Ctrl+I" onclick={() => editor?.chain().focus().toggleItalic().run()}><i>I</i></button>
      <button type="button" aria-label="Strikethrough" aria-pressed={activeMarks.strike} title="Strikethrough" onclick={() => editor?.chain().focus().toggleStrike().run()}><s>S</s></button>
      <button type="button" aria-label="Inline code" aria-pressed={activeMarks.code} title="Inline code" onclick={() => editor?.chain().focus().toggleCode().run()}><span class="inline-code-symbol">{'<>'}</span></button>
      <span class="toolbar-separator"></span>
      <button type="button" class="selection-link" aria-label="Edit link" aria-pressed={activeMarks.link} title="Link" onclick={() => openPanel('link')}>Link<AppIcon name="arrow-up-right" size={13} /></button>
    </div>
  {/if}
  {#if pageStyle && pageMenu && !disabled}
    <div bind:this={menuElement} id={menuId} class="page-command-menu editor-popover" role="listbox" aria-label="Block commands" tabindex="0" aria-activedescendant={menuItems[menuIndex] ? `${menuId}-option-${menuIndex}` : undefined} style:left={`${menuPosition.left}px`} style:top={`${menuPosition.top}px`} onkeydown={commandKeydown} onmousedown={event => event.preventDefault()}>
      <div class="command-menu-heading">{pageMenu === 'turn' ? 'Turn into' : menuQuery ? `Results for “${menuQuery}”` : 'Basic blocks'}</div>
      {#each menuItems as block, index (block.command)}
        <button type="button" role="option" id={`${menuId}-option-${index}`} tabindex="-1" aria-selected={index === menuIndex} onmouseenter={() => menuIndex = index} onclick={() => chooseCommand(block.command)}>
          <span class="command-block-symbol" aria-hidden="true">{block.icon}</span><span><strong>{block.label}</strong><small>{block.description}</small></span>
        </button>
      {:else}<p class="command-empty">No matching blocks.</p>{/each}
      <div class="command-menu-footer"><span>↑↓ navigate · Enter select</span><button type="button" tabindex="-1" onclick={() => closeMenus()}>Esc</button></div>
    </div>
  {/if}
  {#if pageStyle && blockMenu && handle && !disabled}
    <div class="page-block-menu editor-popover" role="menu" aria-label="Block actions" tabindex="-1" style:left={`${panelPosition.left}px`} style:top={`${panelPosition.top}px`} onkeydown={blockMenuKeydown}>
      <div class="command-menu-heading">Block actions</div>
      <button type="button" role="menuitem" disabled={!handle.convertible} onclick={turnBlock}><span>Turn into</span><AppIcon name="chevron-right" size={14} /></button>
      <button type="button" role="menuitem" onclick={() => blockAction('duplicate')}><AppIcon name="file-text" size={16} /><span>Duplicate</span></button>
      <button type="button" role="menuitem" disabled={handle.index === 0} onclick={() => blockAction('up')}><AppIcon name="arrow-up" size={16} /><span>Move up</span></button>
      <button type="button" role="menuitem" disabled={handle.index === (editor?.state.doc.childCount ?? 0) - 1} onclick={() => blockAction('down')}><AppIcon name="arrow-down" size={16} /><span>Move down</span></button>
      <button type="button" role="menuitem" onclick={addBlockBelow}><AppIcon name="plus" size={16} /><span>Insert below</span></button>
      <button type="button" role="menuitem" class="block-delete" onclick={() => blockAction('delete')}><AppIcon name="close" size={16} /><span>Delete</span></button>
    </div>
  {/if}
  {#if pageStyle && panel && !disabled}
    <form class="editor-link-panel editor-popover" aria-label={panel === 'link' ? 'Edit link' : 'Insert attachment'} style:left={`${panelPosition.left}px`} style:top={`${panelPosition.top}px`} onsubmit={event => { event.preventDefault(); if (panel === 'link') applyPageLink(); else void attach(); }}>
      <div class="editor-panel-heading"><strong>{panel === 'link' ? 'Link' : 'Image or file'}</strong><button type="button" aria-label="Close editor panel" disabled={importing} onclick={() => closeMenus()}><AppIcon name="close" size={15} /></button></div>
      {#if panel === 'link'}
        <label for={`${menuId}-link`}>Paste an http, https or mailto link</label><input bind:this={panelInput} id={`${menuId}-link`} aria-label="Link" bind:value={link} placeholder="https://…" autocomplete="off" />
        <div class="editor-panel-actions"><button type="button" onclick={() => { link = ''; applyPageLink(); }}>Remove link</button><button type="submit" class="primary">Apply link</button></div>
      {:else}
        <label for={`${menuId}-attachment`}>Local file path</label><input bind:this={panelInput} id={`${menuId}-attachment`} aria-label="Attachment path" bind:value={attachmentPath} placeholder="/path/to/file" disabled={importing} />
        <div class="editor-panel-actions"><button type="button" disabled={importing} onclick={() => closeMenus()}>Cancel</button><button type="submit" class="primary" disabled={importing || !attachmentPath.trim()}>{importing ? 'Importing…' : 'Insert attachment'}</button></div>
      {/if}
      {#if panelError}<p class="editor-panel-error" role="alert">{panelError}</p>{/if}
    </form>
  {/if}
  {#if pageStyle && dropTarget}<div class="block-drop-line" style:top={`${dropTarget.top}px`}></div>{/if}
  {#if picker}<section class="view-picker" class:editor-popover={pageStyle} class:page-view-picker={pageStyle} style:left={pageStyle ? `${menuPosition.left}px` : undefined} style:top={pageStyle ? `${menuPosition.top}px` : undefined} aria-label="Insert local view">
    <label>Collection<select aria-label="Collection to embed" disabled={disabled || pickerBusy} bind:value={pickerCollection} onchange={() => void loadPickerViews()}><option value="">Choose a collection</option>{#each pickerCollections as c (c.id)}<option value={c.id}>{c.name}</option>{/each}</select></label>
    {#if pickerCursor}<button disabled={pickerBusy} onclick={() => void listPicker(pickerCursor ?? undefined)}>More collections to embed</button>{/if}
    <label>Saved view<select aria-label="View to embed" disabled={disabled || pickerBusy} bind:value={pickerView}><option value="">Choose a view</option>{#each pickerViews as v (v.id)}<option value={v.id}>{v.name} · {v.layout}</option>{/each}</select></label>
    <button disabled={disabled || pickerBusy || !pickerView} onclick={insertView}>Insert saved view</button><button onclick={() => { picker = false; pickerRange = null; if (pageStyle) closeMenus(); }}>Cancel insertion</button>
    {#if pickerError}<p class="error" role="alert">{pickerError}</p>{/if}<p class="metadata">References the original only. Choosing without inserting does not change the note.</p>
  </section>{/if}
  {#if slash && !pageStyle}<div class="slash-menu" role="toolbar" aria-label="Block commands">{#each blocks as block}<button disabled={disabled} onclick={() => command(block.command, true)}>{block.label}</button>{/each}<button onclick={() => slash = false}>Cancel /</button></div>{/if}
  {#if !pageStyle}
    <button class="add-paragraph" disabled={disabled} onclick={() => editor?.chain().focus('end').insertContentAt(editor.state.doc.content.size, { type: 'paragraph' }).run()}><AppIcon name="plus" size={14} />Add paragraph</button>
    <small class="editor-hint">/ for blocks · ⌘Z to undo</small>
  {/if}
</div>
