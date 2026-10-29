import * as THREE from 'three';
import { createBodyGeometry } from '../geometry/body.js';
import { createEyeGeometry, eyeHalfExtents, EYE_MORPHS } from '../geometry/eye.js';
import { buildPartClips } from '../presets/build-part-clips.js';
import { toXY, UNIT } from '../geometry/svg-space.js';
import { pillowZ } from '../geometry/dome.js';
import { createSkeleton } from '../rig/skeleton.js';
import { applyTailSkin } from '../rig/skinning.js';
import { createMaterials } from './materials.js';
import { createClips } from '../animation/clips.js';
import { buildPresetClips } from '../presets/build-preset-clips.js';
// Assembles the rigged mascot. Rest pose = static logo: decorations and eyes at scale 0.
const HIDDEN = 1e-4;
// The skeleton is a sibling of the body mesh (not its child): glTF loaders apply a node's morph-weights track to every
// morph mesh in its subtree, so bones under the body would make the body's reveal weight drive the eye morphs.
function skinned(name, geo, material, rig, parent) {
  const m = new THREE.SkinnedMesh(applyTailSkin(geo), material); m.name = name;
  parent.add(rig.root); parent.add(m); parent.updateMatrixWorld(true); m.bind(rig.skeleton); return m;
}
// Eye plane height: max pillow height over the footprint the eye can reach (half extents x maxScale + look range + surprise lift), so a
// gazing/grown eye never sinks into the curved front. Costs a small constant lift at rest.
function eyeZ(ex, ey, E, L) {
  const [hx, hy] = eyeHalfExtents(E.shape), rx = (hx * E.maxScale + L.eyeRange[0]) * UNIT, ry = (hy * E.maxScale + L.eyeRange[1] + 1.5) * UNIT; let z = -Infinity;
  for (let i = 0; i < 16; i++) { const a = (i / 16) * Math.PI * 2; z = Math.max(z, pillowZ(ex + rx * Math.cos(a), ey + ry * Math.sin(a))); }
  return Math.max(z, pillowZ(ex, ey));
}
export function buildMascot(presets, parts, layers) {
  const mats = createMaterials(); const rig = createSkeleton();
  const group = new THREE.Group(); group.name = 'kinekino';
  const body = skinned('body', createBodyGeometry(), mats.body, rig, group);
  // Colour reveal: one morph target on the body carries the COLOR_0 delta of jaw band / tail strip / inner ears
  // (position delta 0). Weight 0 = solid logo colour, 1 = revealed. The colour delta is written into the glb by
  // tools/export-glb.js (GLTFExporter only writes POSITION/NORMAL morphs).
  // v7: second colour morph 'blush' (cheeks). Both weights belong to the reveal layer.
  body.userData.colorMorph = body.geometry.morphAttributes.color.map((a) => a.array); delete body.geometry.morphAttributes.color;
  body.morphTargetDictionary = { reveal: 0, blush: 1 }; body.morphTargetInfluences = [0, 0];
  const eyes = new THREE.Group(); eyes.name = 'eyes'; rig.head.add(eyes);
  // eye_X_look (owned by the look layer: translation) -> eye_X mesh (owned by the eye layer: scale/rotation/delta/morphs).
  const E = layers.eyes, hw = rig.head.userData.world;
  for (const [n, sx] of [['eye_L', 60 - E.gap / 2], ['eye_R', 60 + E.gap / 2]]) {
    const [ex, ey] = toXY(sx, E.centerY);
    const look = new THREE.Object3D(); look.name = `${n}_look`; look.position.set(ex - hw[0], ey - hw[1], eyeZ(ex, ey, E, layers.look)); eyes.add(look);
    const e = new THREE.Mesh(createEyeGeometry(E.shape, n === 'eye_L' ? 'L' : 'R'), mats.ink); e.name = n;
    e.morphTargetDictionary = Object.fromEntries(EYE_MORPHS.map((m, i) => [m, i])); e.morphTargetInfluences = EYE_MORPHS.map(() => 0);
    look.add(e);
  }
  for (const n of [eyes]) n.scale.setScalar(HIDDEN);
  group.updateMatrixWorld(true);
  const rest = (name) => group.getObjectByName(name).position.toArray();
  group.animations = [...createClips(), ...(presets ? buildPresetClips(presets) : []), ...(parts ? buildPartClips(parts, rest) : [])];
  return group;
}
