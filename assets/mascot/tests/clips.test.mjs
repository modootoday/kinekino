// Part-clip invariants: rest at start/end, layer ownership, wrap_body/groom penetration, body fold check.
import test from 'node:test';
import assert from 'node:assert/strict';
import * as THREE from 'three';
import { parts, layers, mascot, clipsByName, poseAt, skinned } from './_mascot.mjs';
import { resolveClip } from '../src/presets/build-part-clips.js';

const partNames = Object.keys(parts.clips);
const rest = (track) => { // value of a track's property at rest
  const [node, prop] = track.name.split('.'), o = mascot.getObjectByName(node), r = o.userData.__rest;
  if (prop === 'morphTargetInfluences') return new Array(track.getValueSize()).fill(0);
  return { position: r.p, quaternion: r.q, scale: r.s }[prop].toArray();
};
const close = (a, b, eps) => a.every((x, i) => Math.abs(x - b[i]) < eps) || (a.length === 4 && a.every((x, i) => Math.abs(x + b[i]) < eps));

test('every part clip compiled', () => { for (const n of partNames) assert.ok(clipsByName.has(n), n); });

test('every clip starts and ends at rest (restExempt: entry/exit/transition)', () => {
  const bad = [];
  for (const n of partNames) {
    const c = resolveClip(parts, n), clip = clipsByName.get(n), ex = c.restExempt;
    for (const t of clip.tracks) {
      const k = t.getValueSize(), r = rest(t), first = Array.from(t.values.slice(0, k)), last = Array.from(t.values.slice(-k));
      const eps = t.name.endsWith('position') ? 1e-3 : 1e-3;
      if (ex === 'revealed') { if (!close(first, last, eps)) bad.push(`${n} ${t.name} start!=end`); continue; }
      if (ex !== 'entry' && ex !== 'transition' && !close(first, r, eps)) bad.push(`${n} ${t.name} start`);
      if (ex !== 'exit' && ex !== 'transition' && !close(last, r, eps)) bad.push(`${n} ${t.name} end`);
    }
  }
  assert.deepEqual(bad, []);
});

// layers.json channel strings -> "node.property" set
const PROP = { rotation: 'quaternion', translation: 'position', scale: 'scale', weights: 'morphTargetInfluences' };
function owned(layer) {
  const s = new Set();
  for (const ch of layers.layers[layer].channels) {
    const [spec] = ch.split(' '), dot = spec.lastIndexOf('.'), node = spec.slice(0, dot), prop = PROP[spec.slice(dot + 1)] ?? spec.slice(dot + 1);
    const m = node.match(/^(.*)_(\d)\.\.(\d)$/);
    if (m) for (let i = +m[2]; i <= +m[3]; i++) s.add(`${m[1]}_${i}.${prop}`); else s.add(`${node}.${prop}`);
  }
  return s;
}
test('every clip writes only its own layer channels', () => {
  const bad = [];
  for (const n of partNames) {
    const c = resolveClip(parts, n), own = owned(c.layer);
    for (const t of clipsByName.get(n).tracks) if (!own.has(t.name)) bad.push(`${n} (${c.layer}) -> ${t.name}`);
  }
  assert.deepEqual(bad, []);
});
test('no channel owned by two layers', () => {
  const seen = new Map(), dup = [];
  for (const l of Object.keys(layers.layers)) for (const ch of owned(l)) { if (seen.has(ch)) dup.push(`${ch}: ${seen.get(ch)} & ${l}`); seen.set(ch, l); }
  assert.deepEqual(dup, []);
});

