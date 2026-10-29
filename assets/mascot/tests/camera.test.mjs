// v8 camera layer: framing returns to default, holdEnd clips are explicit, dolly keeps subject size, settings drive values.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolveCamClip, cameraAt, apparentScale, defaultView } from '../src/runtime/camera.js';
const J = (f) => JSON.parse(readFileSync(new URL(`../src/presets/${f}`, import.meta.url)));
const C = J('camera.json'), L = J('layers.json'), SM = J('state-machine.json');
const names = Object.keys(C.clips), near = (a, b, e = 1e-6) => Math.abs(a - b) < e;
test('10+ camera clips incl. required set', () => {
  for (const n of ['cam.zoom_in_slow', 'cam.zoom_in_fast', 'cam.zoom_out', 'cam.punch_in', 'cam.dolly_zoom', 'cam.zoom_pulse', 'cam.focus_eyes', 'cam.pull_back_reveal', 'cam.slow_push', 'cam.orbit_zoom']) assert.ok(C.clips[n], n);
});
test('camera layer exists, runtime-only, owns no mascot node channel', () => {
  const cam = L.layers.camera; assert.ok(cam?.runtimeOnly);
  for (const ch of cam.channels) { assert.ok(ch.startsWith('camera.'), ch); for (const [n, l] of Object.entries(L.layers)) if (n !== 'camera') assert.ok(!l.channels.includes(ch)); }
});
test('start/end at default framing unless holdEnd or restExempt', () => {
  for (const n of names) {
    const c = resolveCamClip(C, n), d = defaultView(c.S), cur = { zoom: 1.6 };
    const a = cameraAt(c, 0, cur), b = cameraAt(c, c.duration, cur);
    const isDef = (v) => near(v.distance, d.distance, 1e-3) && near(v.fovDeg, d.fovDeg, 1e-6) && near(v.yawDeg, 0, 1e-6) && near(v.focus[1], 60, 1e-6);
    if (c.restExempt !== 'entry' && !C.clips[n].fromCurrent) assert.ok(isDef(a), `${n} start`);
    if (!c.holdEnd) assert.ok(isDef(b), `${n} end ${JSON.stringify(b)}`); else assert.ok(!isDef(b), `${n} holdEnd must actually hold`);
  }
});
test('dolly zoom keeps apparent subject size while FOV changes', () => {
  const c = resolveCamClip(C, 'cam.dolly_zoom');
  for (let i = 0; i <= 10; i++) { const v = cameraAt(c, (c.duration * i) / 10); assert.ok(near(apparentScale(v, c.S), 1, 1e-9)); }
  assert.ok(near(cameraAt(c, c.duration / 2).fovDeg, C.settings.dollyFovDeg, 1e-6));
});
test('settings are the source of the zoom range and timing', () => {
  const a = resolveCamClip(C, 'cam.zoom_in_slow'), b = resolveCamClip(C, 'cam.zoom_in_slow', { zoomInScale: 2, zoomInSlowSec: 1 });
  assert.ok(near(apparentScale(cameraAt(a, a.duration), a.S), C.settings.zoomInScale, 1e-9));
  assert.ok(near(apparentScale(cameraAt(b, 1), b.S), 2, 1e-9)); assert.equal(b.duration, 2);
});
test('camera combos/events reference existing clips, combos and events', () => {
  for (const [combo, tr] of Object.entries(C.combos)) { assert.ok(L.combos[combo], combo); for (const [clip] of tr) assert.ok(C.clips[clip], clip); }
  for (const [ev, clip] of Object.entries(C.events)) if (ev !== '_doc') { assert.ok(C.clips[clip], clip); assert.ok(SM.events[ev] || ev.endsWith('_end'), ev); }
});
