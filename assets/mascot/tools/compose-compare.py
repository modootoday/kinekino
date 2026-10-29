"""Builds shots/compare_v2_v3.png (rows: v2 vs v3 at front/20°/40°) and shots/physics/physics.strip.png (spring on vs off)."""
from PIL import Image, ImageDraw
def grid(rows, out, w=320):
    img = Image.new('RGB', (w * len(rows[0][1]) + 90, w * len(rows)), (14, 14, 16)); d = ImageDraw.Draw(img)
    for r, (label, files) in enumerate(rows):
        d.text((8, r * w + w // 2), label, fill=(230, 230, 230))
        for c, f in enumerate(files): img.paste(Image.open(f).convert('RGB').resize((w, w)), (90 + c * w, r * w))
    img.save(out)
names = ['01_static', '06_yaw20', '07_yaw40', '08_static_yaw40']
grid([('v2', [f'shots/v2/{n}.png' for n in names]), ('v3', [f'shots/{n}.png' for n in names])], 'shots/compare_v2_v3.png')
grid([('v3', [f'shots/v3/{n}.png' for n in names]), ('slime', [f'shots/{n}.png' for n in names])], 'shots/compare_v3_slime.png')
ts = ['0.2', '0.25', '0.3', '0.35', '0.45', '0.6', '0.9']
grid([('spring ON', [f'shots/physics/on_{t}.png' for t in ts]), ('spring OFF', [f'shots/physics/off_{t}.png' for t in ts])], 'shots/physics/physics.strip.png', 240)
