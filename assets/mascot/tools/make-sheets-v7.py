"""Compose /tmp/kk-frames/* into shots/v7 sheets (rows = clips, cols = frames) + before/after compare."""
import sys, os
from PIL import Image, ImageDraw, ImageChops
F, O = '/tmp/kk-frames', 'shots/v7'; os.makedirs(O, exist_ok=True)
def sheet(key, labels, cols, colnames, out, w=200):
    files = sorted(f for f in os.listdir(f'{F}/{key}') if f.endswith('.png'))
    rows = len(files) // cols; img = Image.new('RGB', (150 + w * cols, 24 + w * rows), (14, 14, 16)); d = ImageDraw.Draw(img)
    for c, n in enumerate(colnames): d.text((150 + c * w + 6, 6), n, fill=(200, 200, 200))
    for i, f in enumerate(files):
        r, c = divmod(i, cols); im = Image.open(f'{F}/{key}/{f}').convert('RGB'); s = min(im.size); im = im.crop(((im.width - s) // 2, (im.height - s) // 2, (im.width + s) // 2, (im.height + s) // 2))
        img.paste(im.resize((w, w)), (150 + c * w, 24 + r * w))
        if c == 0: d.text((6, 24 + r * w + w // 2), labels[r], fill=(235, 235, 235))
    img.save(f'{O}/{out}.png')
for line in open('/tmp/kk/specs.txt'):
    head, clips, fr = line.strip().split('|'); g = head.split(':')[1]
    sheet(f'multi_{g}', clips.split(','), 4, ['20%', '40%', 'hold', '80%'], f'group_{g}')
for spec in sys.argv[1:]:
    n, t = spec.split('@'); sheet(f'combo_{n}', [n], 4, [f't={x}s' for x in t.split(',')], f'combo_{n}')
B = '/tmp/kk/ba'; w = 360; img = Image.new('RGB', (90 + 3 * w, 2 * w + 30), (14, 14, 16)); d = ImageDraw.Draw(img)
for c, y in enumerate([0, 20, 40]):
    d.text((90 + c * w + 6, 8), f'yaw {y}', fill=(200, 200, 200))
    for r, v in enumerate(['v6', 'v7']):
        img.paste(Image.open(f'{B}/{v}_{y}.png').convert('RGB').resize((w, w)), (90 + c * w, 30 + r * w)); d.text((8, 30 + r * w + w // 2), v, fill=(235, 235, 235))
    diff = ImageChops.difference(Image.open(f'{B}/v6_{y}.png').convert('RGB'), Image.open(f'{B}/v7_{y}.png').convert('RGB')).getbbox()
    print('yaw', y, 'diff bbox', diff)
img.save(f'{O}/before_after_static.png')
