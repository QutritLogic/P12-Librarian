import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import license from 'rollup-plugin-license';
import { fileURLToPath } from 'node:url';

// Licence notices of the npm packages bundled into the app, written on every
// production build. A package under any other licence fails the build.
const javascriptLicenses = license({
  thirdParty: {
    includePrivate: false,
    allow: {
      test: '(MIT OR Apache-2.0 OR ISC OR BSD-2-Clause OR BSD-3-Clause OR 0BSD OR OFL-1.1)',
      failOnUnlicensed: true,
      failOnViolation: true,
    },
    output: {
      file: fileURLToPath(new URL('../licenses/javascript.md', import.meta.url)),
      template: (dependencies) =>
        [
          '## JavaScript libraries',
          '',
          ...dependencies
            .slice()
            .sort((a, b) => (a.name ?? '').localeCompare(b.name ?? ''))
            .flatMap((dep) => [
              `### ${dep.name} ${dep.version} (${dep.license})`,
              '',
              '```text',
              (dep.licenseText ?? `${dep.license} licence; no licence file is included in the package.`).trim(),
              '```',
              '',
            ]),
        ].join('\n'),
    },
  },
});

// Tauri expects a fixed port and must not have Vite clear its output.
export default defineConfig({
  plugins: [svelte(), { ...javascriptLicenses, apply: 'build' }],
  clearScreen: false,
  // The dev server may also read the repository root, for DISCLAIMER.md.
  server: { port: 5173, strictPort: true, fs: { allow: ['..'] } },
});
