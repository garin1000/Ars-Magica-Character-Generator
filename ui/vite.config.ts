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
  build: {
    // Vite's 500 kB default is a web download budget; this bundle is read from
    // the local disk inside the installed app, so it does not apply. The bundle
    // is ~580 kB, a third of it the two raw `.ftl` locales (`lib/i18n.ts`). The
    // limit stays finite so an unexpected doubling still warns.
    chunkSizeWarningLimit: 1024,
  },
});
