"""v8: acrobatic 360-degree part clips + combos + events (idempotent: rewrites the acro entries)."""
import json
P = 'src/presets/parts.json'; L = 'src/presets/layers.json'; S = 'src/presets/state-machine.json'
p, l, s = (json.load(open(f)) for f in (P, L, S))
E = 'easeInOut'
JUMP_D = 1.4
def jump_tracks():  # crouch - takeoff stretch - apex - descend - land squash - settle (SVG units)
    return [{"target": "root", "path": "scale3", "easing": E, "keys": [[0, [1, 1, 1]], [0.15, [1.12, 0.84, 1.12]], [0.3, [0.9, 1.15, 0.9]], [0.65, [1, 1, 1]], [1.0, [0.94, 1.08, 0.94]], [1.1, [1.18, 0.8, 1.18]], [1.25, [0.97, 1.05, 0.97]], [1.4, [1, 1, 1]]]},
            {"target": "root", "path": "trans", "easing": E, "keys": [[0, [0, 0, 0]], [0.15, [0, -3, 0]], [0.3, [0, 6, 0]], [0.65, [0, 34, 0]], [1.0, [0, 6, 0]], [1.1, [0, -4, 0]], [1.25, [0, 1, 0]], [1.4, [0, 0, 0]]]}]
def rot(path, deg, t0, t1, d):
    k = [[0, 0]] + ([[t0, 0]] if t0 > 0 else []) + [[t1, deg]] + ([[d, deg]] if t1 < d else [])
    return {"target": "root", "path": path, "easing": E, "keys": k}
def ground(d):  # in-place: small anticipation squash before and settle after
    return [{"target": "root", "path": "scale3", "easing": E, "keys": [[0, [1, 1, 1]], [0.12, [1.08, 0.9, 1.08]], [0.25, [0.97, 1.04, 0.97]], [d - 0.2, [1, 1, 1]], [d - 0.1, [1.06, 0.93, 1.06]], [d, [1, 1, 1]]]}]
AX = {"spin_yaw": ("rotY", 1), "flip_front": ("rotX", 1), "flip_back": ("rotX", -1), "cartwheel": ("rotZ", -1)}
DOC = {"spin_yaw": "세로축 팽이 회전", "flip_front": "앞 공중제비(가로축 pitch)", "flip_back": "뒤 공중제비", "cartwheel": "옆돌기(정면 축 roll)", "roll": "옆으로 굴러 한 바퀴(이동 후 미끄러져 복귀)"}
acro = {}
for name, (path, sgn) in AX.items():
    for turns in ([1, 2] if name == 'spin_yaw' else [1]):
        deg = 360 * turns
        acro[f"acro.{name}_{deg}"] = {"layer": "body", "duration": 1.0 * turns, "loop": False, "channels": [rot(path, sgn * deg, 0.15, 1.0 * turns - 0.15, 1.0 * turns), *ground(1.0 * turns)], "doc": f"{DOC[name]} {deg}° 제자리", "turnsDeg": {path: sgn * deg}, "batch": 8}
        acro[f"acro.{name}_{deg}_jump"] = {"layer": "body", "duration": JUMP_D, "loop": False, "channels": [rot(path, sgn * deg, 0.3, 1.05, JUMP_D), *jump_tracks()], "doc": f"{DOC[name]} {deg}° 점프(도약-정점-착지 squash&stretch)", "turnsDeg": {path: sgn * deg}, "landAt": 1.1, "batch": 8}
acro["acro.roll_360"] = {"layer": "body", "duration": 1.8, "loop": False, "channels": [rot("rotZ", -360, 0.1, 0.9, 1.8),
    {"target": "root", "path": "trans", "easing": E, "keys": [[0, [0, 0, 0]], [0.1, [0, 0, 0]], [0.9, [24, 0, 0]], [1.2, [24, 0, 0]], [1.8, [0, 0, 0]]]}, *ground(1.8)],
    "doc": DOC["roll"], "turnsDeg": {"rotZ": -360}, "landAt": 0.9, "batch": 8}
