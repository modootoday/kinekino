import * as THREE from 'three';
import { H } from '../animation/tracks.js';
import { UNIT } from '../geometry/svg-space.js';
import { EYE_MORPHS } from '../geometry/eye.js';
// Morph target names per node (weights vectors are padded to this length; 'w:<name>' writes one entry).
export const MORPHS = { eye_L: EYE_MORPHS, eye_R: EYE_MORPHS, body: ['reveal', 'blush'] };
// Turns presets.json into THREE.AnimationClips (keys are resampled so easing survives glTF LINEAR export).
const EASE = { linear: (t) => t, easeOut: (t) => 1 - (1 - t) ** 3, easeIn: (t) => t ** 3,
  easeInOut: (t) => (t < 0.5 ? 4 * t ** 3 : 1 - (-2 * t + 2) ** 3 / 2) };
const lerp = (a, b, f) => (Array.isArray(a) ? a.map((v, i) => v + (b[i] - v) * f) : a + (b - a) * f);
export function sampleKeys(keys, easing, rate) {
  const out = []; const e = EASE[easing ?? 'linear'];
  for (let k = 0; k < keys.length - 1; k++) {
    const [t0, v0] = keys[k], [t1, v1] = keys[k + 1], n = Math.max(1, Math.round((t1 - t0) * rate));
    for (let s = 0; s < n; s++) out.push([t0 + ((t1 - t0) * s) / n, lerp(v0, v1, e(s / n))]);
  }
  out.push(keys[keys.length - 1]); return out;
}
export function sampleOsc(o, path, duration, rate) {
  const out = [[0, null]];
  for (let t = o.start; t <= o.end + 1e-6; t += 1 / rate) {
    const local = t - o.start, fade = Math.min(1, local / 0.08, (o.end - t) / 0.08 + 0.0001);
    const v = o.amp * Math.exp(-(o.decay ?? 0) * local) * Math.sin(2 * Math.PI * o.freq * local + (o.phase ?? 0)) * Math.max(0, fade);
    out.push([t, v]);
  }
  out.push([duration, null]);
  // scale3: 1 + axis*v; trans: axis*v (SVG units, default x); scalar paths: v.
  return out.map(([t, v]) => [t, path === 'scale3' ? (v === null ? [1, 1, 1] : (o.axis ?? [1, 1, 1]).map((a) => 1 + a * v))
    : path === 'trans' ? (o.axis ?? [1, 0, 0]).map((a) => a * (v ?? 0)) : v ?? 0]);
}
export const Q = new THREE.Quaternion(), AX = { rotZ: new THREE.Vector3(0, 0, 1), rotX: new THREE.Vector3(1, 0, 0), rotY: new THREE.Vector3(0, 1, 0) };
// Weight vector for a node: 'weights' = array (padded with 0), 'reveal' = body[0], 'w:<morph>' = one named entry.
function weightVec(target, path, v) {
  const names = MORPHS[target] ?? [], out = names.map(() => 0);
  if (path === 'weights') { v.forEach((x, i) => { out[i] = x; }); return out; }
  const name = path === 'reveal' ? 'reveal' : path.slice(2), i = names.indexOf(name);
  if (i < 0) throw new Error(`no morph ${name} on ${target}`); out[i] = v; return out;
}
export function toTrack(target, path, samples, rest) {
  const times = samples.map(([t]) => t);
  if (path === 'trans') { const r = rest?.(target) ?? [0, 0, 0];
    return new THREE.VectorKeyframeTrack(`${target}.position`, times, samples.flatMap(([, v]) => v.map((c, i) => r[i] + c * UNIT))); }
  if (path === 'weights' || path === 'reveal' || path.startsWith('w:'))
    return new THREE.NumberKeyframeTrack(`${target}.morphTargetInfluences`, times, samples.flatMap(([, v]) => weightVec(target, path, v)));
  if (path === 'scale3') return new THREE.VectorKeyframeTrack(`${target}.scale`, times,
    samples.flatMap(([, v]) => (v.every((c) => c === 0) ? [H, H, H] : v)));
  return new THREE.QuaternionKeyframeTrack(`${target}.quaternion`, times,
    samples.flatMap(([, d]) => Q.setFromAxisAngle(AX[path], THREE.MathUtils.degToRad(d)).toArray()));
}
function decorEnvelope(d, env) {
  const t = [0, env.in, d - env.out, d], v = [H, 1, 1, H];
  return [new THREE.NumberKeyframeTrack('body.morphTargetInfluences', t, v.flatMap((x) => [x <= H ? 0 : 1, 0]))]; // [reveal, blush]
}
export function buildPresetClips(json) {
  return Object.entries(json.presets).map(([name, p]) => {
    const tracks = p.channels.map((c) => toTrack(c.target, c.path,
      c.osc ? sampleOsc(c.osc, c.path, p.duration, json.sampleRate) : sampleKeys(c.keys, c.easing, json.sampleRate)));
    return new THREE.AnimationClip(name, p.duration, [...decorEnvelope(p.duration, json.envelope), ...mergeRotations(tracks)]);
  });
}
// Several channels may hit one node property (rotZ+rotX+rotY, trans x2, scale3 x2, several w:<morph>): resample on the
// union timeline and combine - quaternions multiply in channel order, scale multiplies, position adds deltas from rest
// (rest(name) gives the rest position), morph weights add. Same rule in every runtime (Rust: see .spec/guide/mascot-part-layers.guide.md).
export function mergeRotations(tracks, rest) {
  const groups = new Map(); for (const t of tracks) groups.set(t.name, [...(groups.get(t.name) ?? []), t]);
  return [...groups.values()].map((g) => {
    if (g.length === 1) return g[0];
    const times = [...new Set(g.flatMap((t) => [...t.times]))].sort((a, b) => a - b);
    const prop = g[0].name.split('.').pop(), node = g[0].name.slice(0, -prop.length - 1), its = g.map((t) => t.createInterpolant());
    const val = (i, time) => [...its[i].evaluate(time)];
    if (prop === 'quaternion') return new THREE.QuaternionKeyframeTrack(g[0].name, times, times.flatMap((time) => {
      const q = new THREE.Quaternion(); g.forEach((_, i) => q.multiply(new THREE.Quaternion().fromArray(val(i, time)))); return q.toArray(); }));
    const r = prop === 'position' ? rest?.(node) ?? [0, 0, 0] : null;
    const combine = (time) => g.reduce((acc, _, i) => { const v = val(i, time);
      if (!acc) return v;
      return acc.map((a, k) => (prop === 'scale' ? a * v[k] : prop === 'position' ? a + v[k] - r[k] : a + v[k])); }, null);
    const Ctor = prop === 'morphTargetInfluences' ? THREE.NumberKeyframeTrack : THREE.VectorKeyframeTrack;
    return new Ctor(g[0].name, times, times.flatMap(combine));
  });
}
