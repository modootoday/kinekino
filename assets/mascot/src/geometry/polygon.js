import * as THREE from 'three';
import { toXY } from './svg-space.js';
// Builds a THREE.Shape from SVG points, densifying edges so skinning can bend them.
// radius (SVG units) rounds every corner with a small fillet so large bevels do not fold at acute corners.
export function shapeFromSvg(points, stepsPerEdge = 1, radius = 0, arcSteps = 10) {
  const n = points.length, out = [];
  const pt = (x, y) => out.push(new THREE.Vector2(...toXY(x, y)));
  const corner = (i) => {
    const p = points[i], a = points[(i - 1 + n) % n], b = points[(i + 1) % n];
    const la = Math.hypot(a[0] - p[0], a[1] - p[1]), lb = Math.hypot(b[0] - p[0], b[1] - p[1]);
    const r = Math.min(radius, la / 3, lb / 3);
    return { p, s: [p[0] + (a[0] - p[0]) * r / la, p[1] + (a[1] - p[1]) * r / la], e: [p[0] + (b[0] - p[0]) * r / lb, p[1] + (b[1] - p[1]) * r / lb], r };
  };
  for (let i = 0; i < n; i++) {
    const c = corner(i), next = corner((i + 1) % n);
    if (c.r > 0) for (let k = 0; k < arcSteps; k++) { // quadratic Bezier fillet through the corner
      const t = k / arcSteps, u = 1 - t;
      pt(u * u * c.s[0] + 2 * u * t * c.p[0] + t * t * c.e[0], u * u * c.s[1] + 2 * u * t * c.p[1] + t * t * c.e[1]);
    }
    const [x0, y0] = c.r > 0 ? c.e : c.p, [x1, y1] = next.r > 0 ? next.s : next.p;
    for (let s = c.r > 0 ? 0 : 0; s < stepsPerEdge; s++) {
      if (c.r > 0 && s === 0) { pt(x0, y0); continue; }
      const t = s / stepsPerEdge; pt(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
    }
  }
  return new THREE.Shape(out);
}
