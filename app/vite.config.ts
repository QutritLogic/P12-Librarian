import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Tauri expects a fixed port and must not have Vite clear its output.
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  // The dev server may also read the repository root, for DISCLAIMER.md.
  server: { port: 5173, strictPort: true, fs: { allow: ['..'] } },
});
