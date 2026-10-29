import './node-polyfills.js';
import { writeFileSync } from 'node:fs';
import * as THREE from 'three';
import { GLTFExporter } from 'three/addons/exporters/GLTFExporter.js';
import { buildMascot } from '../src/scene/build-mascot.js';
import { loadPresets, loadJson } from '../src/presets/load-presets.js';
import { EYE_MORPHS } from '../src/geometry/eye.js';
const scene = new THREE.Scene(); const m = buildMascot(loadPresets(), loadJson('parts.json'), loadJson('layers.json')); scene.add(m); scene.updateMatrixWorld(true);
const bodyMesh = m.getObjectByName('body'), colorMorph = bodyMesh.userData.colorMorph; delete bodyMesh.userData.colorMorph; // keep it out of JSON extras
const glb = await new GLTFExporter().parseAsync(scene, { binary: true, animations: m.animations });
// Post-process: turn the body's (all-zero) POSITION morph target into a COLOR_0 morph target carrying the colour
// reveal delta. glTF 2.0 allows COLOR_n in morph targets; three.js GLTFLoader reads it (morphAttributes.color).
const out = addColorMorph(Buffer.from(glb), colorMorph);
writeFileSync(new URL(process.env.OUT ?? '../out/kinekino-mascot.glb', import.meta.url), out);
console.log('glb bytes', out.length, 'clips', m.animations.map((c) => c.name).join(','));

function addColorMorph(buf, delta) {
  const jsonLen = buf.readUInt32LE(12), json = JSON.parse(buf.subarray(20, 20 + jsonLen).toString('utf8'));
  const binStart = 20 + jsonLen + 8, bin0 = buf.subarray(binStart, binStart + buf.readUInt32LE(20 + jsonLen));
  const node = json.nodes.find((n) => n.name === 'body'), prim = json.meshes[node.mesh].primitives[0];
  // Keep the zero POSITION delta (three.js r170 only enables morphing in the shader when a POSITION/NORMAL morph
  // exists) and append the COLOR_0 delta as a new accessor.
  // v7: one COLOR_0 delta per target (0 reveal, 1 blush).
  const n = json.accessors[prim.targets[0].POSITION].count, parts = [bin0];
  let off = bin0.length;
  // v7 size: the zero POSITION deltas become one shared accessor without bufferView (glTF: zero-initialised),
  // and each COLOR_0 delta is stored sparse (only vertices with a non-zero delta: blush touches ~2 spots).
  const push = (b) => { const pad = Buffer.alloc((4 - (off % 4)) % 4); off += pad.length; json.bufferViews.push({ buffer: 0, byteOffset: off, byteLength: b.length }); off += b.length; parts.push(pad, b); return json.bufferViews.length - 1; };
  json.accessors.push({ componentType: 5126, count: n, type: 'VEC3', min: [0, 0, 0], max: [0, 0, 0] });
  const zeroPos = json.accessors.length - 1;
  delta.forEach((d, k) => {
    if (n * 3 !== d.length) throw new Error('colour morph size mismatch');
    if (prim.targets[k].NORMAL !== undefined) delete prim.targets[k].NORMAL;
    prim.targets[k].POSITION = zeroPos;
    const idx = []; for (let i = 0; i < n; i++) if (d[i * 3] || d[i * 3 + 1] || d[i * 3 + 2]) idx.push(i);
    const iv = push(Buffer.from(new Uint32Array(idx).buffer)), vals = new Float32Array(idx.length * 3);
    idx.forEach((i, j) => vals.set(d.subarray(i * 3, i * 3 + 3), j * 3));
    const vv = push(Buffer.from(vals.buffer));
    json.accessors.push({ componentType: 5126, count: n, type: 'VEC3', sparse: { count: idx.length, indices: { bufferView: iv, componentType: 5125 }, values: { bufferView: vv } } });
    prim.targets[k].COLOR_0 = json.accessors.length - 1;
  });
  compact(json, Buffer.concat(parts), (b) => { parts.length = 0; parts.push(b); });
  const bin = parts[0]; json.buffers[0].byteLength = bin.length;
  json.meshes[node.mesh].extras = { targetNames: ['reveal', 'blush'] };
  for (const e of ['eye_L', 'eye_R']) { const m = json.meshes[json.nodes.find((x) => x.name === e).mesh]; m.extras = { ...m.extras, targetNames: EYE_MORPHS }; }
  let j = Buffer.from(JSON.stringify(json), 'utf8'); j = Buffer.concat([j, Buffer.alloc((4 - (j.length % 4)) % 4, 0x20)]);
  const head = Buffer.alloc(20); head.writeUInt32LE(0x46546c67, 0); head.writeUInt32LE(2, 4);
  head.writeUInt32LE(20 + j.length + 8 + bin.length, 8); head.writeUInt32LE(j.length, 12); head.writeUInt32LE(0x4e4f534a, 16);
  const bh = Buffer.alloc(8); bh.writeUInt32LE(bin.length, 0); bh.writeUInt32LE(0x004e4942, 4);
  return Buffer.concat([head, j, bh, bin]);
}

// Drop bufferViews no accessor/image references any more (e.g. the replaced zero POSITION deltas) and repack the BIN.
function compact(json, bin, set) {
  const refs = new Set();
  for (const a of json.accessors) { if (a.bufferView !== undefined) refs.add(a.bufferView); if (a.sparse) { refs.add(a.sparse.indices.bufferView); refs.add(a.sparse.values.bufferView); } }
  for (const im of json.images ?? []) if (im.bufferView !== undefined) refs.add(im.bufferView);
  const map = new Map(), views = [], chunks = []; let off = 0;
  json.bufferViews.forEach((v, i) => { if (!refs.has(i)) return; const pad = (4 - (off % 4)) % 4; chunks.push(Buffer.alloc(pad)); off += pad;
    chunks.push(bin.subarray(v.byteOffset ?? 0, (v.byteOffset ?? 0) + v.byteLength)); map.set(i, views.length); views.push({ ...v, byteOffset: off }); off += v.byteLength; });
  json.bufferViews = views;
  for (const a of json.accessors) { if (a.bufferView !== undefined) a.bufferView = map.get(a.bufferView); if (a.sparse) { a.sparse.indices.bufferView = map.get(a.sparse.indices.bufferView); a.sparse.values.bufferView = map.get(a.sparse.values.bufferView); } }
  for (const im of json.images ?? []) if (im.bufferView !== undefined) im.bufferView = map.get(im.bufferView);
  set(Buffer.concat(chunks));
}
