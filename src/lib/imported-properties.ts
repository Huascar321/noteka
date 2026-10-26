import type { Block, Document } from './contracts.generated';
export type ImportedProperty = { name: string; value: string };
export type ImportedPresentation = { properties: ImportedProperty[]; prefix: Block[]; body: Document };
function inline(block: Block): string {
  return block.content.map(part => part.type === 'hard_break' ? '\n' : part.text).join('');
}
// Recognize only the exact leading structure emitted by import-notion-export.py.
// The prefix is retained verbatim and recomposed on every body edit.
export function importedPresentation(document: Document, sourceCollection?: string): ImportedPresentation {
  const unchanged = { properties: [], prefix: [], body: document };
  if (![2, 3].includes(document.schema_version)) return unchanged;
  const [heading, list] = document.blocks;
  const headingAttributes = heading?.attrs as Record<string, unknown> | undefined;
  if (heading?.type !== 'heading' || headingAttributes?.level !== 3 || inline(heading) !== 'Propiedades de Notion' || list?.type !== 'bullet_list' || !list.children?.length) return unchanged;
  const properties: ImportedProperty[] = [];
  for (const item of list.children) {
    if (item.type !== 'list_item' || item.children?.length !== 1 || item.children[0].type !== 'paragraph') return unchanged;
    const match = /^([^:\n]{1,80}):\s*([^\n]+)$/.exec(inline(item.children[0]));
    if (!match || properties.some(property => property.name === match[1])) return unchanged;
    properties.push({ name: match[1], value: match[2] });
  }
  if (!properties.some(property => ['Archived','Created','Favorite','Type','Updated','Tag','Tags','URL','Status','Priority','Due','Name'].includes(property.name))) return unchanged;
  const expected = new Set(properties.map(property => `${property.name}: ${property.value}`.trim()));
  const roleProperties = sourceCollection === 'Goals' ? new Set(['Status','Tag','Progress','Archived','Projects','Updated','Milestones','Goal Set','Target Deadline','Achieved','Created','This Quarter','This Year','Latest Activity'])
    : sourceCollection === 'Milestones' ? new Set(['Goal','Goal Area','Target Deadline','Date Completed','Created','Updated'])
    : sourceCollection === 'Notes' ? new Set(['Favorite','Archived','Type','Created','Updated','Root Tag','Project Tag','Project People','Updated (Short)','Project Archived','Tag','Tags','Project','Projects','People','URL','Image','Review Date','Note Date']) : null;
  let split = 2;
  while (split < Math.min(document.blocks.length, 12)) {
    const block = document.blocks[split];
    if (block.type !== 'paragraph') break;
    const lines = inline(block).split('\n').map(line => line.trim()).filter(Boolean);
    if (!lines.length) break;
    if (sourceCollection === 'Notes') {
      // Only the first plain export paragraph, with multiple exact anchors.
      // Conflicting values, repeated names or rich/ambiguous text stay visible.
      if (split !== 2 || block.content.length !== 1 || block.content[0].type !== 'text' || block.content[0].marks?.length || lines.filter(line => expected.has(line)).length < 3) break;
      const parsed = lines.map(line => /^([^:\n]{1,80}):\s*([^\n]*)$/.exec(line));
      if (parsed.some(match => !match || !roleProperties?.has(match[1]))) break;
      const names = parsed.map(match => match![1]);
      if (new Set(names).size !== names.length || parsed.some(match => properties.some(property => property.name === match![1] && property.value !== match![2]))) break;
    }
    if (!lines.every(line => expected.has(line))) {
      // Verified exports repeat relation properties in this first paragraph even
      // when the CSV header omits them. Require the canonical import marker,
      // an exact known line, and only fields from that verified collection.
      if (!roleProperties || split !== 2 || !lines.some(line => expected.has(line))) break;
      const extra = lines.map(line => /^([^:\n]{1,80}):\s*([^\n]*)$/.exec(line));
      if (extra.some(match => !match || !roleProperties.has(match[1]))) break;
      for (const match of extra) if (match && !properties.some(property => property.name === match[1])) properties.push({ name: match[1], value: match[2] });
    }
    split++;
  }
  return { properties, prefix: document.blocks.slice(0, split), body: { ...document, blocks: document.blocks.slice(split) } };
}
export function composeImportedBody(prefix: Block[], body: Document): Document {
  return { ...body, blocks: [...JSON.parse(JSON.stringify(prefix)) as Block[], ...body.blocks] };
}
export function propertyStyle(name: string): 'checkbox' | 'chip' | 'text' {
  return ['Favorite','Archived'].includes(name) ? 'checkbox' : ['Type','Status','Priority'].includes(name) ? 'chip' : 'text';
}
