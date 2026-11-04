import { readFile } from 'node:fs/promises';

import initialize from './ironpages.js';

export * from './ironpages.js';

let initialization;

export default function init() {
  initialization ??= initializeNode();
  return initialization;
}

async function initializeNode() {
  try {
    const wasm = await readFile(new URL('./ironpages_bg.wasm', import.meta.url));
    return await initialize({ module_or_path: wasm });
  } catch (cause) {
    throw new Error(
      'Failed to initialize Ironpages WebAssembly in Node.js. ' +
        'The packaged ironpages_bg.wasm asset could not be loaded or instantiated.',
      { cause },
    );
  }
}