// Penetration: tail-dominant vertices vs. the body slab. For each XY grid cell the body (root/deformer-dominant
// vertices) spans [zmin, zmax]; a tail vertex strictly inside that span (minus a skin margin) is inside the body.
const mesh = mascot.getObjectByName('body'), bones = mesh.skeleton.bones.map((b) => b.name);
const si = mesh.geometry.attributes.skinIndex, sw = mesh.geometry.attributes.skinWeight, N = si.count;
const dom = (i, pred) => { let w = 0; for (let k = 0; k < 4; k++) if (pred(bones[si.getComponent(i, k)])) w += sw.getComponent(i, k); return w > 0.5; };
const isTail = Array.from({ length: N }, (_, i) => dom(i, (b) => b.startsWith('tail')));
const isBody = Array.from({ length: N }, (_, i) => dom(i, (b) => b === 'root' || b === 'kinekino' || b === 'crown' || b.startsWith('base_')));
export function penetration(P, margin) {
  const cell = 0.02, grid = new Map(), key = (x, y) => `${Math.floor(x / cell)},${Math.floor(y / cell)}`;
  for (let i = 0; i < N; i++) if (isBody[i]) { const k = key(P[i * 3], P[i * 3 + 1]), g = grid.get(k) ?? [Infinity, -Infinity]; g[0] = Math.min(g[0], P[i * 3 + 2]); g[1] = Math.max(g[1], P[i * 3 + 2]); grid.set(k, g); }
  let n = 0, depth = 0;
  for (let i = 0; i < N; i++) if (isTail[i]) {
    const g = grid.get(key(P[i * 3], P[i * 3 + 1])); if (!g || g[1] - g[0] < 2 * margin) continue;
    const z = P[i * 3 + 2]; if (z > g[0] + margin && z < g[1] - margin) { n++; depth = Math.max(depth, Math.min(z - g[0], g[1] - z)); }
  }
  return { n, depth };
}
const span = (() => { let a = Infinity, b = -Infinity; const p = mesh.geometry.attributes.position; for (let i = 0; i < N; i++) { a = Math.min(a, p.getZ(i)); b = Math.max(b, p.getZ(i)); } return b - a; })();
const MARGIN = span * 0.15;
test('rest pose: tail does not penetrate body (baseline)', () => { poseAt(null, 0); const r = penetration(skinned(), MARGIN); assert.equal(r.n, 0, JSON.stringify(r)); });
for (const [clip, ts] of [['tail.wrap_body', [0.4, 0.8, 1.2]], ['body.groom', [0.6, 1.2, 1.8, 2.4]], ['tail.tuck', [0.5]], ['tail.heart_curve', [0.6]]]) {
  test(`${clip}: tail does not penetrate body`, () => {
    for (const t of ts) { poseAt(clip, t); const r = penetration(skinned(), MARGIN); assert.equal(r.n, 0, `${clip}@${t}: ${JSON.stringify(r)}`); }
  });
}
// Fold check: no body triangle flips (3D face normal turns > 90 deg vs. rest) under shape/body clips.
test('shape/body deformers do not fold triangles', () => {
  const idx = mesh.geometry.index.array; poseAt(null, 0); const R = skinned();
  const nrm = (P, a, b, c) => { const u = [0, 1, 2].map((k) => P[b * 3 + k] - P[a * 3 + k]), v = [0, 1, 2].map((k) => P[c * 3 + k] - P[a * 3 + k]);
    return [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]]; };
  const bad = [];
  for (const clip of ['body.knead', 'body.crouch', 'body.arch', 'body.loaf', 'body.wobble_hit', 'body.squash_press', 'body.melt']) {
    const c = clipsByName.get(clip); poseAt(clip, (c.userData.holdAt ?? c.duration * 0.5)); const P = skinned(); let n = 0;
    for (let i = 0; i < idx.length; i += 3) { const a = nrm(R, idx[i], idx[i + 1], idx[i + 2]), b = nrm(P, idx[i], idx[i + 1], idx[i + 2]), la = Math.hypot(...a), lb = Math.hypot(...b);
      if (la < 1e-9) continue; if (lb < 1e-9 || (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / (la * lb) < 0) n++; }
    if (n) bad.push(`${clip}: ${n}`);
  }
  assert.deepEqual(bad, []);
});
// v8 strict tail check (tools/tail-penetration.mjs): no tail-dominant vertex inside the body volume (logo polygon x
// pillow height field) at 19 samples of every tail clip and body.groom.
import { metric } from '../tools/tail-penetration.mjs';
test('v8 strict: no tail clip pushes the tail into the body', () => {
  const bad = [];
  for (const n of [...clipsByName.keys()].filter((c) => c.startsWith('tail.') || c === 'body.groom')) {
    const d = clipsByName.get(n).duration; for (let k = 1; k < 20; k++) { poseAt(n, (d * k) / 20); const m = metric(skinned()); if (m.overlap > 0) { bad.push(`${n}@${((d * k) / 20).toFixed(2)} ${(m.overlap * 100).toFixed(1)}%`); break; } }
  }
  assert.deepEqual(bad, []);
});
