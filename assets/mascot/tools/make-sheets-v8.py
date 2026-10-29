"""Compose /tmp/kk-frames into shots/v8 (tail before/after, acro with angle labels, camera with zoom labels)."""
import json, os, math
from PIL import Image, ImageDraw
F, O = '/tmp/kk-frames', 'shots/v8'; os.makedirs(O, exist_ok=True)
# Front view (yaw 0) is the main row. ANGLES="15,40" appends one extra column per angle from KEY_yA/ (render-v7.sh ANGLES=...).
ANGLES = [a for a in os.environ.get('ANGLES', '').split(',') if a]
def extra(key, n=0):
    return [(f'yaw {a}°', frames(f'{key}_y{a}')[n]) for a in ANGLES if os.path.isdir(f'{F}/{key}_y{a}')]
P = json.load(open('src/presets/parts.json')); P = P.get('clips', P); L = json.load(open('src/presets/layers.json'))['combos']
def frames(key): return [Image.open(f'{F}/{key}/{f}').convert('RGB') for f in sorted(os.listdir(f'{F}/{key}'), key=lambda s: (len(s), s)) if f.endswith('.png')]
def grid(rows, colnames, rowlabels, out, w=200, lw=170):
    img = Image.new('RGB', (lw + w * len(colnames), 24 + w * len(rows)), (14, 14, 16)); d = ImageDraw.Draw(img)
    for c, n in enumerate(colnames): d.text((lw + c * w + 6, 6), n, fill=(200, 200, 200))
    for r, row in enumerate(rows):
        d.text((6, 24 + r * w + w // 2), rowlabels[r], fill=(235, 235, 235))
        for c, im in enumerate(row):
            img.paste(im.resize((w, w)), (lw + c * w, 24 + r * w))
            if isinstance(colnames[c], str) and len(rows) == 1: pass
    img.save(f'{O}/{out}.png'); return img
# tail before/after: per clip a 2-row sheet (v7 / v8), cols 20/40/60/80 %
T = 'tail.stiff,tail.question,tail.quiver,tail.thump,tail.heart_curve,tail.wrap_body,body.groom,tail.tuck'.split(',')
a, b = frames('multi_tailv7'), frames('multi_tailv8')
for i, n in enumerate(T): grid([a[4*i:4*i+4], b[4*i:4*i+4]], ['20%', '40%', '60%', '80%'], [f'{n} v7(before)', f'{n} v8(after)'], f'tail_before_after_{n.split(".")[1]}')
# acro: angle labels from the root rotation channel of the body clip
def ang(clip, t):
    out = []
    for ch in P[clip]['channels']:
        if ch['target'] == 'root' and ch['path'] in ('rotX', 'rotY', 'rotZ'):
            k = ch['keys']; v = k[-1][1] if t >= k[-1][0] else k[0][1]
            for (t0, a0), (t1, a1) in zip(k, k[1:]):
                if t0 <= t <= t1: u = (t - t0) / (t1 - t0 or 1); u = u * u * (3 - 2 * u) if ch.get('easing', 'easeInOut') != 'linear' else u; v = a0 + (a1 - a0) * u; break
            out.append(f"{ch['path']} {v:+.0f}°")
    return ' '.join(out) or '0°'
def bodyclip(c):
    for tr in L[c]['tracks']:
        if tr[0] == 'body': return tr[1], tr[2]
AT = [0, 0.35, 0.5, 0.65, 0.8, 0.95, 1.15, 1.9]
for c in ['acro_show', 'acro_roll', 'acro_flip_front', 'acro_flip_back', 'acro_cartwheel']:
    bc, at = bodyclip(c); grid([frames(f'combo_{c}')], [f't={t}s {ang(bc, t - at)}' if i % 2 == 0 else f'{ang(bc, t - at)}' for i, t in enumerate(AT)], [f'{c}\n{bc}'], c, w=220)
    # single-row headers can overlap: redraw with two-line headers
    fr = frames(f'combo_{c}'); ex = extra(f'combo_{c}'); w = 220; img = Image.new('RGB', (170 + w * (8 + len(ex)), 44 + w), (14, 14, 16)); d = ImageDraw.Draw(img)
    for j, (lab, im) in enumerate(ex): x = 170 + (8 + j) * w; img.paste(im.resize((w, w)), (x, 44)); d.text((x + 6, 4), f't={AT[0]}s', fill=(200, 200, 200)); d.text((x + 6, 20), lab, fill=(120, 200, 255))
    for i, (t, im) in enumerate(zip(AT, fr)): img.paste(im.resize((w, w)), (170 + i * w, 44)); d.text((170 + i * w + 6, 4), f't={t}s', fill=(200, 200, 200)); d.text((170 + i * w + 6, 20), ang(bc, t - at), fill=(255, 190, 120))
    d.text((6, 44 + w // 2), f'{c}\n{bc}', fill=(235, 235, 235)); img.save(f'{O}/{c}.png')
CT = [0, 0.7, 1.2, 2.8, 3.3, 4.9, 5.4, 6.4]; fr = frames('combo_acro_combo'); ex = extra('combo_acro_combo'); w = 220
img = Image.new('RGB', (170 + w * (8 + len(ex)), 44 + w), (14, 14, 16)); d = ImageDraw.Draw(img)
for j, (lab, im) in enumerate(ex): x = 170 + (8 + j) * w; img.paste(im.resize((w, w)), (x, 44)); d.text((x + 6, 4), f't={CT[0]}s', fill=(200, 200, 200)); d.text((x + 6, 20), lab, fill=(120, 200, 255))
segs = [(0, 'acro_show'), (2.1, 'acro_flip_back'), (4.2, 'acro_cartwheel')]
for i, (t, im) in enumerate(zip(CT, fr)):
    s0, sc = [s for s in segs if s[0] <= t][-1]; bc, at = bodyclip(sc)
    img.paste(im.resize((w, w)), (170 + i * w, 44)); d.text((170 + i * w + 6, 4), f't={t}s {sc}', fill=(200, 200, 200)); d.text((170 + i * w + 6, 20), ang(bc, t - s0 - at), fill=(255, 190, 120))
d.text((6, 44 + w // 2), 'acro_combo', fill=(235, 235, 235)); img.save(f'{O}/acro_combo.png')
CL = json.load(open('/tmp/kk8/camlab.json'))
for n, labs in CL.items():
    fr = frames(f'cam_cam.{n}'); w = 300; img = Image.new('RGB', (170 + w * 3, 30 + w), (14, 14, 16)); d = ImageDraw.Draw(img)
    for i, (lab, im) in enumerate(zip(labs, fr)): img.paste(im.resize((w, w)), (170 + i * w, 30)); d.text((170 + i * w + 6, 8), ['before ', 'mid ', 'after '][i] + lab, fill=(200, 200, 200))
    d.text((6, 30 + w // 2), f'cam.{n}', fill=(235, 235, 235)); img.save(f'{O}/cam_{n}.png')
print(sorted(os.listdir(O)))
