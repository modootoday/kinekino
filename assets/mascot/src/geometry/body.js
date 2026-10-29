import * as THREE from 'three';
import { mergeVertices, mergeGeometries } from 'three/examples/jsm/utils/BufferGeometryUtils.js';
import { shapeFromSvg } from './polygon.js';
import { toXY, UNIT } from './svg-space.js';
import { PALETTE } from './palette.js';
import { densify, pillowZ, pillowNormal, rimDistance, PILLOW_R } from './dome.js';
// Pillow/jelly body: one closed volume whose front and back are height fields z = ±pillowZ(x, y).
// pillowZ is 0 exactly on the logo outline (front silhouette unchanged) and rises with a circular
// roll-off (vertical tangent at the rim), so every side edge is a large-radius curve instead of a flat wall.
// Colour regions (jaw band + tail strip, inner ears) are painted into COLOR_0 on the body itself; the triangles
// are cut exactly along the logo's colour boundaries, so the front colour edge is crisp and identical to the logo.
export const BODY_POINTS = [[20, 14], [60, 36], [100, 14], [100, 74], [20, 106]];
export const BODY_CORNER_RADIUS = 1.5; // SVG units, softens tips without changing the logo
export const BODY_MAX_EDGE = 0.085, RIM_MAX_EDGE = 0.05;
export const BODY_AO = (globalThis.process?.env?.SURFACE ?? 'bake') === 'bake' ? 0.14 : 0; // 0 keeps the rest pose a flat #FF5A1F
export const FRONT_Z = 0; // eyes are placed relative to pillowZ(x, y)
// Colour regions in SVG units (jaw band and tail strip share one straight boundary line, slope -0.4).
// v7 blush: soft cheek discs (SVG centre, radius) revealed by a second COLOR_0 morph target ('blush'); front only.
export const BLUSH = { color: 0xff7f8f, spots: [[45, 70], [75, 70]], r: 5.5 };
export const BAND_POLY = [[20, 99], [100, 67], [100, 74], [20, 106]];
export const EAR_L_POLY = [[26, 22], [26, 34], [38, 30]], EAR_R_POLY = [[94, 22], [82, 30], [94, 34]];
const REGIONS = [[BAND_POLY, PALETTE.jaw], [EAR_L_POLY, PALETTE.ear], [EAR_R_POLY, PALETTE.ear]].map(([p, c]) => [p.map((q) => toXY(...q)), c]);
const CUT_LINES = [[BAND_POLY[0], BAND_POLY[1]], ...[EAR_L_POLY, EAR_R_POLY].flatMap((p) => p.map((q, i) => [q, p[(i + 1) % 3]]))]
  .map(([a, b]) => [toXY(...a), toXY(...b)]);
