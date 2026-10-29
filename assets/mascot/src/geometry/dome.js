import { TessellateModifier } from 'three/examples/jsm/modifiers/TessellateModifier.js';
import { toXY } from './svg-space.js';
// Face convexity: height field that is 0 on the logo outline and rises toward the interior,
// so the front silhouette is unchanged while the face bulges forward.
const OUTLINE = [[20, 14], [60, 36], [100, 14], [100, 74], [20, 106]].map((p) => toXY(...p));
export const DOME_HEIGHT = 0.16, DOME_REACH = 0.7, SOFTNESS = 0.08;
function edgeDistance(x, y) {
  // Soft-min over the outline edges removes medial-axis creases (no pinched 'pillow' seams).
  let sum = 0;
  for (let i = 0; i < OUTLINE.length; i++) {
    const [ax, ay] = OUTLINE[i], [bx, by] = OUTLINE[(i + 1) % OUTLINE.length];
    const dx = bx - ax, dy = by - ay, t = Math.max(0, Math.min(1, ((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy)));
    sum += Math.exp(-Math.hypot(x - ax - t * dx, y - ay - t * dy) / SOFTNESS);
  }
  return Math.max(0, -SOFTNESS * Math.log(sum));
}
export function domeZ(x, y) {
  const u = Math.min(edgeDistance(x, y) / DOME_REACH, 1);
  return DOME_HEIGHT * (1 - (1 - u) ** 3);
}
// Splits faces until edges are <= maxEdge (non-indexed geometry in, non-indexed out).
export function densify(geo, maxEdge, maxIterations = 40) {
  return new TessellateModifier(maxEdge, maxIterations).modify(geo.index ? geo.toNonIndexed() : geo);
}
// Lifts every vertex whose z passes `minZ` by the dome height (used for body front cap and front plates).
export function liftByDome(geo, minZ = -Infinity) {
  const p = geo.attributes.position;
  for (let i = 0; i < p.count; i++) if (p.getZ(i) >= minZ) p.setZ(i, p.getZ(i) + domeZ(p.getX(i), p.getY(i)));
  p.needsUpdate = true; return geo;
}
// Analytic surface normal of the dome (smooth shading independent of triangle quality).
export function domeNormal(x, y, e = 1e-3) {
  const gx = (domeZ(x + e, y) - domeZ(x - e, y)) / (2 * e), gy = (domeZ(x, y + e) - domeZ(x, y - e)) / (2 * e);
  const l = Math.hypot(gx, gy, 1); return [-gx / l, -gy / l, 1 / l];
}
// Overwrites normals of vertices lying on the domed surface z = baseZ + domeZ(x, y).
export function setDomeNormals(geo, baseZ, tol = 2e-4) {
  const p = geo.attributes.position, n = geo.attributes.normal;
  for (let i = 0; i < p.count; i++) {
    const x = p.getX(i), y = p.getY(i);
    if (Math.abs(p.getZ(i) - baseZ - domeZ(x, y)) < tol) n.setXYZ(i, ...domeNormal(x, y));
  }
  n.needsUpdate = true; return geo;
}
// Front surface height of the body including the bevel roll-off (matches ExtrudeGeometry's bevel profile:
// inset = size*(1-sin θ), z = depth/2 + thickness*cos θ). Used to emboss plates so they wrap over the rounded edge.
export function surfaceZ(x, y, depthHalf, thickness, size) {
  let d = Infinity;
  for (let i = 0; i < OUTLINE.length; i++) {
    const [ax, ay] = OUTLINE[i], [bx, by] = OUTLINE[(i + 1) % OUTLINE.length];
    const dx = bx - ax, dy = by - ay, t = Math.max(0, Math.min(1, ((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy)));
    d = Math.min(d, Math.hypot(x - ax - t * dx, y - ay - t * dy));
  }
  if (d >= size) return depthHalf + thickness + domeZ(x, y);
  const sin = 1 - d / size; return depthHalf + thickness * Math.sqrt(Math.max(0, 1 - sin * sin));
}

// ---- Pillow volume (v5) ----
import { shapeFromSvg as _shape } from './polygon.js';
const RIM = _shape([[20, 14], [60, 36], [100, 14], [100, 74], [20, 106]], 64, 1.5).getPoints().map((v) => [v.x, v.y]);
export const PILLOW_R = 0.34; // rim roll-off radius (world)
export function rimDistance(x, y) {
  let d = Infinity;
  for (let i = 0; i < RIM.length; i++) {
    const [ax, ay] = RIM[i], [bx, by] = RIM[(i + 1) % RIM.length];
    const dx = bx - ax, dy = by - ay, l2 = dx * dx + dy * dy || 1e-12;
    const t = Math.max(0, Math.min(1, ((x - ax) * dx + (y - ay) * dy) / l2));
    d = Math.min(d, Math.hypot(x - ax - t * dx, y - ay - t * dy));
  }
  return d;
}
function insideRim(x, y) {
  let c = false;
  for (let i = 0, j = RIM.length - 1; i < RIM.length; j = i++) {
    const [xi, yi] = RIM[i], [xj, yj] = RIM[j];
    if ((yi > y) !== (yj > y) && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi) c = !c;
  }
  return c;
}
// Half thickness: circular rim roll-off + soft interior dome (0 on the outline).
export function pillowZ(x, y) {
  const s = Math.min(rimDistance(x, y) / PILLOW_R, 1);
  return PILLOW_R * Math.sqrt(Math.max(0, 1 - (1 - s) ** 2)) + domeZ(x, y);
}
export function pillowNormal(x, y, e = 2e-4) {
  const d = rimDistance(x, y);
  if (d < 2 * e) { // on the rim: outward in-plane normal (signed distance: unsigned d has zero central gradient on the outline)
    const sd = (u, v) => (insideRim(u, v) ? 1 : -1) * rimDistance(u, v), h = 1e-3;
    const gx = sd(x + h, y) - sd(x - h, y), gy = sd(x, y + h) - sd(x, y - h);
    const l = Math.hypot(gx, gy) || 1; return [-gx / l, -gy / l, 0];
  }
  const gx = (pillowZ(x + e, y) - pillowZ(x - e, y)) / (2 * e), gy = (pillowZ(x, y + e) - pillowZ(x, y - e)) / (2 * e);
  const l = Math.hypot(gx, gy, 1); return [-gx / l, -gy / l, 1 / l];
}
