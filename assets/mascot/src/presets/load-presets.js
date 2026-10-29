import { readFileSync } from 'node:fs';
// Node-side loader for presets.json (browser viewer reads clips from the .glb instead).
export const loadPresets = () => JSON.parse(readFileSync(new URL('./presets.json', import.meta.url), 'utf8'));
export const loadJson = (f) => JSON.parse(readFileSync(new URL(`./${f}`, import.meta.url), 'utf8'));