function inPoly([x, y], poly) {
  let c = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const [xi, yi] = poly[i], [xj, yj] = poly[j];
    if ((yi > y) !== (yj > y) && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi) c = !c;
  }
  return c;
}
// Splits every triangle crossing the infinite line a-b (shared edges are cut at the same point: no T-junctions).
function cutByLine(tris, [a, b]) {
  const nx = a[1] - b[1], ny = b[0] - a[0], c = -(nx * a[0] + ny * a[1]), out = [];
  const sd = (p) => { const s = nx * p[0] + ny * p[1] + c; return Math.abs(s) < 1e-9 ? 0 : s; };
  const lerp = (p, q, sp, sq) => { const t = sp / (sp - sq); return [p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t]; };
  for (const t of tris) {
    const s = t.map(sd);
    if (s.every((v) => v >= 0) || s.every((v) => v <= 0)) { out.push(t); continue; }
    // rotate so vertex 0 is alone on its side (the other two are on the other side or on the line)
    let k = [0, 1, 2].find((i) => s[i] !== 0 && Math.sign(s[(i + 1) % 3]) !== Math.sign(s[i]) && Math.sign(s[(i + 2) % 3]) !== Math.sign(s[i]));
    const [p0, p1, p2] = [t[k], t[(k + 1) % 3], t[(k + 2) % 3]], [s0, s1, s2] = [s[k], s[(k + 1) % 3], s[(k + 2) % 3]];
    if (s1 === 0) { const m = lerp(p0, p2, s0, s2); out.push([p0, p1, m], [m, p1, p2]); continue; }
    if (s2 === 0) { const m = lerp(p0, p1, s0, s1); out.push([p0, m, p2], [m, p1, p2]); continue; }
    const m1 = lerp(p0, p1, s0, s1), m2 = lerp(p0, p2, s0, s2);
    out.push([p0, m1, m2], [m1, p1, p2], [m1, p2, m2]);
  }
  return out;
}
const toTris = (g) => { const p = g.attributes.position, t = []; for (let i = 0; i < p.count; i += 3) t.push([0, 1, 2].map((k) => [p.getX(i + k), p.getY(i + k)])); return t; };
const fromTris = (t) => { const b = new THREE.BufferGeometry(); b.setAttribute('position', new THREE.Float32BufferAttribute(t.flatMap((tr) => tr.flatMap(([x, y]) => [x, y, 0])), 3)); return b; };
function refineRim(g) { // steep rim roll-off: triangles near the outline are refined much finer (no faceted edge)
  const near = [], far = [];
  for (const t of toTris(g)) (t.some(([x, y]) => rimDistance(x, y) < PILLOW_R * 1.1) ? near : far).push(t);
  return mergeGeometries([densify(fromTris(near), RIM_MAX_EDGE), fromTris(far)]);
}
// Conforming refinement: an edge longer than maxEdge is split at its midpoint for BOTH neighbours (global edge map),
// so no T-junctions/cracks appear (TessellateModifier splits per triangle and left cracks on the steep rim).
function refineConforming(tris, maxEdge) {
  const key = (a, b) => { const ka = a[0].toFixed(7) + ',' + a[1].toFixed(7), kb = b[0].toFixed(7) + ',' + b[1].toFixed(7); return ka < kb ? ka + '|' + kb : kb + '|' + ka; };
  for (let it = 0; it < 40; it++) {
    const mid = new Map(); let any = false;
    for (const t of tris) for (let i = 0; i < 3; i++) { const a = t[i], b = t[(i + 1) % 3];
      if (Math.hypot(a[0] - b[0], a[1] - b[1]) > maxEdge) { mid.set(key(a, b), [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2]); any = true; } }
    if (!any) break;
    const out = [];
    for (const t of tris) {
      const m = [0, 1, 2].map((i) => mid.get(key(t[i], t[(i + 1) % 3]))), c = m.filter(Boolean).length;
      if (c === 0) { out.push(t); continue; }
      if (c === 3) { out.push([t[0], m[0], m[2]], [m[0], t[1], m[1]], [m[2], m[1], t[2]], [m[0], m[1], m[2]]); continue; }
      const r = [0, 1, 2].find((i) => (c === 1 ? m[i] : !m[i])); // c=1: split edge r; c=2: edge r unsplit
      const [A, B, C] = [t[r], t[(r + 1) % 3], t[(r + 2) % 3]], [mAB, mBC, mCA] = [m[r], m[(r + 1) % 3], m[(r + 2) % 3]];
      if (c === 1) out.push([A, mAB, C], [mAB, B, C]);
      else out.push([A, B, mBC], [A, mBC, mCA], [mCA, mBC, C]);
    }
    tris = out;
  }
  return tris;
}
export function createBodyGeometry() {
  const base = new THREE.ShapeGeometry(shapeFromSvg(BODY_POINTS, 32, BODY_CORNER_RADIUS)).toNonIndexed();
  let tris = toTris(base); for (const l of CUT_LINES) tris = cutByLine(tris, l);
  const area = ([a, b, c]) => Math.abs((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]));
  const front = fromTris(refineConforming(tris.filter((t) => area(t) > 1e-10), RIM_MAX_EDGE).filter((t) => area(t) > 1e-10 && !t.every(([x, y]) => pillowZ(x, y) < 1e-3))); // drop flat fins lying on the outline
  const p = front.attributes.position, n = p.count, bodyC = new THREE.Color(PALETTE.body);
  const pos = new Float32Array(n * 6), nor = new Float32Array(n * 6), col = new Float32Array(n * 6), dcol = new Float32Array(n * 6), bcol = new Float32Array(n * 6);
  const blushC = new THREE.Color(BLUSH.color), blushD = [blushC.r - bodyC.r, blushC.g - bodyC.g, blushC.b - bodyC.b];
  const blushF = (x, y) => Math.max(...BLUSH.spots.map(([sx, sy]) => { const [bx, by] = toXY(sx, sy), d = Math.hypot(x - bx, y - by) / (BLUSH.r * UNIT);
    return d >= 1 ? 0 : 1 - d * d * (3 - 2 * d); }));
  for (let i = 0; i < n; i += 3) {
    const cx = (p.getX(i) + p.getX(i + 1) + p.getX(i + 2)) / 3, cy = (p.getY(i) + p.getY(i + 1) + p.getY(i + 2)) / 3;
    const reg = REGIONS.find(([poly]) => inPoly([cx, cy], poly));
    const rc = reg && new THREE.Color(reg[1]); // linear-space delta (Color.sub clamps at 0, so subtract manually)
    const d = { toArray: () => (rc ? [rc.r - bodyC.r, rc.g - bodyC.g, rc.b - bodyC.b] : [0, 0, 0]) };
    for (let k = 0; k < 3; k++) {
      const v = i + k, x = p.getX(v), y = p.getY(v), z0 = pillowZ(x, y), rim = z0 < 1e-3; // outline vertex: snap to z=0 so front/back weld (no seam crack)
      const z = rim ? 0 : z0, [nx, ny, nz] = rim ? pillowNormal(x, y, 1) : pillowNormal(x, y);
      const j = n + i + [0, 2, 1][k]; // back face: mirrored z, reversed winding
      pos.set([x, y, z], v * 3); nor.set([nx, ny, nz], v * 3); pos.set([x, y, -z], j * 3); nor.set([nx, ny, -nz], j * 3);
      const ao = BODY_AO ? 1 - BODY_AO * (1 - Math.min(rimDistance(x, y) / PILLOW_R, 1)) ** 2 : 1; // baked soft rim occlusion
      const bc = bodyC.toArray().map((c) => c * ao); col.set(bc, v * 3); col.set(bc, j * 3);
      dcol.set(d.toArray(), v * 3); if (rim) dcol.set(d.toArray(), j * 3);
      const bf = blushF(x, y); bcol.set(blushD.map((c) => c * bf), v * 3); // regions are front-only; rim vertices share the delta so they weld
    }
  }
  let g = new THREE.BufferGeometry();
  g.setAttribute('position', new THREE.BufferAttribute(pos, 3)); g.setAttribute('normal', new THREE.BufferAttribute(nor, 3));
  g.setAttribute('color', new THREE.BufferAttribute(col, 3));
  g.morphAttributes.position = [0, 1].map(() => new THREE.BufferAttribute(new Float32Array(n * 6), 3));
  g.morphAttributes.color = [new THREE.BufferAttribute(dcol, 3), new THREE.BufferAttribute(bcol, 3)]; // 0 reveal, 1 blush
  g.morphTargetsRelative = true;
  return mergeVertices(g, 1e-6); // vertices on a colour boundary stay split (different deltas) -> hard, crisp edge
}
