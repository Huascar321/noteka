import { mount } from 'svelte';
import App from './App.svelte';
import './style.css';

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

mount(App, { target: document.getElementById('app')! });
