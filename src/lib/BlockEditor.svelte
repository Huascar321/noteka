<script lang="ts">
  import { onMount, untrack, mount, unmount } from 'svelte';
  import { Editor, Node } from '@tiptap/core';
  import { NodeSelection } from '@tiptap/pm/state';
  import { closeHistory } from '@tiptap/pm/history';
  import StarterKit from '@tiptap/starter-kit';
  import UniqueID from '@tiptap/extension-unique-id';
  import TaskList from '@tiptap/extension-task-list';
  import TaskItem from '@tiptap/extension-task-item';
  import { toEditor, fromEditor } from './editor-adapter';
  import { SourceLocator, sourceKey } from './source-locator';
  import { operate } from './api';
  import EmbeddedCollectionView from './EmbeddedCollectionView.svelte';
  import { ReadQueue, ReadLifetime } from './embedded-reads';
  import type { Attachment, AttachmentData, Document, Entity, Collection, CollectionBundle, CollectionView, Contract } from './contracts.generated';

  let { initial, disabled = false, workspace, focusBlock, onchange, onerror, oninvalid, onopen }: {
    initial: Document; disabled?: boolean; workspace: string;
    focusBlock?: string;
    onchange: (document: Document) => void; onerror: (message: string) => void;
    oninvalid: (raw: string, message: string) => void;
    onopen: (entity: Entity) => Promise<void>;
  } = $props();
  let element: HTMLDivElement;
  let editor = $state.raw<Editor>();
  let slash = $state(false);
  let slashStart = 0;
  let attachmentPath = $state('');
  let link = $state('');
  let importing = $state(false);
  let generation = 0;
  let serial = $state(0);
  let documentVersion = 2;
  const reads = new ReadQueue(2);
  let picker = $state(false), pickerBusy = $state(false), pickerError = $state('');
  let pickerCollections = $state<Collection[]>([]), pickerCursor = $state<string | null>(null);
  let pickerCollection = $state(''), pickerView = $state(''), pickerViews = $state<CollectionView[]>([]);
  let pickerRange: { from: number; to: number } | null = null;
  const viewSelected = $derived.by(() => { serial; return editor?.state.selection instanceof NodeSelection && editor.state.selection.node.type.name === 'collectionView'; });

  const blocks = [
    { label: 'Párrafo', command: 'paragraph' }, { label: 'Encabezado', command: 'heading' },
    { label: 'Lista', command: 'bullet' }, { label: 'Lista numerada', command: 'ordered' },
    { label: 'Checklist', command: 'task' }, { label: 'Cita', command: 'quote' },
    { label: 'Código', command: 'code' }, { label: 'Divisor', command: 'rule' },
    { label: 'Base incrustada', command: 'view' },
  ];
  function command(name: string, fromSlash = false) {
    if (!editor || disabled) return;
    if (name === 'view') { void openViewPicker(fromSlash); return; }
    let c = editor.chain().focus();
    if (fromSlash) c = c.deleteRange({ from: slashStart, to: editor.state.selection.from });
    slash = false;
    switch (name) {
      case 'paragraph': c.setParagraph().run(); break;
      case 'heading': c.toggleHeading({ level: 2 }).run(); break;
      case 'bullet': c.toggleBulletList().run(); break;
      case 'ordered': c.toggleOrderedList().run(); break;
      case 'task': c.toggleTaskList().run(); break;
      case 'quote': c.toggleBlockquote().run(); break;
      case 'code': c.toggleCodeBlock().run(); break;
      case 'rule': c.setHorizontalRule().run(); break;
    }
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
    pickerRange = fromSlash ? { from: slashStart, to: editor.state.selection.from } : null;
    slash = false; picker = true; pickerCollection = ''; pickerView = ''; pickerViews = [];
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
    const chain = editor.chain().focus();
    chain.command(({ tr }) => { closeHistory(tr); return true; });
    if (pickerRange) chain.deleteRange(pickerRange);
    chain.insertContent({ type: 'collectionView', attrs: { collection_id: pickerCollection, view_id: pickerView } }).run();
    picker = false; pickerRange = null;
  }
  function removeView() {
    if (!editor || disabled || !viewSelected) return;
    editor.chain().focus().command(({ tr }) => { closeHistory(tr); return true; }).deleteSelection().run();
  }
  function collectionNode() {
    return Node.create({
      name: 'collectionView', group: 'block', atom: true, draggable: true,
      addAttributes() { return { collection_id: { default: null }, view_id: { default: null } }; },
      renderHTML({ node }) { return ['div', { 'data-collection-view': node.attrs.view_id }, 'Referencia a vista local']; },
      addNodeView() { return ({ node, editor: current, getPos }) => {
        const dom = document.createElement('div'); dom.className = 'collection-view-node'; dom.contentEditable = 'false';
        const select = document.createElement('button'); select.type = 'button'; select.textContent = 'Seleccionar vista incrustada';
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
      renderHTML({ node }) { return ['figure', { 'data-attachment': node.attrs.attachment_id }, name === 'image' ? node.attrs.alt || 'Imagen local' : node.attrs.name || 'Archivo local']; },
      addNodeView() { return ({ node }) => {
        const dom = document.createElement('figure'); dom.className = 'managed-attachment'; dom.contentEditable = 'false';
        const caption = document.createElement('figcaption'); caption.textContent = name === 'image' ? node.attrs.alt || 'Imagen local' : `▧ ${node.attrs.name || 'Archivo local'}`; dom.append(caption);
        let disposed = false;
        if (name === 'image') void operate<AttachmentData>({ operation: 'attachment.read', input: { id: node.attrs.attachment_id } }, workspace).then(data => {
          if (disposed) return;
          if (!['image/png', 'image/jpeg', 'image/gif', 'image/webp'].includes(data.attachment.mime)) throw new Error('Formato de imagen no admitido.');
          const img = document.createElement('img'); img.alt = node.attrs.alt || data.attachment.name; img.src = `data:${data.attachment.mime};base64,${data.base64}`; dom.prepend(img);
        }).catch(e => { if (!disposed) caption.textContent = `Original no disponible: ${String(e)}`; });
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
      const position = selection instanceof NodeSelection ? selection.to : { from: selection.from, to: selection.to };
      current.chain().focus().insertContentAt(position, { type: image ? 'image' : 'file', attrs: { attachment_id: a.id, ...(image ? { alt: a.name } : { name: a.name }) } }).run();
      attachmentPath = '';
    } catch (e) { onerror(e instanceof Error ? e.message : String(e)); }
    finally { importing = false; }
  }
  function setLink() {
    if (!editor) return;
    if (!link) editor.chain().focus().extendMarkRange('link').unsetLink().run();
    else if (/^(https?:\/\/|mailto:)/.test(link) && !/[\x00-\x1f\x7f]/.test(link)) editor.chain().focus().extendMarkRange('link').setLink({ href: link }).run();
    else onerror('Usá un enlace http, https o mailto; no se abren al editar.');
  }
  onMount(() => {
    documentVersion = initial.schema_version;
    editor = new Editor({
      element, content: toEditor(initial), editable: !disabled,
      extensions: [
        StarterKit.configure({ heading: { levels: [1, 2, 3] }, link: { openOnClick: false, autolink: false, linkOnPaste: false, protocols: ['http', 'https', 'mailto'] }, underline: false, trailingNode: false }),
        TaskList, TaskItem.configure({ nested: true }), attachmentNode('image'), attachmentNode('file'), collectionNode(), SourceLocator,
        UniqueID.configure({ types: ['paragraph', 'heading', 'bulletList', 'orderedList', 'listItem', 'taskList', 'taskItem', 'blockquote', 'horizontalRule', 'codeBlock', 'image', 'file', 'collectionView'], generateID: () => crypto.randomUUID() }),
      ],
      editorProps: {
        attributes: { class: 'noteka-document', role: 'textbox', 'aria-label': 'Contenido de la nota', 'aria-multiline': 'true' },
        handleKeyDown(view, e) {
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
        handleDrop(_view, event, _slice, moved) { if (moved) return false; event.preventDefault(); onerror('Importá archivos por una ruta local explícita; no se descargan URLs ni imágenes del portapapeles.'); return true; },
      },
      onUpdate({ editor }) { try { const document = fromEditor(editor.getJSON(), documentVersion); documentVersion = document.schema_version; onchange(document); } catch (e) { oninvalid(JSON.stringify(editor.getJSON(), null, 2), String(e)); } },
      onTransaction() { serial++; },
    });
    return () => { generation++; editor?.destroy(); };
  });
  $effect(() => { editor?.setEditable(!disabled, false); });
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

<div class="block-editor" data-transaction={serial}>
  <div class="format-toolbar" role="toolbar" aria-label="Formato y bloques">
    <button disabled={disabled} title="Negrita · ⌘B" onclick={() => editor?.chain().focus().toggleBold().run()}><b>B</b></button>
    <button disabled={disabled} title="Cursiva · ⌘I" onclick={() => editor?.chain().focus().toggleItalic().run()}><i>I</i></button>
    <button disabled={disabled} title="Tachado" onclick={() => editor?.chain().focus().toggleStrike().run()}>S̶</button>
    {#each blocks as block}<button disabled={disabled} onclick={() => command(block.command)}>{block.label}</button>{/each}
    <button disabled={disabled} onclick={() => editor?.commands.undo()}>Deshacer</button><button disabled={disabled} onclick={() => editor?.commands.redo()}>Rehacer</button>
    <button disabled={disabled || !viewSelected} onclick={removeView}>Quitar vista seleccionada</button>
  </div>
  <div bind:this={element}></div>
  {#if picker}<section class="view-picker" aria-label="Insertar vista local">
    <label>Colección<select aria-label="Colección para incrustar" disabled={disabled || pickerBusy} bind:value={pickerCollection} onchange={() => void loadPickerViews()}><option value="">Elegí colección</option>{#each pickerCollections as c (c.id)}<option value={c.id}>{c.name}</option>{/each}</select></label>
    {#if pickerCursor}<button disabled={pickerBusy} onclick={() => void listPicker(pickerCursor ?? undefined)}>Más colecciones para incrustar</button>{/if}
    <label>Vista guardada<select aria-label="Vista para incrustar" disabled={disabled || pickerBusy} bind:value={pickerView}><option value="">Elegí vista</option>{#each pickerViews as v (v.id)}<option value={v.id}>{v.name} · {v.layout}</option>{/each}</select></label>
    <button disabled={disabled || pickerBusy || !pickerView} onclick={insertView}>Insertar vista guardada</button><button onclick={() => picker = false}>Cancelar inserción</button>
    {#if pickerError}<p class="error" role="alert">{pickerError}</p>{/if}<p class="metadata">Sólo referencia al original. Elegir sin insertar no cambia la nota.</p>
  </section>{/if}
  {#if slash}<div class="slash-menu" role="toolbar" aria-label="Comandos de bloque">{#each blocks as block}<button disabled={disabled} onclick={() => command(block.command, true)}>{block.label}</button>{/each}<button onclick={() => slash = false}>Cancelar /</button></div>{/if}
  <button class="add-paragraph" disabled={disabled} onclick={() => editor?.chain().focus('end').insertContentAt(editor.state.doc.content.size, { type: 'paragraph' }).run()}>+ Añadir párrafo</button>
  <div class="editor-tools">
    <label>Enlace del texto seleccionado<input aria-label="Enlace" bind:value={link} placeholder="https://…" disabled={disabled} /></label><button disabled={disabled} onclick={setLink}>Aplicar enlace</button>
    <label>Original local<input aria-label="Ruta del adjunto" bind:value={attachmentPath} placeholder="/ruta/al/archivo" disabled={disabled || importing} /></label><button disabled={disabled || importing || !attachmentPath.trim()} onclick={() => void attach()}>{importing ? 'Copiando…' : 'Insertar adjunto'}</button>
  </div>
  <small class="metadata">/ para bloques · ⌘Z / ⇧⌘Z · Adjuntos de hasta 64 MiB. Quitar un bloque no borra el original.</small>
</div>