# Drag overlap (centrifugal lag): tail swings DOWN/away (positive rotZ: never toward the body), ears flare out, then damped settle.
acro["tail.acro_drag"] = {"layer": "tail", "duration": JUMP_D, "loop": False, "channels": [
    {"target": f"tail_{i}", "path": "rotZ", "easing": E, "keys": [[0, 0], [0.35, 4 + 4 * i], [1.0, 6 + 5 * i], [1.15, -2 - i], [1.3, 1 + 0.5 * i], [1.4, 0]]} for i in range(4)], "doc": "회전 원심력 끌림 + 착지 반동", "batch": 8}
acro["ear_L.acro_drag"] = {"layer": "ear_L", "duration": JUMP_D, "loop": False, "channels": [
    {"target": "ear_L", "path": "rotZ", "easing": E, "keys": [[0, 0], [0.35, 8], [1.0, 10], [1.15, -6], [1.3, 2], [1.4, 0]]},
    {"target": "ear_L_tip", "path": "rotZ", "easing": E, "keys": [[0, 0], [0.4, 12], [1.05, 14], [1.2, -8], [1.32, 3], [1.4, 0]]}], "doc": "회전 원심력으로 바깥으로 젖힘 + 착지 반동", "batch": 8}
acro["ear_R.acro_drag"] = {"mirror": "ear_L.acro_drag", "batch": 8}
for k in [k for k in p['clips'] if k.startswith('acro.') or k.endswith('.acro_drag')]: del p['clips'][k]
p['clips'].update(acro)
l['looks']['look.release'] = {"mode": "fixed", "target": [0, 0], "omega": 30, "doc": "v8: 회전 중 look-at 해제(정면 고정); 착지 후 look.track 으로 복귀"}
def show(body, d=2.0, land=1.1, ts=1.0):
    return {"duration": d, "tracks": [["look", "look.release", 0], ["body", body, 0], ["tail", "tail.acro_drag", 0, {"timeScale": ts}],
        ["ear_L", "ear_L.acro_drag", 0, {"timeScale": ts}], ["ear_R", "ear_R.acro_drag", 0, {"timeScale": ts}],
        ["shape", "body.wobble_hit", land], ["eye_L", "eye_L.blink", land + 0.05], ["eye_R", "eye_R.blink", land + 0.05], ["look", "look.track", land + 0.35]],
        "group": "곡예", "meaning": body, "batch": 8, "blend": 0.15}
combos = {"acro_show": show("acro.spin_yaw_360_jump"), "acro_roll": show("acro.roll_360", 2.3, 0.9, 1.4 / 1.8),
    "acro_flip_front": show("acro.flip_front_360_jump"), "acro_flip_back": show("acro.flip_back_360_jump"), "acro_cartwheel": show("acro.cartwheel_360_jump"),
    "acro_combo": {"duration": 6.4, "tracks": [["combo", "acro_show", 0], ["combo", "acro_flip_back", 2.1], ["combo", "acro_cartwheel", 4.2]], "group": "곡예", "meaning": "연속 기술", "batch": 8, "blend": 0.15}}
for n, c in combos.items(): c['meaning'] = {"acro_show": "팽이 점프", "acro_roll": "옆구르기", "acro_flip_front": "앞 공중제비", "acro_flip_back": "뒤 공중제비", "acro_cartwheel": "옆돌기", "acro_combo": "연속 기술"}[n]
l['combos'].update(combos)
ev = {"achievement": {"combo": "acro_show", "tier": "info", "note": "v8: 업적/마일스톤"},
      "task_complete": {"combo": "acro_flip_front", "tier": "info", "note": "v8: 긴 작업 완료"},
      "celebrate": {"combo": "acro_combo", "tier": "info", "note": "v8: 큰 축하(연속 기술)"},
      "pet_rapid": {"combo": "acro_roll", "tier": "direct", "internal": True, "note": "v8: 쓰다듬기 연타(stroke 3회/2s) -> 옆구르기"}}
s['events'].update(ev)
s['events']['success'].setdefault('streak', {})['bigStreak'] = {"count": 6, "combo": "acro_cartwheel", "note": "v8"}
s['events']['stroke']['rapid'] = {"count": 3, "within": 2, "event": "pet_rapid", "note": "v8"}
for f, o in ((P, p), (L, l), (S, s)): json.dump(o, open(f, 'w'), indent=1, ensure_ascii=False); open(f, 'a').write('\n')
print(len(p['clips']), len(l['combos']), len(s['events']))
