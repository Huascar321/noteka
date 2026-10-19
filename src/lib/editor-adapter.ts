import type { JSONContent } from '@tiptap/core';
import type { Block, Document, Text } from './contracts.generated';

const names = {
  paragraph: 'paragraph', heading: 'heading', bullet_list: 'bulletList', ordered_list: 'orderedList',
  list_item: 'listItem', task_list: 'taskList', task_item: 'taskItem', blockquote: 'blockquote',
  horizontal_rule: 'horizontalRule', code_block: 'codeBlock', image: 'image', file: 'file',
  collection_view: 'collectionView',
} as const;
const containers = new Set(['bullet_list', 'ordered_list', 'list_item', 'task_list', 'task_item', 'blockquote']);
const keys: Record<string, string[]> = { heading: ['level'], ordered_list: ['start'], task_item: ['checked'], code_block: ['language'], image: ['attachment_id', 'alt'], file: ['attachment_id', 'name'], collection_view: ['collection_id', 'view_id'] };

export function toEditor(document: Document): JSONContent {
  if (![1, 2, 3].includes(document.schema_version)) throw new Error('Versión de documento no soportada.');
  function inline(t: Text): JSONContent | null {
    if (t.type === 'hard_break') return { type: 'hardBreak' };
    if (!t.text) return null;
    return { type: 'text', text: t.text, ...(t.marks?.length ? { marks: t.marks.map(m => ({ type: m.type, ...(m.href ? { attrs: { href: m.href } } : {}) })) } : {}) };
  }
  function block(b: Block): JSONContent {
    if (!(b.type in names)) throw new Error(`Bloque no soportado: ${b.type}`);
    const attributes = b.attrs as Record<string, unknown> | undefined;
    if (b.type === 'collection_view' && (document.schema_version !== 3 || b.content.length || b.children?.length || !attributes || Object.keys(attributes).some(k => !keys.collection_view.includes(k)) || keys.collection_view.some(k => typeof attributes[k] !== 'string' || !attributes[k]))) throw new Error('Referencia de vista no reconocida.');
    const content = containers.has(b.type) ? (b.children ?? []).map(block) : b.content.map(inline).filter((n): n is JSONContent => n !== null);
    return { type: names[b.type], attrs: { ...((b.attrs ?? {}) as Record<string, unknown>), id: b.id }, ...(content.length ? { content } : {}) };
  }
  return { type: 'doc', content: document.blocks.map(block) };
}

export function fromEditor(json: JSONContent, minimumVersion = 2): Document {
  if (json.type !== 'doc') throw new Error('El editor no devolvió un documento.');
  if (![1, 2, 3].includes(minimumVersion)) throw new Error('Versión mínima no soportada.');
  let version = Math.max(2, minimumVersion);
  const ids = new Set<string>();
  function block(n: JSONContent): Block {
    const type = Object.entries(names).find(([, name]) => name === n.type)?.[0] as Block['type'] | undefined;
    if (!type) throw new Error(`Bloque del editor no soportado: ${n.type}`);
    if (type === 'collection_view') {
      if (n.content?.length || Object.keys(n.attrs ?? {}).some(k => !['id', ...keys.collection_view].includes(k)) || keys.collection_view.some(k => typeof n.attrs?.[k] !== 'string' || !n.attrs[k])) throw new Error('Referencia de vista no reconocida.');
      version = 3;
    }
    const id = n.attrs?.id as string;
    if (!id || ids.has(id)) throw new Error('El editor generó un ID vacío o duplicado. No se guardó.');
    ids.add(id);
    const attrs: Record<string, unknown> = {};
    for (const k of keys[type] ?? []) if (n.attrs?.[k] !== undefined) attrs[k] = n.attrs[k];
    const b: Block = { id, type, content: [] };
    if (Object.keys(attrs).length) b.attrs = attrs;
    if (containers.has(type)) b.children = (n.content ?? []).map(block);
    else b.content = (n.content ?? []).map(t => {
      if (t.type === 'hardBreak') return { type: 'hard_break', text: '' };
      if (t.type !== 'text' || t.text === undefined) throw new Error(`Inline no soportado: ${t.type}`);
      return { type: 'text', text: t.text, ...(t.marks?.length ? { marks: t.marks.map(m => {
        if (!['bold', 'italic', 'strike', 'code', 'link'].includes(m.type!)) throw new Error(`Marca no soportada: ${m.type}`);
        return { type: m.type as NonNullable<Text['marks']>[number]['type'], ...(m.type === 'link' ? { href: m.attrs?.href as string } : {}) };
      }) } : {}) };
    });
    return b;
  }
  const blocks = (json.content ?? []).map(block);
  return { schema_version: version, blocks };
}

export function pendingJson(title: string, document: Document): string {
  return JSON.stringify({ title, document }, null, 2);
}
