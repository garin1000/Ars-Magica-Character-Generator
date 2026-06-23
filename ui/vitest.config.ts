import { defineConfig } from 'vitest/config';

// Unit tests for the pure helpers in `src/lib` (derive.ts etc.). These exercise
// in-memory logic only — no DOM, no Tauri IPC — so a plain Node environment is
// enough. E2E lives separately under `e2e/` and is excluded here.
export default defineConfig({
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts'],
    exclude: ['node_modules/', 'dist/', 'e2e/'],
  },
});
