import * as THREE from 'three';
import { shapeFromSvg } from './polygon.js';
import { densify, pillowZ, pillowNormal } from './dome.js';
import { mergeVertices } from 'three/examples/jsm/utils/BufferGeometryUtils.js';
// Thin front-face plates (ears, jaw band, tail overlay) sitting just in front of the body.
const FRONT_Z = 0;
// Emboss: every plate vertex rides the body's front surface (dome + bevel roll-off), so plates hug the rounded edge.
function emboss(g) {
  const p = g.attributes.position;
  for (let i = 0; i < p.count; i++) {
    const x = p.getX(i), y = p.getY(i);
    p.setZ(i, pillowZ(x, y) + 0.008 + (p.getZ(i) - FRONT_Z));
  }
  p.needsUpdate = true; return g;
}
function plate(points, steps = 1) {
  let g = new THREE.ExtrudeGeometry(shapeFromSvg(points, steps), { depth: 0.01, bevelEnabled: false });
  g.translate(0, 0, FRONT_Z); g.deleteAttribute('uv');
  // Follow the face dome so plates hug the curved front.
  g = mergeVertices(emboss(densify(g, 0.022)), 1e-6); g.computeVertexNormals();
  // Front (top) face of the plate shares the body's analytic normal, so plate/body shading is continuous.
  { const p = g.attributes.position, n = g.attributes.normal;
    for (let i = 0; i < p.count; i++) { const x = p.getX(i), y = p.getY(i);
      if (p.getZ(i) - pillowZ(x, y) > 0.012) n.setXYZ(i, ...pillowNormal(x, y)); } n.needsUpdate = true; }
  return g;
}
export const EAR_LEFT_POINTS = [[26, 22], [26, 34], [38, 30]];
export const EAR_RIGHT_POINTS = [[94, 22], [82, 30], [94, 34]];
export const JAW_POINTS = [[37, 99.4], [100, 74], [100, 67], [37, 92.2]];
export const TAIL_POINTS = [[20, 99], [37, 92.2], [37, 99.4], [20, 106]];
export const createEarGeometry = (pts) => plate(pts, 4);
export const createJawGeometry = () => plate(JAW_POINTS, 16);
// Morph target 0 'hidden' collapses the overlay to its centroid (glTF-legal visibility on a skinned mesh).
export function createTailOverlayGeometry() {
  const g = plate(TAIL_POINTS, 12); g.computeBoundingBox();
  const c = g.boundingBox.getCenter(new (g.boundingBox.min.constructor)());
  const p = g.attributes.position, d = new Float32Array(p.count * 3);
  for (let i = 0; i < p.count; i++) d.set([c.x - p.getX(i), c.y - p.getY(i), c.z - p.getZ(i)], i * 3);
  g.morphAttributes.position = [new (p.constructor)(d, 3)]; g.morphTargetsRelative = true;
  return g;
}
