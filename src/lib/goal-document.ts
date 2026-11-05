import type { Block, Document } from './contracts.generated';

const OVERVIEW = 'noteka-goal-overview-v1-';
const DETAILS = 'noteka-goal-details-v1-';
const JOURNAL = 'noteka-goal-journal-v1-';
type Parts = { legacy: Block[]; overview: Block[]; details: Block[]; journal: Block[]; marker: string | null };

function marker(block: Block | undefined, prefix: string, title: string): string | null {
  if (!block || !block.id.startsWith(prefix) || block.type !== 'heading' ||
      (block.attrs as Record<string, unknown> | undefined)?.level !== 2 ||
      block.children?.length || block.content.length !== 1 ||
      block.content[0].type !== 'text' || block.content[0].text !== title || block.content[0].marks?.length) return null;
  const suffix = block.id.slice(prefix.length);
  return /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(suffix) ? suffix : null;
}

// Only a complete, identifiable pair is interpreted. Every other block stays
// editable as original content, including incomplete or older markers.
export function splitGoalDocument(document: Document): Parts {
  const blocks = document.blocks;
  for (let j = blocks.length - 1; j >= 0; j--) {
    const id = marker(blocks[j], JOURNAL, 'Journal');
    if (!id) continue;
    for (let i = j - 1; i >= 0; i--) {
      if (marker(blocks[i], OVERVIEW, 'Goal Overview') !== id) continue;
      let detail = -1;
      for (let k = i + 1; k < j; k++) if (marker(blocks[k], DETAILS, 'Additional Details') === id) { detail = k; break; }
      return { legacy: blocks.slice(0, i), overview: blocks.slice(i + 1, detail < 0 ? j : detail), details: detail < 0 ? [] : blocks.slice(detail + 1, j), journal: blocks.slice(j + 1), marker: id };
    }
  }
  return { legacy: blocks.slice(), overview: [], details: [], journal: [], marker: null };
}

export function composeGoalDocument(document: Document, parts: Parts, markerId: string): Document {
  const heading = (id: string, title: string): Block => ({ id, type: 'heading', attrs: { level: 2 }, content: [{ type: 'text', text: title }] });
  return { ...document, blocks: [...parts.legacy, heading(OVERVIEW + markerId, 'Goal Overview'), ...parts.overview, heading(DETAILS + markerId, 'Additional Details'), ...parts.details, heading(JOURNAL + markerId, 'Journal'), ...parts.journal] };
}

export function goalHasContent(blocks: Block[]): boolean {
  return blocks.some(block => block.type !== 'paragraph' || block.content.some(item => item.type !== 'text' || item.text.length > 0) || !!block.children?.length);
}
