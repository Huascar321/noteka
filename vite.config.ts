import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
export default defineConfig({
  plugins: [svelte()],
  // Self-contained ESM stays lazy and avoids stale optimizer URLs after a new
  // icon package is added while the Tauri development session is already open.
  optimizeDeps: { exclude: ['@animateicons/vanilla'] },
  server: {
    port: 1420, strictPort: true, host: '127.0.0.1',
    watch: { ignored: ['**/target/**', '**/src-tauri/**', '**/crates/**'] },
  },
  clearScreen: false,
});
