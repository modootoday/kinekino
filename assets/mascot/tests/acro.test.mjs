// v8 acrobatic presets: whole 360° turns, rest at both ends, tail does not enter the body, combos/events wired.
import test from 'node:test';
import assert from 'node:assert/strict';
import { parts, layers, clipsByName, poseAt, skinned } from './_mascot.mjs';
import { metric } from '../tools/tail-penetration.mjs';
import { readFileSync } from 'node:fs';
const SM = JSON.parse(readFileSync(new URL('../src/presets/state-machine.json', import.meta.url)));
const acro = Object.keys(parts.clips).filter((n) => n.startsWith('acro.'));
test('acro clip set (spin/roll/flip front+back/cartwheel, in-place + jump, 720)', () => {
  for (const n of ['acro.spin_yaw_360', 'acro.spin_yaw_360_jump', 'acro.spin_yaw_720', 'acro.roll_360', 'acro.flip_front_360', 'acro.flip_front_360_jump', 'acro.flip_back_360', 'acro.flip_back_360_jump', 'acro.cartwheel_360', 'acro.cartwheel_360_jump']) assert.ok(clipsByName.has(n), n);
});
test('rotation amount is a whole multiple of 360° and ends at the rest pose', () => {
  for (const n of acro) {
    const c = parts.clips[n];
    for (const [path, deg] of Object.entries(c.turnsDeg)) { assert.equal(Math.abs(deg % 360), 0, n); const ch = c.channels.find((x) => x.path === path); assert.equal(ch.keys.at(-1)[1], deg, n); assert.equal(ch.keys[0][1], 0, n); }
    const t = clipsByName.get(n).tracks.find((x) => x.name === 'root.quaternion'), q = Array.from(t.values.slice(-4));
    assert.ok(Math.abs(Math.abs(q[3]) - 1) < 1e-3, `${n} end quat ${q}`);
    const P = clipsByName.get(n).tracks.find((x) => x.name === 'root.position'); if (P) { const r = Array.from(P.values.slice(-3)), r0 = Array.from(P.values.slice(0, 3)); r.forEach((v, i) => assert.ok(Math.abs(v - r0[i]) < 1e-3)); }
  }
});
test('acro combos: tail does not enter the body (strict metric)', () => {
  for (const c of ['tail.acro_drag']) for (let k = 1; k < 14; k++) { poseAt(c, k * 0.1); const m = metric(skinned()); assert.equal(m.overlap, 0, `${c}@${k / 10}`); }
});
test('acro combos + events', () => {
  for (const c of ['acro_show', 'acro_roll', 'acro_flip_front', 'acro_flip_back', 'acro_cartwheel', 'acro_combo']) assert.ok(layers.combos[c], c);
  for (const e of ['achievement', 'task_complete', 'celebrate', 'pet_rapid']) assert.ok(layers.combos[SM.events[e].combo], e);
  const look = layers.combos.acro_show.tracks.filter((t) => t[0] === 'look').map((t) => t[1]); assert.deepEqual(look, ['look.release', 'look.track']);
});
