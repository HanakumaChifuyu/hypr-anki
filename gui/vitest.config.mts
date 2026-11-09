import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    includeSource: ['src/renderer/**/*.{ts,tsx,js,jsx}'],
  },
});
