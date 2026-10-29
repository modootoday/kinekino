import * as THREE from 'three';
import { UNIT } from './svg-space.js';
import { PALETTE } from './palette.js';
// Capsule (stadium) eye: straight left/right sides, semicircular top/bottom caps. Solid ink, pivot at the centre so the
// eye-layer scaleY blink compresses it symmetrically.
// Geometry (v7): own indexed mesh instead of ExtrudeGeometry. The front/back faces are triangulated as vertical strips
// (outline point i is paired with its mirror across the horizontal axis, N/2 - i), so every morph target that keeps
// "one top point and one bottom point per x" (capsule, clipped capsule, arched band) stays fold-free.
// Morph targets (absolute POSITION, glTF standard), same N outline vertices remapped by arc-length fraction:
//   0 squint      horizontal capsule (wider, very low)
//   1 round       circle of equal area
//   2 wide        capsule x1.15
//   3 half_lid    capsule with the top cut flat (upper lid half down); bottom cap stays a semicircle
//   4 closed_happy thin horizontal capsule bent up into an arch (^ ^)
//   5 glare       top cut on a slant, lower toward the inner side (side-aware: L inner = +x, R inner = -x)
//   6 worried     top cut on a slant, higher toward the inner side, slightly lowered outer corner
//   7 dilate      capsule stretched vertically only (x1.22 in y)
//   8 sparkle     base capsule + a small highlight diamond (COLOR_0 white) that is collapsed to a point otherwise
export const EYE_MORPHS = ['squint', 'round', 'wide', 'half_lid', 'closed_happy', 'glare', 'worried', 'dilate', 'sparkle'];
const N = 64; // outline samples (multiple of 4)
const HL = 8; // highlight ring vertices
export function eyeHalfExtents(shape) { return [shape.width / 2, shape.straight / 2 + shape.capRadius]; }
// Stadium outline with half extents a (x), b (y), N points by arc length, starting at the top centre, CCW.
export function stadium(a, b, r = Math.min(a, b)) {
  const vert = b >= a, L = vert ? b - r : a - r, segs = [];
  const arc = (cx, cy, a0) => [Math.PI * r, (t) => [cx + r * Math.cos(a0 + Math.PI * t), cy + r * Math.sin(a0 + Math.PI * t)]];
  const line = (x0, y0, x1, y1) => [Math.hypot(x1 - x0, y1 - y0), (t) => [x0 + (x1 - x0) * t, y0 + (y1 - y0) * t]];
  if (vert) {
    segs.push([Math.PI * r / 2, (t) => { const q = Math.PI / 2 + (Math.PI / 2) * t; return [r * Math.cos(q), L + r * Math.sin(q)]; }],
      line(-r, L, -r, -L), arc(0, -L, Math.PI), line(r, -L, r, L),
      [Math.PI * r / 2, (t) => { const q = (Math.PI / 2) * t; return [r * Math.cos(q), L + r * Math.sin(q)]; }]);
  } else segs.push(line(0, r, -L, r), arc(-L, 0, Math.PI / 2), line(-L, -r, L, -r), arc(L, 0, -Math.PI / 2), line(L, r, 0, r));
  const total = segs.reduce((s, [l]) => s + l, 0), pts = [];
  for (let i = 0; i < N; i++) {
    let d = (i / N) * total;
    for (let k = 0; k < segs.length; k++) { const [l, f] = segs[k]; if (d <= l + 1e-9 || k === segs.length - 1) { pts.push(f(l ? Math.min(d / l, 1) : 0)); break; } d -= l; }
  }
  // exact symmetry: point i and N/2 - i are mirrors across y = 0 (kills arc-length float drift)
  for (let i = 1; i < N / 4; i++) { const j = N / 2 - i; pts[j] = [pts[i][0], -pts[i][1]]; pts[N - i] = [-pts[i][0], pts[i][1]]; pts[N / 2 + i] = [-pts[i][0], -pts[i][1]]; }
  pts[N / 4] = [pts[N / 4][0], 0]; pts[3 * N / 4] = [-pts[N / 4][0], 0];
  return pts;
}
// Top of the outline (points whose partner is below them) is clamped to y <= cut(x). Keeps one top + one bottom per x.
// The partner (mirror across y=0) of top point i is N/2 - i (left half) / 3N/2 - i (right half); keep >= 0.4 above it.
const partner = (i) => (i <= N / 2 ? N / 2 - i : (3 * N) / 2 - i) % N;
const clipTop = (pts, cut) => pts.map(([x, y], i) => (i < N / 4 || i > (3 * N) / 4 ? [x, Math.max(Math.min(y, cut(x)), pts[partner(i)][1] + 0.4)] : [x, y]));
export function eyeOutlines(shape, side = 'L') {
  const [a, b] = eyeHalfExtents(shape), r = shape.capRadius, base = stadium(a, b, r), inner = side === 'L' ? 1 : -1;
  const areaR = Math.sqrt((Math.PI * r ** 2 + shape.width * shape.straight) / Math.PI);
  const arch = stadium(a * 1.12, 1.15).map(([x, y]) => [x, y + 2.6 * (1 - (x / (a * 1.12)) ** 2) - 1.2]);
  const keepBottom = (cut) => clipTop(base, cut);
  return {
    base,
    squint: stadium(a * 1.08, b * 0.28), round: stadium(areaR, areaR), wide: stadium(a * 1.15, b * 1.15, r * 1.15),
    half_lid: keepBottom(() => b * 0.05),
    closed_happy: arch,
    glare: keepBottom((x) => b * 0.3 - 0.5 * x * inner),
    worried: keepBottom((x) => b * 0.35 + 0.45 * x * inner),
    dilate: base.map(([x, y]) => [x, y * 1.22]),
    sparkle: base,
  };
}
// Highlight: diamond (centre + HL ring) at the upper outer area; collapsed to its centre in every target but sparkle.
const HL_C = [-1.1, 3.4], HL_R = 1.35;
const hlRing = (s) => Array.from({ length: HL }, (_, k) => { const t = (k / HL) * 2 * Math.PI, rr = s * HL_R * (k % 2 ? 0.45 : 1); return [HL_C[0] + rr * Math.cos(t), HL_C[1] + rr * Math.sin(t)]; });
export function createEyeGeometry(shape, side = 'L') {
  const O = eyeOutlines(shape, side), Z0 = 0.012, Z1 = 0.032, ZH = Z1 + 0.002;
  // vertex layout: [front outline N][back outline N][side walls 4N (duplicated for flat normals)][hl centre + ring]
  const layout = (pts, hl) => {
    const v = [];
    for (const [x, y] of pts) v.push([x, y, Z1]);
    for (const [x, y] of pts) v.push([x, y, Z0]);
    for (let i = 0; i < N; i++) { const p = pts[i], q = pts[(i + 1) % N]; v.push([p[0], p[1], Z1], [q[0], q[1], Z1], [q[0], q[1], Z0], [p[0], p[1], Z0]); }
    v.push([HL_C[0], HL_C[1], ZH], ...hl.map(([x, y]) => [x, y, ZH]));
    return v.flatMap(([x, y, z]) => [x * UNIT, y * UNIT, z]);
  };
  const collapsed = Array.from({ length: HL }, () => HL_C);
  const pos = layout(O.base, collapsed), idx = [];
  const strip = (off, flip) => { // vertical strips: left half pairs (i, N/2-i), right half (N/2+j, N-j)
    const tri = (a, b, c) => (flip ? idx.push(off + a, off + c, off + b) : idx.push(off + a, off + b, off + c));
    for (let i = 0; i < N / 4; i++) { const a = i, b = i + 1, c = N / 2 - i - 1, d = N / 2 - i; tri(a, b, c); tri(a, c, d); }
    for (let j = 0; j < N / 4; j++) { const a = N / 2 + j, b = N / 2 + j + 1, c = (N - j - 1) % N, d = (N - j) % N; tri(a, b, c); tri(a, c, d); }
  };
  strip(0, false); strip(N, true);
  for (let i = 0; i < N; i++) { const o = 2 * N + 4 * i; idx.push(o, o + 3, o + 2, o, o + 2, o + 1); }
  const hc = 6 * N; for (let k = 0; k < HL; k++) idx.push(hc, hc + 1 + k, hc + 1 + ((k + 1) % HL));
  const g = new THREE.BufferGeometry();
  g.setAttribute('position', new THREE.Float32BufferAttribute(pos, 3)); g.setIndex(idx);
  const ink = new THREE.Color(PALETTE.ink), col = new Float32Array(pos.length);
  for (let v = 0; v < pos.length / 3; v++) (v >= hc ? [1, 1, 1] : ink.toArray()).forEach((c, k) => { col[v * 3 + k] = c; });
  g.setAttribute('color', new THREE.BufferAttribute(col, 3));
  g.morphAttributes.position = EYE_MORPHS.map((m) => new THREE.Float32BufferAttribute(layout(O[m], m === 'sparkle' ? hlRing(1) : collapsed), 3));
  g.computeVertexNormals();
  return g;
}
