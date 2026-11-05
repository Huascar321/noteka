import { mount } from 'svelte';
import App from './App.svelte';
import './style.css';
import { operate } from './lib/api';
import type { WorkspaceInfo } from './lib/contracts.generated';
import { openStartupWorkspace, readWorkspacePath, saveWorkspacePath, clearWorkspacePath } from './lib/workspace-startup';

// External file and URL drops must not navigate the workspace or import content.
// Internal project and block drags use their own data types and handlers.
function preventExternalDrop(event: DragEvent) {
  const types = Array.from(event.dataTransfer?.types ?? []);
  if (!types.includes('Files') && !types.includes('text/uri-list')) return;
  event.preventDefault();
  if (event.type === 'dragover' && event.dataTransfer) event.dataTransfer.dropEffect = 'none';
}
window.addEventListener('dragover', preventExternalDrop, true);
window.addEventListener('drop', preventExternalDrop, true);

// El workspace se resuelve antes de montar: un arranque normal no muestra la
// pantalla de bienvenida ni la hace parpadear. Sin workspace disponible, la
// interfaz cae en la selección explícita de siempre.
const startup = await openStartupWorkspace(
  readWorkspacePath(() => localStorage),
  path => operate<WorkspaceInfo>({ operation: 'workspace.open', input: {} }, path || undefined),
).catch(() => ({ workspace: null, error: '' }));
if (startup.workspace) saveWorkspacePath(startup.workspace.path, () => localStorage);
else if (!startup.error) clearWorkspacePath(() => localStorage);

mount(App, { target: document.getElementById('app')!, props: { startupWorkspace: startup.workspace, startupError: startup.error } });
