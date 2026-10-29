import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { expandCombo } from '../src/runtime/combo-schedule.js';
import { mulberry32 } from '../src/runtime/rng.js';
import { resolveClip } from '../src/presets/build-part-clips.js';
const J = (f) => JSON.parse(readFileSync(new URL(`../src/presets/${f}`, import.meta.url)));
const L = J('layers.json'), P = J('parts.json');
test('68 combos (10 v6 + 52 v7 + 6 v8 acro)', () => assert.equal(Object.keys(L.combos).length, 68));
test('every combo expands; each event clip exists and belongs to its track layer', () => {
  for (const n of Object.keys(L.combos)) for (const seed of [1, 2]) {
    for (const e of expandCombo(L, n, { rng: mulberry32(seed) })) {
      if (e.stopAll) continue;
      if (e.look) { assert.ok(L.looks[e.look], `${n}: look ${e.look}`); continue; }
      assert.ok(P.clips[e.clip], `${n}: clip ${e.clip}`);
      assert.equal(resolveClip(P, e.clip).layer, e.layer, `${n}: ${e.clip} layer`);
      assert.ok(L.layers[e.layer], `${n}: layer ${e.layer}`);
      assert.ok(e.at <= L.combos[n].duration + 20, n);
    }
  }
});
test('vars, bpm and random side are deterministic per seed', () => {
  const g = expandCombo(L, 'greet', { vars: { entry: 'fx.pop_in' } }); assert.equal(g[0].clip, 'fx.pop_in');
  assert.equal(expandCombo(L, 'greet')[0].clip, 'fx.peek');
  const m = expandCombo(L, 'music_groove', { bpm: 120 }); assert.ok(m.some((e) => e.opts?.timeScale === 1.2));
  assert.deepEqual(expandCombo(L, 'idle_twitch', { rng: mulberry32(9) }), expandCombo(L, 'idle_twitch', { rng: mulberry32(9) }));
});
test('no excluded items: no heart/star eyes, no split_merge', () => {
  const s = JSON.stringify([L, P]); for (const bad of ['heart_eye', 'star_eye', 'eye_L.heart', 'eye_L.star', 'fx.split_merge']) assert.ok(!s.includes(bad), bad);
});
test('mirror clips: every _R side clip is a mirror of its _L definition', () => {
  for (const [n, c] of Object.entries(P.clips)) if (/_R\./.test(n) && P.clips[n.replace('_R.', '_L.')] && c.batch) assert.ok(c.mirror, n);
});
