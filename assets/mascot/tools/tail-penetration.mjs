// Strict tail-into-body metric per tail clip: tail-weighted vertices (tail weight > 0.3) inside the posed body slab
// (body-dominant vertices, per 0.02 XY cell) with a small margin. Prints ratio and max depth (world units; 1 = 40 SVG).
import { mascot, clipsByName, poseAt, skinned } from '../tests/_mascot.mjs';
import { BODY_POINTS } from '../src/geometry/body.js';
import { pillowZ } from '../src/geometry/dome.js';
import { toXY } from '../src/geometry/svg-space.js';
const mesh = mascot.getObjectByName('body'), bones = mesh.skeleton.bones.map((b) => b.name);
const si = mesh.geometry.attributes.skinIndex, sw = mesh.geometry.attributes.skinWeight, N = si.count;
const wt = (i, p) => { let w = 0; for (let k = 0; k < 4; k++) if (p(bones[si.getComponent(i, k)])) w += sw.getComponent(i, k); return w; };
const tw = Array.from({ length: N }, (_, i) => wt(i, (b) => b.startsWith('tail')));
const isTail = tw.map((w) => w > 0.5), isBody = tw.map((w) => w < 0.02);
const nTail = isTail.filter(Boolean).length;
// Body volume = rest logo polygon (minus the tail strip zone) between back/front height fields z = -/+pillowZ.
const POLY = BODY_POINTS.map((q) => toXY(...q));
const inPoly = (x, y) => { let c = false; for (let i = 0, j = POLY.length - 1; i < POLY.length; j = i++) { const [xi, yi] = POLY[i], [xj, yj] = POLY[j]; if ((yi > y) !== (yj > y) && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi) c = !c; } return c; };
const inBody = (x, y) => { const sx = x * 40 + 60, sy = 60 - y * 40; return inPoly(x, y) && !(sx < 46 && sy > 95); };
export function metric(P, margin = 0.005) {
  let n = 0, depth = 0, ov = 0;
  for (let i = 0; i < N; i++) if (isTail[i]) { const x = P[i * 3], y = P[i * 3 + 1], z = P[i * 3 + 2]; if (!inBody(x, y)) continue;
    ov++; const d = pillowZ(x, y) - Math.abs(z); if (d > margin) { n++; depth = Math.max(depth, d); } }
  return { ratio: n / nTail, depth, overlap: ov / nTail };
}
if (process.argv[1]?.endsWith('tail-penetration.mjs')) main();
function main() {
const clips = [...clipsByName.keys()].filter((c) => c.startsWith('tail.') || c === 'body.groom');
poseAt(null, 0); const r0 = metric(skinned());
const out = { rest: r0 };
for (const c of clips) { const d = clipsByName.get(c).duration; let w = { ratio: 0, depth: 0, overlap: 0, t: 0 };
  for (let k = 1; k < 20; k++) { const t = d * k / 20; poseAt(c, t); const m = metric(skinned()); if (m.overlap + m.depth > w.overlap + w.depth) w = { ...m, t: +t.toFixed(2) }; }
  out[c] = { overlapPct: +(w.overlap * 100).toFixed(2), insidePct: +(w.ratio * 100).toFixed(2), depthSvg: +(w.depth * 40).toFixed(2), worstT: w.t }; }
console.log(JSON.stringify(out, null, 1));
}
