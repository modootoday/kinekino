import * as THREE from 'three';
import { PALETTE } from '../geometry/palette.js';
// Slime/jelly look: soft broad highlights (high roughness, low specular), sheen as a cheap SSS/rim approximation,
// faint self-glow of the base colour so shadowed sides stay warm instead of going grey.
export const mat = (name, color, { roughness = 0.5, sheen = 0.8, glow = 0.12, vertexColors = false } = {}) => {
  const c = new THREE.Color(color);
  return Object.assign(new THREE.MeshPhysicalMaterial({
    color: vertexColors ? new THREE.Color(1, 1, 1) : c, vertexColors, roughness, metalness: 0, specularIntensity: 0.35,
    sheen, sheenRoughness: 0.6, sheenColor: c.clone().lerp(new THREE.Color(0xffd2b8), 0.6),
    clearcoat: 0.25, clearcoatRoughness: 0.55, emissive: c, emissiveIntensity: glow,
  }), { name });
};
// Surface variants (all core glTF PBR + KHR_materials_{sheen,specular,clearcoat,transmission,volume,ior,emissive_strength}).
//  v5:   current (clearcoat 0.25).
//  soft: matte jelly - no clearcoat, higher roughness, broad sheen rim.
//  jelly: soft + faint subsurface look (small transmission, volume thickness + warm attenuation, ior 1.33).
//  bake: soft + baked vertex-colour AO toward the rim (geometry side, see body.js BODY_AO).
export const SURFACE = globalThis.process?.env?.SURFACE ?? 'bake';
const SOFT = { roughness: 0.68, specularIntensity: 0.18, sheen: 1, sheenRoughness: 0.9, clearcoat: 0 };
function bodyMat() {
  const m = mat('body', PALETTE.body, { vertexColors: true, glow: 0.06 });
  if (SURFACE === 'v5') return m;
  Object.assign(m, SOFT);
  m.sheenColor = new THREE.Color(PALETTE.body).lerp(new THREE.Color(0xffe0cc), 0.75);
  if (SURFACE === 'jelly') Object.assign(m, { transmission: 0.12, thickness: 0.6, ior: 1.33,
    attenuationColor: new THREE.Color(0xd8401a), attenuationDistance: 0.8 });
  return m;
}
export const createMaterials = () => ({
  // Body colour comes from COLOR_0 (logo colours painted per region); glow kept low so it does not tint the band.
  body: bodyMat(),
  ink: mat('ink', PALETTE.ink, { roughness: 0.35, sheen: 0, glow: 0, vertexColors: true }), // v7: COLOR_0 = ink + white sparkle highlight
});
