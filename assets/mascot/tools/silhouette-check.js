// Front silhouette check: orthographic XY raster of the static body mesh vs the 2D logo polygon (IoU + max outline deviation).
import './node-polyfills.js';
import { createBodyGeometry, BODY_POINTS } from '../src/geometry/body.js';
import { toXY } from '../src/geometry/svg-space.js';
const N = 1200, S = N / 3.2, px = (x) => (x + 1.6) * S, py = (y) => (1.6 - y) * S;
const a = new Uint8Array(N * N), b = new Uint8Array(N * N);
function fillTri(buf, p) {
  const [x0, y0, x1, y1, x2, y2] = p; const minx = Math.max(0, Math.floor(Math.min(x0, x1, x2))), maxx = Math.min(N - 1, Math.ceil(Math.max(x0, x1, x2)));
  const miny = Math.max(0, Math.floor(Math.min(y0, y1, y2))), maxy = Math.min(N - 1, Math.ceil(Math.max(y0, y1, y2)));
  const e = (ax, ay, bx, by, cx, cy) => (bx - ax) * (cy - ay) - (by - ay) * (cx - ax), A = e(x0, y0, x1, y1, x2, y2); if (!A) return;
  for (let y = miny; y <= maxy; y++) for (let x = minx; x <= maxx; x++) {
    const cx = x + 0.5, cy = y + 0.5, w0 = e(x1, y1, x2, y2, cx, cy) / A, w1 = e(x2, y2, x0, y0, cx, cy) / A, w2 = e(x0, y0, x1, y1, cx, cy) / A;
    if (w0 >= 0 && w1 >= 0 && w2 >= 0) buf[y * N + x] = 1;
  }
}
const g = createBodyGeometry(), p = g.attributes.position, idx = g.index.array;
for (let i = 0; i < idx.length; i += 3) fillTri(a, [0, 1, 2].flatMap((k) => [px(p.getX(idx[i + k])), py(p.getY(idx[i + k]))]));
const poly = BODY_POINTS.map((q) => toXY(...q)).map(([x, y]) => [px(x), py(y)]);
for (const [i, j] of [[2, 3], [3, 4], [4, 0]]) fillTri(b, [...poly[1], ...poly[i], ...poly[j]]); // fan from the notch vertex (concave polygon)
let inter = 0, uni = 0; for (let i = 0; i < a.length; i++) { inter += a[i] & b[i]; uni += a[i] | b[i]; }

const ext = (arr) => { let mnx = N, mxx = 0, mny = N, mxy = 0; for (let i = 0; i < arr.length; i++) if (arr[i]) { const x = i % N, y = (i / N) | 0; mnx = Math.min(mnx, x); mxx = Math.max(mxx, x); mny = Math.min(mny, y); mxy = Math.max(mxy, y); } return [mnx, mxx, mny, mxy]; };
const ea = ext(a), eb = ext(b), svgPerPx = 120 / (3 * S);
console.log(JSON.stringify({ iou: +(inter / uni).toFixed(4), boundsDiffSvgUnits: ea.map((v, i) => +((v - eb[i]) * svgPerPx).toFixed(2)), verts: p.count }));
