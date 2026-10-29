import * as THREE from 'three';
import { toXY } from '../geometry/svg-space.js';
// Bone index order matters for skinning: 0 root, 1 head, 2 ear_L, 3 ear_R, 4..7 tail_0..3, 8 ear_L_tip, 9 ear_R_tip,
// 10..12 body deformers crown / base_L / base_R (v7, owned by the body layer; identity at rest so the static logo is unchanged).
export const TAIL_JOINTS = [[40, 99], [33, 101.5], [27, 103.5], [21, 105.5]];
// Ear base (root bone) and mid (tip bone) joints, SVG space; ear tips are the logo corners (20,14)/(100,14).
// Body deformers (SVG centre, falloff radius in SVG units). Smooth radial skin falloff; zero inside ears/tail chains.
export const DEFORMERS = { crown: { c: [60, 34], r: 22 }, base_L: { c: [54, 88], r: 20 }, base_R: { c: [86, 72], r: 20 } };
export const EAR_JOINTS = { L: { base: [38, 32], mid: [28, 22], tip: [20, 14] }, R: { base: [82, 32], mid: [92, 22], tip: [100, 14] } };
function bone(name, parent, worldXY) {
  const b = new THREE.Bone(); b.name = name;
  const p = parent.userData.world || [0, 0];
  b.position.set(worldXY[0] - p[0], worldXY[1] - p[1], 0);
  b.userData.world = worldXY; parent.add(b); return b;
}
export function createSkeleton() {
  const root = new THREE.Bone(); root.name = 'root'; root.userData.world = [0, 0];
  const head = bone('head', root, toXY(60, 58));
  const earL = bone('ear_L', root, toXY(...EAR_JOINTS.L.base));
  const earR = bone('ear_R', root, toXY(...EAR_JOINTS.R.base));
  let parent = root; const tail = [];
  TAIL_JOINTS.forEach((j, i) => { parent = bone(`tail_${i}`, parent, toXY(...j)); tail.push(parent); });
  const earLTip = bone('ear_L_tip', earL, toXY(...EAR_JOINTS.L.mid));
  const earRTip = bone('ear_R_tip', earR, toXY(...EAR_JOINTS.R.mid));
  const deform = Object.entries(DEFORMERS).map(([n, d]) => bone(n, root, toXY(...d.c)));
  const bones = [root, head, earL, earR, ...tail, earLTip, earRTip, ...deform];
  return { root, head, earL, earR, earLTip, earRTip, tail, deform, bones, skeleton: new THREE.Skeleton(bones) };
}
