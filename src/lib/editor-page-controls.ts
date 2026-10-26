import type { Editor, JSONContent } from '@tiptap/core';
import type { Node as ProseNode } from '@tiptap/pm/model';
import { NodeSelection } from '@tiptap/pm/state';
import { closeHistory } from '@tiptap/pm/history';

export const pageBlocks = [
  { command: 'paragraph', label: 'Text', description: 'Just start writing with plain text.', icon: 'T', keywords: 'paragraph texto párrafo' },
  { command: 'heading1', label: 'Heading 1', description: 'A large section heading.', icon: 'H₁', keywords: 'h1 heading1 title heading título encabezado' },
  { command: 'heading2', label: 'Heading 2', description: 'A medium section heading.', icon: 'H₂', keywords: 'h2 heading2 heading título encabezado' },
  { command: 'heading3', label: 'Heading 3', description: 'A small section heading.', icon: 'H₃', keywords: 'h3 heading3 heading título encabezado' },
  { command: 'bullet', label: 'Bulleted list', description: 'Create a simple bulleted list.', icon: '•', keywords: 'bullet list lista viñetas' },
  { command: 'ordered', label: 'Numbered list', description: 'Create a list with numbering.', icon: '1.', keywords: 'number list lista numerada' },
  { command: 'task', label: 'To-do list', description: 'Keep track of tasks with a checklist.', icon: '☑', keywords: 'checklist task todo tarea lista' },
  { command: 'quote', label: 'Quote', description: 'Make a quotation stand out.', icon: '“', keywords: 'quote cita' },
  { command: 'code', label: 'Code', description: 'Write a block of code.', icon: '</>', keywords: 'code código' },
  { command: 'rule', label: 'Divider', description: 'Separate sections with a line.', icon: '―', keywords: 'divider line separador divisor' },
  { command: 'attachment', label: 'Image or file', description: 'Insert an original from your computer.', icon: '▧', keywords: 'image file attachment imagen archivo adjunto' },
  { command: 'view', label: 'Embedded database', description: 'Link an existing local database view.', icon: '▦', keywords: 'database view colección base vista' },
];

export function slashQuery(editor: Editor) {
  const { empty, $from, from } = editor.state.selection;
  if (!empty || !$from.parent.isTextblock || $from.parent.type.spec.code || editor.isActive('link')) return null;
  const before = $from.parent.textBetween(0, $from.parentOffset, '\n', '\ufffc');
  const match = /(?:^|\s)\/([\p{L}\p{N} -]{0,40})$/u.exec(before);
  return match ? { from: from - match[1].length - 1, query: match[1] } : null;
}

export function matchingBlocks(query: string, transform = false) {
  const term = query.trim().toLocaleLowerCase();
  return pageBlocks.filter(block => (!transform || !['rule', 'attachment', 'view'].includes(block.command))
    && `${block.label} ${block.keywords}`.toLocaleLowerCase().includes(term));
}

export function topLevelBlock(doc: ProseNode, position: number) {
  const resolved = doc.resolve(Math.max(0, Math.min(position, doc.content.size)));
  const index = Math.min(resolved.index(0), doc.childCount - 1);
  if (index < 0) return null;
  let pos = 0;
  for (let i = 0; i < index; i++) pos += doc.child(i).nodeSize;
  return { node: doc.child(index), pos, index };
}

export function blockById(doc: ProseNode, id: string): ReturnType<typeof topLevelBlock> {
  let result: ReturnType<typeof topLevelBlock> = null;
  doc.forEach((node, pos, index) => { if (node.attrs.id === id) result = { node, pos, index }; });
  return result;
}

export function copyWithNewIds(node: JSONContent): JSONContent {
  return { ...node, ...(node.attrs ? { attrs: { ...node.attrs, ...('id' in node.attrs ? { id: crypto.randomUUID() } : {}) } } : {}),
    ...(node.content ? { content: node.content.map(copyWithNewIds) } : {}) };
}

export function movePageBlock(editor: Editor, id: string, targetIndex: number) {
  const doc = editor.state.doc, block = blockById(doc, id);
  if (!block || targetIndex < 0 || targetIndex > doc.childCount || targetIndex === block.index || targetIndex === block.index + 1) return false;
  let destination = 0;
  for (let i = 0; i < targetIndex; i++) destination += doc.child(i).nodeSize;
  const tr = closeHistory(editor.state.tr).delete(block.pos, block.pos + block.node.nodeSize);
  const position = tr.mapping.map(destination);
  tr.insert(position, block.node).setSelection(NodeSelection.create(tr.doc, position));
  editor.view.dispatch(tr.scrollIntoView());
  editor.view.focus();
  return true;
}

export function popoverPosition(rect: Pick<DOMRect, 'left' | 'right' | 'top' | 'bottom'>, width: number, height: number, above = false) {
  const gutter = 10, w = Math.min(width, window.innerWidth - gutter * 2), h = Math.min(height, window.innerHeight - gutter * 2);
  const below = rect.bottom + 6, over = rect.top - h - 6;
  const top = above && over >= gutter ? over : below + h <= window.innerHeight - gutter ? below : over >= gutter ? over : Math.max(gutter, window.innerHeight - h - gutter);
  return { left: Math.max(gutter, Math.min(rect.left, window.innerWidth - w - gutter)), top: Math.max(gutter, Math.min(top, window.innerHeight - h - gutter)) };
}
