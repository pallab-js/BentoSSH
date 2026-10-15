import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { fileURLToPath } from 'node:url';

export default defineConfig({
  plugins: [svelte()],
  test: {
    environment: 'jsdom',
    include: ['src/**/*.{test,spec}.{js,ts}'],
    globals: true
  },
  resolve: {
    // Use the browser runtime so runes-based `.svelte.ts` modules execute.
    conditions: ['browser'],
    alias: {
      // $app/environment is a SvelteKit virtual module; stub it for unit tests.
      '$app/environment': fileURLToPath(new URL('./src/test/app-environment-stub.ts', import.meta.url)),
      $lib: fileURLToPath(new URL('./src/lib', import.meta.url))
    }
  }
});
