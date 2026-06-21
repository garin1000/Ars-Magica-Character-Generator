import { defineConfig } from 'vite';
import { svelte, vitePreprocess } from '@sveltejs/vite-plugin-svelte';

// Tauri expects a fixed port and no terminal screen-clearing so its CLI output
// stays visible. 1420 matches `tauri.conf.json`'s `build.devUrl`.
export default defineConfig({
  plugins: [svelte({ preprocess: vitePreprocess() })],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    // Allow importing the Fluent `.ftl` files from the repo-level `locales/`
    // directory (one level above `ui/`).
    fs: { allow: ['..'] },
  },
});
