import * as THREE from 'three';
import { sampleKeys, sampleOsc, toTrack, mergeRotations } from './build-preset-clips.js';
// parts.json -> one THREE.AnimationClip per part clip (name 'layer.action', e.g. 'ear_L.perk').
// rest(name) returns a node's rest position (trans channels are deltas in SVG units).
// v7: a clip may be {"mirror": "<left clip>"}: right side = left definition with targets _L -> _R, rotZ/rotY and
// trans x negated (rotX, scale, weights unchanged; eye glare/worried morphs are already side-aware in the geometry).
export function resolveClip(json, name) {
  const c = json.clips[name]; if (!c.mirror) return c;
  const src = json.clips[c.mirror], sw = (s) => s.replace(/_L(\b|_|$)/g, '_R$1');
  const flip = (path, v) => (path === 'rotZ' || path === 'rotY' ? (Array.isArray(v) ? v.map((x) => -x) : -v)
    : path === 'trans' ? [-v[0], v[1], v[2] ?? 0] : v);
  return { ...src, ...Object.fromEntries(Object.entries(c).filter(([k]) => k !== 'mirror')), layer: sw(src.layer), mirrorOf: c.mirror,
    channels: src.channels.map((ch) => ({ ...ch, target: sw(ch.target),
      ...(ch.keys ? { keys: ch.keys.map(([t, v]) => [t, flip(ch.path, v)]) } : {}),
      ...(ch.osc ? { osc: ch.path === 'rotZ' || ch.path === 'rotY' ? { ...ch.osc, amp: -ch.osc.amp }
        : ch.path === 'trans' ? { ...ch.osc, axis: (ch.osc.axis ?? [1, 0, 0]).map((a, i) => (i === 0 ? -a : a)) } : ch.osc } : {}) })) };
}
export function sampleChannel(ch, c, rate) {
  const samples = ch.osc ? sampleOsc(ch.osc, ch.path, c.duration, rate) : sampleKeys(ch.keys, ch.easing, rate);
  return samples.filter((s, i) => i === samples.length - 1 || samples[i + 1][0] - s[0] > 1e-6); // osc spanning 0..duration repeats end times
}
export function buildPartClips(json, rest) {
  return Object.keys(json.clips).map((name) => {
    const c = resolveClip(json, name);
    const tracks = c.channels.map((ch) => toTrack(ch.target, ch.path, sampleChannel(ch, c, json.sampleRate), rest));
    const clip = new THREE.AnimationClip(name, c.duration, mergeRotations(tracks, rest).map((t) => reduceKeys(t)));
    clip.userData = { layer: c.layer, loop: !!c.loop, holdAt: c.holdAt ?? null };
    return clip;
  });
}

// v7 size control: drop interior keys that linear interpolation of their kept neighbours reproduces within tol
// (greedy, per component). Keeps first/last keys, so rest-at-start/end is untouched. 4.9MB -> see .spec/guide/mascot-part-layers.guide.md.
export function reduceKeys(track, tol = 2e-4) {
  const k = track.getValueSize(), T = track.times, V = track.values, keep = [0];
  for (let i = 1; i < T.length - 1; i++) {
    const a = keep[keep.length - 1], u = (T[i] - T[a]) / (T[i + 1] - T[a]);
    let ok = true; for (let j = 0; j < k && ok; j++) ok = Math.abs(V[a * k + j] + (V[(i + 1) * k + j] - V[a * k + j]) * u - V[i * k + j]) < tol;
    // also make sure skipping i does not break any already-skipped key between a and i
    if (ok) for (let m = a + 1; m < i && ok; m++) { const w = (T[m] - T[a]) / (T[i + 1] - T[a]); for (let j = 0; j < k && ok; j++) ok = Math.abs(V[a * k + j] + (V[(i + 1) * k + j] - V[a * k + j]) * w - V[m * k + j]) < tol; }
    if (!ok) keep.push(i);
  }
  if (T.length > 1) keep.push(T.length - 1);
  const times = keep.map((i) => T[i]), values = keep.flatMap((i) => Array.from(V.slice(i * k, i * k + k)));
  const R = new track.constructor(track.name, times, values); return R;
}
