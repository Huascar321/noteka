import { Extension } from '@tiptap/core';
import { Plugin } from '@tiptap/pm/state';
import type { Node as ProseMirrorNode } from '@tiptap/pm/model';
import type { NodeView } from '@tiptap/pm/view';
import { markdownTable } from './markdown-table';

// A lossless view over an existing Markdown code block, not a persisted table type.
export const DocumentTableView = Extension.create({
  name: 'documentTableView',
  addProseMirrorPlugins() {
    return [new Plugin({ props: { nodeViews: { codeBlock(initial, view, getPos): NodeView {
      let rows = markdownTable(initial.textContent, initial.attrs.language);
      const tabular = rows !== null;
      const dom = document.createElement(tabular ? 'div' : 'pre');
      const contentDOM = document.createElement('code');
      const table = document.createElement('table');
      let sourceDetails: HTMLDetailsElement | undefined;
      let sourceInput: HTMLTextAreaElement | undefined;
      function render(values: string[][]) {
        table.replaceChildren(); const body = document.createElement('tbody');
        for (const row of values) { const tr = document.createElement('tr'); for (const cell of row) { const td = document.createElement('td'); td.textContent = cell; tr.append(td); } body.append(tr); }
        table.append(body);
      }
      if (rows) {
        dom.className = 'document-table'; table.contentEditable = 'false'; table.setAttribute('aria-label',"Document table"); render(rows); dom.append(table);
        const details = document.createElement('details'); details.className = 'document-table-source'; details.contentEditable = 'false'; sourceDetails = details;
        const summary = document.createElement('summary'); summary.textContent = "Edit table as Markdown";
        sourceInput = document.createElement('textarea'); sourceInput.setAttribute('aria-label',"Table Markdown"); sourceInput.value = initial.textContent; sourceInput.spellcheck = false;
        sourceInput.addEventListener('input', () => {
          const position = getPos();
          if (!sourceInput || typeof position !== 'number') return;
          const current = view.state.doc.nodeAt(position);
          if (!current || !view.editable) { sourceInput.value = current?.textContent ?? ''; return; }
          view.dispatch(view.state.tr.insertText(sourceInput.value, position + 1, position + 1 + current.content.size));
        });
        details.append(summary, sourceInput); dom.append(details);
        // ProseMirror owns this mirror for mapping/serialization. Raw edits above
        // dispatch a real transaction; they never mutate a detached DOM buffer.
        const mirror = document.createElement('pre'); mirror.hidden = true; mirror.setAttribute('aria-hidden','true'); mirror.append(contentDOM); dom.append(mirror);
      } else dom.append(contentDOM);
      return { dom, contentDOM,
        update(next: ProseMirrorNode) {
          if (next.type.name !== 'codeBlock') return false;
          if (!tabular) return true;
          const values = markdownTable(next.textContent, next.attrs.language);
          table.hidden = values === null;
          if (!values && sourceDetails) sourceDetails.open = true;
          if (values && JSON.stringify(values) !== JSON.stringify(rows)) render(values);
          if (sourceInput && sourceInput.value !== next.textContent) sourceInput.value = next.textContent;
          rows = values; return true;
        },
        stopEvent(event) { return event.target instanceof Element && !!event.target.closest('summary,table,textarea'); },
        setSelection(anchor, head) {
          if (sourceDetails && sourceInput) { sourceDetails.open = true; sourceInput.focus(); sourceInput.setSelectionRange(anchor, head); return; }
          const target = contentDOM.firstChild ?? contentDOM;
          const limit = target.nodeType === 3 ? (target.textContent?.length ?? 0) : target.childNodes.length;
          dom.ownerDocument.getSelection()?.setBaseAndExtent(target, Math.min(anchor, limit), target, Math.min(head, limit));
        },
        ignoreMutation(mutation) { return mutation.type !== 'selection' && mutation.target !== contentDOM && !contentDOM.contains(mutation.target); },
      };
    } } } })];
  },
});
