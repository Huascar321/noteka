import { Extension } from '@tiptap/core';
import { Plugin, PluginKey } from '@tiptap/pm/state';
import { Decoration, DecorationSet } from '@tiptap/pm/view';

export const sourceKey = new PluginKey<string | null>('noteka-source');
export const SourceLocator = Extension.create({
  name: 'notekaSource',
  addProseMirrorPlugins() {
    return [new Plugin({
      key: sourceKey,
      state: { init: () => null, apply: (tr, value) => tr.getMeta(sourceKey) ?? value },
      props: { decorations(state) {
        const id = sourceKey.getState(state);
        const decorations: Decoration[] = [];
        if (id) state.doc.descendants((node, pos) => {
          if (node.attrs.id === id) decorations.push(Decoration.node(pos, pos + node.nodeSize, { class: 'source-highlight' }));
        });
        return DecorationSet.create(state.doc, decorations);
      } },
    })];
  },
});
