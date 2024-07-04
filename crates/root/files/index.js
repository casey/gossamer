import main from './loader.js';

await main(new URL('index.wasm', import.meta.url));
