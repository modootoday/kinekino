# Compose /tmp/kk-frames/* (from render-sheets.sh) into labelled PNG keyframe sheets under shots/v6.
from PIL import Image, ImageDraw
import os, glob, json
F = '/tmp/kk-frames'
P = json.load(open('src/presets/parts.json'))['clips']; L = json.load(open('src/presets/layers.json'))['combos']
def sheet(d, out, title, labels, cols=None, sz=300):
    fs = sorted(glob.glob(f'{F}/{d}/*.png')); cols = cols or len(fs); rows = (len(fs) + cols - 1) // cols
    W = Image.new('RGB', (cols * sz, rows * sz + 28), (14, 14, 16)); g = ImageDraw.Draw(W); g.text((8, 8), title, fill='white')
    for i, f in enumerate(fs):
        x, y = (i % cols) * sz, 28 + (i // cols) * sz
        W.paste(Image.open(f).convert('RGB').resize((sz, sz)), (x, y)); g.text((x + 6, y + 6), labels[i], fill=(255, 220, 120))
    W.save(out); return W
os.makedirs('shots/v6/parts', exist_ok=True); tiles = []
for k in sorted(os.listdir(F)):
    if k.startswith('part_'):
        n = k[5:]; d = P[n]['duration']
        tiles.append(sheet(k, f'shots/v6/parts/{n}.png', f'part {n} ({P[n]["layer"]})', [f't={d*f:.2f}s' for f in (.25, .5, .75)], sz=240))
    elif k.startswith('combo_'):
        n = k[6:]; d = L[n]['duration']
        sheet(k, f'shots/v6/combo_{n}.png', f'combo {n}: ' + ' + '.join(t[1] for t in L[n]['tracks']), [f't={d*f:.2f}s' for f in (.15, .35, .55, .75)])
    elif k.startswith('look'):
        sheet(k, 'shots/v6/look9.png', 'look-at: gaze fixed (pointer grid)', [f'gaze ({x},{y})' for y in (1, 0, -1) for x in (-1, 0, 1)], cols=3)
if tiles:
    W = Image.new('RGB', (720 * 3, (len(tiles) + 2) // 3 * 268), (14, 14, 16))
    for i, t in enumerate(tiles): W.paste(t.resize((720, 268)), ((i % 3) * 720, (i // 3) * 268))
    W.save('shots/v6/parts_overview.png')
