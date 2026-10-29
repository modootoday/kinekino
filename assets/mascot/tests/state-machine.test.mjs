import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { StateMachine, comboLayers } from '../src/runtime/state-machine.js';
import { mulberry32 } from '../src/runtime/rng.js';
const J = (f) => JSON.parse(readFileSync(new URL(`../src/presets/${f}`, import.meta.url)));
const cfg = J('state-machine.json'), layers = J('layers.json');
const mk = (o = {}) => new StateMachine(cfg, layers, { rng: mulberry32(o.seed ?? 7), ...o });
const names = (cmds, type = 'combo') => cmds.filter((c) => c.type === type).map((c) => c.name);

test('31 events (27 page + 4 v8 acro), every mapped combo exists', () => {
  assert.equal(Object.values(cfg.events).filter((e) => !e.helper).length, 31);
  for (const [ev, e] of Object.entries(cfg.events)) for (const k of ['combo', 'fatalCombo', 'double']) if (e[k]) assert.ok(layers.combos[e[k]], `${ev}.${k}=${e[k]}`);
});
test('priority: higher tier interrupts lower, lower is rejected on shared layers', () => {
  const sm = mk();
  assert.deepEqual(names(sm.dispatch('drag_start', 0)), ['drag_hold']);
  assert.deepEqual(names(sm.dispatch('click', 0.5)), [], 'direct (4) must not interrupt drag (5)');
  assert.deepEqual(names(sm.end('drop', 1)), []);
  assert.deepEqual(names(sm.dispatch('drop', 1)), ['drop_land']);
  const sm2 = mk(); sm2.dispatch('success', 0);
  assert.deepEqual(names(sm2.dispatch('error', 0.2)), ['error'], 'alert (3) interrupts info (2)');
  assert.deepEqual(names(sm2.dispatch('success', 0.4)), [], 'info blocked while error owns its layers');
});
test('same tier: newest wins', () => { const sm = mk(); sm.dispatch('error', 0); assert.deepEqual(names(sm.dispatch('warning', 0.3)), ['warning']); });
test('layer-level interrupt: notification takes ear/reveal/look, music body keeps playing', () => {
  const sm = mk(); const m = sm.dispatch('music_start', 0, { bpm: 120 }); assert.equal(m[0].bpm, 120);
  const n = sm.dispatch('notification', 1, { target: [0.8, 0.9] })[0];
  assert.equal(n.name, 'notification'); assert.deepEqual(n.look, [0.8, 0.9]);
  assert.ok(!n.layers.includes('body') && n.layers.includes('ear_L') && n.layers.includes('look'));
  assert.equal(sm.owners.get('body').combo, 'music_groove'); assert.equal(sm.owners.get('ear_L').combo, 'notification');
});
test('blend: 0.05 for startle-type, 0.2 default', () => {
  const sm = mk(); assert.equal(sm.dispatch('cursor_fast', 0)[0].blend, 0.05); assert.equal(sm.dispatch('drop', 1)[0].blend, 0.2);
});
test('loading: <300ms skipped, >=5s escalates to loading_long', () => {
  const a = mk(); a.dispatch('load_start', 0); assert.deepEqual(names(a.dispatch('load_end', 0.2)), []); assert.deepEqual(names(a.tick(1)), []);
  const b = mk(); b.dispatch('load_start', 0); assert.deepEqual(names(b.tick(0.31)), ['loading']); assert.deepEqual(names(b.tick(5)), ['loading_long']);
  assert.deepEqual(names(b.dispatch('load_end', 6), 'stop'), ['loading_long']);
});
test('success streak 3 -> big_success; error resets streak; fatal error -> scared', () => {
  const sm = mk(); assert.deepEqual([0, 3, 6].flatMap((t) => names(sm.dispatch('success', t))), ['success', 'success', 'big_success']);
  sm.dispatch('success', 12); sm.dispatch('error', 15); assert.deepEqual(names(sm.dispatch('success', 20)), ['success']);
  assert.deepEqual(names(mk().dispatch('error', 0, { fatal: true })), ['scared']);
});
test('app_start: first run pops in, later peeks; typing releases 2s after last key', () => {
  assert.deepEqual(mk({ firstRun: true }).dispatch('app_start', 0)[0].vars, { entry: 'fx.pop_in' });
  assert.deepEqual(mk().dispatch('app_start', 0)[0].vars, { entry: 'fx.peek' });
  const sm = mk(); const a = sm.dispatch('typing', 0); assert.deepEqual(names(a), ['typing_buddy']); assert.deepEqual(names(a, 'trigger'), ['input']);
  sm.dispatch('typing', 1); assert.deepEqual(names(sm.tick(2.5), 'stop'), []); assert.deepEqual(names(sm.tick(3.01), 'stop'), ['typing_buddy']);
});
test('clicks: double, rapid, overstim -> dizzy', () => {
  const sm = mk(); assert.deepEqual(names(sm.dispatch('click', 0)), ['click_poke']); assert.deepEqual(names(sm.dispatch('click', 0.2)), ['double_click']);
  assert.deepEqual(names(sm.dispatch('click', 0.6)), ['rapid_click']); sm.dispatch('click', 0.9); assert.deepEqual(names(sm.dispatch('click', 1.2)), ['dizzy']);
});
test('stroke overstim: pet_purr -> tail.tip_flick at 8s -> annoyed', () => {
  const sm = mk(); assert.deepEqual(names(sm.dispatch('stroke', 0)), ['stroke']);
  const c = sm.tick(8); assert.equal(c[0].clip, 'tail.tip_flick'); assert.deepEqual(names(sm.tick(9)), ['annoyed']);
  const s2 = mk(); s2.dispatch('stroke', 0); s2.end('stroke_end', 4); assert.deepEqual(s2.tick(10).filter((x) => x.type !== 'combo' || x.tier > 1), []);
});
test('offline -> online plays a weak success; hunting only in play mode', () => {
  const sm = mk(); sm.dispatch('offline', 0); const r = sm.end('online', 3); assert.deepEqual(names(r), ['success']); assert.equal(r.find((x) => x.type === 'combo').weight, 0.5);
  assert.deepEqual(names(mk().dispatch('cursor_slow', 0)), []);
  const p = mk({ playMode: true }); assert.deepEqual(names(p.dispatch('cursor_slow', 0)), ['hunting_stalk']); assert.deepEqual(names(p.tick(3)), ['pounce']);
});
function runIdle(seed, until = 170) { const sm = mk({ seed }), seq = []; for (let t = 0; t <= until; t += 0.5) for (const c of sm.tick(t)) if (c.type === 'combo') seq.push([t, c.name]); return seq; }
test('idle: nothing before 30s, variants after, idle_deep at 180s, wake_up only when asleep', () => {
  const seq = runIdle(3); assert.ok(seq.length > 3); assert.ok(seq[0][0] >= 30);
  const sm = mk(); let deep = []; for (let t = 0; t <= 181; t += 0.5) deep.push(...names(sm.tick(t))); assert.ok(deep.includes('idle_deep'));
  assert.deepEqual(names(sm.dispatch('click', 185)).slice(0, 1), ['click_poke'], 'not asleep yet at +5s -> no wake_up');
  const s2 = mk(); for (let t = 0; t <= 200; t += 0.5) s2.tick(t); assert.ok(s2.idle.asleep);
  assert.deepEqual(names(s2.dispatch('click', 201)), ['wake_up']); // wake_up (direct) owns the layers; the click is absorbed
});
test('idle cooldown: same variant >= 20s apart, big variants >= 60s apart, no repeat of last 2', () => {
  for (const seed of [1, 2, 3, 4, 5, 42]) {
    const seq = runIdle(seed, 175), last = {}; let lastBig = -Infinity;
    seq.forEach(([t, n], i) => {
      if (last[n] !== undefined) assert.ok(t - last[n] >= 20, `seed ${seed}: ${n} ${t - last[n]}s`); last[n] = t;
      if (cfg.idle.cooldown.bigSet.includes(n)) { assert.ok(t - lastBig >= 60, `seed ${seed}: big ${n}`); lastBig = t; }
      assert.ok(!seq.slice(Math.max(0, i - 2), i).some(([, m]) => m === n), `seed ${seed}: repeat ${n}`);
    });
  }
});
test('seed injection: same seed -> identical sequence, different seed -> different', () => {
  assert.deepEqual(runIdle(11), runIdle(11)); assert.notDeepEqual(runIdle(11), runIdle(12));
});
test('comboLayers expands nested combos and random ears', () => {
  const l = comboLayers(layers, 'idle_deep'); for (const x of ['shape', 'eye_L', 'tail', 'body']) assert.ok(l.has(x));
  assert.ok(comboLayers(layers, 'idle_twitch').has('ear_R'));
});
test('v8 acro events: achievement/task_complete/celebrate, stroke rapid -> acro_roll, 6-success streak -> acro_cartwheel', () => {
  assert.deepEqual(names(mk().dispatch('achievement', 0)), ['acro_show']);
  assert.deepEqual(names(mk().dispatch('task_complete', 0)), ['acro_flip_front']);
  assert.deepEqual(names(mk().dispatch('celebrate', 0)), ['acro_combo']);
  const s = mk(); s.dispatch('stroke', 0); s.end?.('stroke_end', 0.3); s.dispatch('stroke', 0.6); s.end?.('stroke_end', 0.9);
  assert.ok(names(s.dispatch('stroke', 1.2)).includes('acro_roll'));
  const g = mk(); let last = []; for (let i = 0; i < 6; i++) last = g.dispatch('success', i * 4); assert.ok(names(last).includes('acro_cartwheel'), JSON.stringify(names(last)));
});
