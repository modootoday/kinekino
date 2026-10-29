import * as THREE from 'three';
// Small helpers producing glTF-compatible keyframe tracks (scale / quaternion).
export const scaleTrack = (node, times, values) =>
  new THREE.VectorKeyframeTrack(`${node}.scale`, times, values.flatMap((v) => (Array.isArray(v) ? v : [v, v, v])));
export const rotZTrack = (node, times, degs) => {
  const q = new THREE.Quaternion(), out = [];
  degs.forEach((d) => { q.setFromAxisAngle(new THREE.Vector3(0, 0, 1), THREE.MathUtils.degToRad(d)); out.push(q.x, q.y, q.z, q.w); });
  return new THREE.QuaternionKeyframeTrack(`${node}.quaternion`, times, out);
};
export const H = 1e-4;
