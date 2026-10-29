// Shared test fixture: build the mascot once (same path as tools/export-glb.js).
import '../tools/node-polyfills.js';
import * as THREE from 'three';
import { buildMascot } from '../src/scene/build-mascot.js';
import { loadPresets, loadJson } from '../src/presets/load-presets.js';
export const parts = loadJson('parts.json'), layers = loadJson('layers.json');
export const mascot = buildMascot(loadPresets(), parts, layers);
export const clipsByName = new Map(mascot.animations.map((c) => [c.name, c]));
// Pose the mascot with one clip at time t (fresh mixer), world matrices updated.
export function poseAt(clipName, t) {
  mascot.traverse((o) => { if (o.userData.__rest) { o.position.copy(o.userData.__rest.p); o.quaternion.copy(o.userData.__rest.q); o.scale.copy(o.userData.__rest.s); } });
  const mixer = new THREE.AnimationMixer(mascot);
  if (clipName) { const a = mixer.clipAction(clipsByName.get(clipName)); a.setLoop(THREE.LoopOnce); a.clampWhenFinished = true; a.play(); mixer.update(t); }
  mascot.updateMatrixWorld(true); mixer.stopAllAction(); mixer.uncacheRoot(mascot);
}
mascot.traverse((o) => { o.userData.__rest = { p: o.position.clone(), q: o.quaternion.clone(), s: o.scale.clone() }; });
// Skinned body vertex positions (CPU skinning) in mascot space.
export function skinned() {
  const mesh = mascot.getObjectByName('body'), pos = mesh.geometry.attributes.position, out = new Float32Array(pos.count * 3), v = new THREE.Vector3();
  for (let i = 0; i < pos.count; i++) { v.fromBufferAttribute(pos, i); mesh.applyBoneTransform(i, v); out.set([v.x, v.y, v.z], i * 3); }
  return out;
}
