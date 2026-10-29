import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { StateMachine } from '../src/runtime/state-machine.js';
import { mulberry32 } from '../src/runtime/rng.js';
const J = (f) => JSON.parse(readFileSync(new URL(`../src/presets/${f}`, import.meta.url)));
const cfg = J('state-machine.json'), layers = J('layers.json'), camera = J('camera.json');
const mk = () => new StateMachine(cfg, layers, { rng: mulberry32(7), camera });
const cams = (o) => o.filter((c) => c.type === 'camera').map((c) => c.name);

test('camera: no camera config -> no camera commands (backward compatible)', () => {
  assert.deepEqual(cams(new StateMachine(cfg, layers, { rng: mulberry32(7) }).dispatch('success', 0)), []);
});
test('camera: event mapping success->punch_in, notification->zoom_pulse', () => {
  assert.deepEqual(cams(mk().dispatch('success', 0)), ['cam.punch_in']);
  assert.deepEqual(cams(mk().dispatch('notification', 0)), ['cam.zoom_pulse']);
});
test('camera: every mapped clip exists and every rules.cooldowns key is a camera event', () => {
  for (const n of Object.values(camera.events).filter((x) => typeof x === 'string' && x.startsWith('cam.'))) assert.ok(camera.clips[n], n);
  for (const k of Object.keys(camera.rules.cooldowns)) assert.ok(camera.events[k], k);
});
test('camera: per-event cooldown', () => {
  const s = mk(); assert.deepEqual(cams(s.dispatch('notification', 0)), ['cam.zoom_pulse']);
  assert.deepEqual(cams(s.dispatch('notification', 3)), [], 'within 4s cooldown');
  assert.deepEqual(cams(s.dispatch('notification', 8)), ['cam.zoom_pulse']);
});
test('camera: priority - lower tier cannot replace a running higher-tier camera, higher can', () => {
  const s = mk(); assert.deepEqual(cams(s.dispatch('error', 0)), ['cam.punch_in']); // alert 3
  const s2 = mk(); s2.dispatch('success', 0); assert.deepEqual(cams(s2.dispatch('error', 0.1)), ['cam.punch_in'], 'alert replaces info');
});
test('camera: acro combo brings its own camera track and owns the camera for its duration', () => {
  const s = mk(), o = s.dispatch('achievement', 0);
  assert.deepEqual(cams(o), ['cam.pull_back_reveal']); assert.equal(o.find((c) => c.type === 'camera').source, 'acro_show');
  assert.deepEqual(cams(s.dispatch('notification', 0.5)), [], 'same/lower tier event camera dropped during acro');
  assert.deepEqual(cams(s.dispatch('notification', 2.5)), ['cam.zoom_pulse'], 'free after acro ends');
  const c = mk().dispatch('celebrate', 0).filter((x) => x.type === 'camera');
  assert.deepEqual(c.map((x) => x.at), [0, 2.1, 5.3]);
});
test('camera: holdEnd slow_push owned until hover_end -> cam.reset; reset blocked during acro', () => {
  const s = mk(); assert.deepEqual(cams(s.dispatch('hover', 0)), ['cam.slow_push']);
  assert.equal(s.cam.until, Infinity);
  assert.deepEqual(cams(s.end('hover_end', 10)), ['cam.reset']);
  const a = mk(); a.dispatch('achievement', 0); assert.deepEqual(cams(a.end('hover_end', 0.5)), []);
});
test('camera: rejected body event (blocked by drag) gives no camera', () => {
  const s = mk(); s.dispatch('drag_start', 0); assert.deepEqual(cams(s.dispatch('success', 0.5)), []);
});
