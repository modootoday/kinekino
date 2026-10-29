import * as THREE from 'three';
import { toXY } from '../geometry/svg-space.js';
import { TAIL_JOINTS, EAR_JOINTS, DEFORMERS } from './skeleton.js';
// Skin weights (bone order from createSkeleton): 0 root, 2/3 ear_L/R, 4..7 tail_0..3, 8/9 ear_L/R_tip.
// Tail: smooth falloff from the body into the chain, linear blend between neighbouring tail bones.
// Ears: falloff from ear base toward the logo corner; outer half blends into the tip bone so only the tip flutters.
const TAIL_FALLOFF_Y = 0.7; // v8: 0.4 -> 0.7, wider body->tail gradient (slime-like root)
const TAIL_ROOT_LEAD = 8 / 40; // v8: ramp starts 8 SVG units inside the body (was at the strip edge)
const TAIL_BASE = 4, EAR = { L: [2, 8], R: [3, 9] };
const S = THREE.MathUtils;
function tailInfluence(x, y, xs, baseX, tipX, bandY) {
  if (y >= bandY || x >= baseX + TAIL_ROOT_LEAD) return null;
  // Vertical falloff above the tail (was a hard cut at bandY -> tearing/fold at the left edge when the tail lifts).
  const infl = S.smootherstep(baseX + TAIL_ROOT_LEAD - x, 0, (baseX + TAIL_ROOT_LEAD - tipX) * 0.8) * S.smootherstep(bandY - y, 0, TAIL_FALLOFF_Y);
  const u = S.clamp((xs[0] - x) / (xs[0] - tipX), 0, 1) * (xs.length - 1);
  const k = Math.min(Math.floor(u), xs.length - 2), f = u - k;
  return [[TAIL_BASE + k, infl * (1 - f)], [TAIL_BASE + k + 1, infl * f]];
}
function earInfluence(x, y, side) {
  const j = EAR_JOINTS[side], b = toXY(...j.base), t = toXY(...j.tip);
  const dx = t[0] - b[0], dy = t[1] - b[1], len2 = dx * dx + dy * dy;
  const u = ((x - b[0]) * dx + (y - b[1]) * dy) / len2; // 0 at base, 1 at corner
  if (u <= -0.6) return null;
  const infl = S.smootherstep(u, -0.6, 0.45), tip = S.smootherstep(u, 0.45, 0.95);
  const [root, tipBone] = EAR[side];
  return [[root, infl * (1 - tip)], [tipBone, infl * tip]];
}
// Deformers (bones 10..12): weight = smootherstep falloff, scaled by (1 - tail/ear influence) so chains stay rigid to their bones.
// Vertices without chain influence get up to 3 deformers (slots 1..3); chain vertices get the strongest one in slot 3.
const DEF = Object.values(DEFORMERS).map((d, i) => ({ bone: 10 + i, c: toXY(...d.c), r: d.r / 40 }));
function deformInfluence(x, y, free) {
  const all = DEF.map((d) => [d.bone, S.smootherstep(1 - Math.hypot(x - d.c[0], y - d.c[1]) / d.r, 0, 1)]).filter(([, v]) => v > 1e-4);
  all.sort((a, b) => b[1] - a[1]); const pick = all.slice(0, free), s = pick.reduce((a, [, v]) => a + v, 0);
  return s > 0.9 ? pick.map(([b, v]) => [b, (v * 0.9) / s]) : pick; // leave >= 10% on root
}
export function applyTailSkin(geometry) {
  const pos = geometry.attributes.position, n = pos.count;
  const idx = new Uint16Array(n * 4), w = new Float32Array(n * 4);
  const xs = TAIL_JOINTS.map((j) => toXY(...j)[0]);
  const [baseX] = toXY(44, 0), tipX = xs[xs.length - 1], [, bandY] = toXY(0, 90), [, earY] = toXY(0, 40);
  for (let i = 0; i < n; i++) {
    const x = pos.getX(i), y = pos.getY(i);
    const inf = tailInfluence(x, y, xs, baseX, tipX, bandY)
      ?? (y > earY ? earInfluence(x, y, x < 0 ? 'L' : 'R') : null) ?? [];
    const sum0 = inf.reduce((a, [, v]) => a + v, 0);
    const def = deformInfluence(x, y, 3 - inf.length).map(([b, v]) => [b, v * (1 - sum0)]);
    const all = [...inf, ...def], sum = all.reduce((a, [, v]) => a + v, 0);
    const slots = [[0, 1 - sum], ...all, [0, 0], [0, 0], [0, 0]].slice(0, 4);
    slots.forEach(([b, v], s) => { idx[i * 4 + s] = b; w[i * 4 + s] = v; });
  }
  geometry.setAttribute('skinIndex', new THREE.Uint16BufferAttribute(idx, 4));
  geometry.setAttribute('skinWeight', new THREE.Float32BufferAttribute(w, 4));
  return geometry;
}
