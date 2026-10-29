import * as THREE from 'three';
import { scaleTrack, rotZTrack, H } from './tracks.js';
// idle: static logo. reveal: decorations + eyes appear. blink. tail_wave. sequence: full loop back to static.
// Colour reveal of the body (jaw band, tail strip, inner ears): morph weight 0..1 (overshoot clamped).
const decor = (times, vals) => [new THREE.NumberKeyframeTrack('body.morphTargetInfluences', times, vals.flatMap((v) => [v <= H ? 0 : Math.min(v, 1), 0]))]; // [reveal, blush]
const eyes = (times, sy) => scaleTrack('eyes', times, sy.map((y) => (y === 0 ? [H, H, H] : [1, y, 1])));
const TAIL_AMP = [0.4, 0.6, 0.8, 1];
const tail = (times, degs) => TAIL_AMP.map((a, i) => rotZTrack(`tail_${i}`, times, degs.map((d) => d * a)));
export function createClips() {
  const idle = new THREE.AnimationClip('idle', 2, [...decor([0, 2], [H, H]), eyes([0, 2], [0, 0]), ...tail([0, 2], [0, 0])]);
  const reveal = new THREE.AnimationClip('reveal', 1.2, [
    ...decor([0, 0.5, 0.8], [H, 1.08, 1]), eyes([0, 0.1, 0.4, 1.2], [0, 0.12, 1, 1])]);
  const blink = new THREE.AnimationClip('blink', 0.6, [...decor([0, 0.6], [1, 1]), eyes([0, 0.2, 0.25, 0.5, 0.6], [1, 1, 0.08, 1, 1])]);
  const tw = [0, 0.4, 0.8, 1.2, 1.6, 2.0], twD = [0, -9, 7, -5, 3, 0];
  const tailWave = new THREE.AnimationClip('tail_wave', 2.0, [...decor([0, 2], [1, 1]), eyes([0, 2], [1, 1]),
    ...tail(tw, twD)]);
  const S = 7;
  // Sequence: reveal -> ear perk -> blink -> tail wave -> ear flutter -> back to static logo.
  const perkT = [0, 1.6, 1.7, 1.85, 2.0, 2.15, S], perk = [0, 0, -15, 4, -2, 0, 0];
  const flutT = [0, 5.4, 5.45, 5.5, 5.55, 5.6, 5.65, 5.7, 5.8, S], flut = [0, 0, 6, -5, 4, -3, 2, -1, 0, 0];
  const sequence = new THREE.AnimationClip('sequence', S, [
    ...decor([0, 0.9, 1.3, 1.5, 6.3, 6.8, S], [H, H, 1.08, 1, 1, H, H]),
    eyes([0, 1.0, 1.2, 1.5, 2.8, 2.86, 3.1, 6.3, 6.7, S], [0, 0, 0.12, 1, 1, 0.08, 1, 1, 0, 0]),
    rotZTrack('ear_L', perkT, perk), rotZTrack('ear_R', perkT, perk.map((d) => -d)),
    rotZTrack('ear_L_tip', flutT, flut), rotZTrack('ear_R_tip', flutT.map((t, i) => (i > 1 && i < flutT.length - 1 ? t + 0.04 : t)), flut.map((d) => -d)),
    ...tail([0, 3.3, 3.7, 4.1, 4.5, 4.9, 5.3, S], [0, 0, -9, 7, -5, 3, 0, 0])]);
  return [idle, reveal, blink, tailWave, sequence];
}
