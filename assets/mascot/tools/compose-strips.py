"""Joins shots/presets/<clip>_<t>.png into one labelled strip per clip (shots/presets/<clip>.strip.png)."""
import glob, os, re
from collections import defaultdict
from PIL import Image, ImageDraw
groups = defaultdict(list)
for f in glob.glob('shots/presets/*_*.png'):
    m = re.match(r'(.+)_([\d.]+)\.png$', os.path.basename(f))
    if m: groups[m[1]].append((float(m[2]), f))
for clip, items in groups.items():
    items.sort(); w = 360
    out = Image.new('RGB', (w * len(items), w + 28), (14, 14, 16)); d = ImageDraw.Draw(out)
    for i, (t, f) in enumerate(items):
        out.paste(Image.open(f).convert('RGB'), (i * w, 28)); d.text((i * w + 8, 8), f'{clip}  t={t}s', fill=(230, 230, 230))
    out.save(f'shots/presets/{clip}.strip.png')
print('strips', sorted(groups))
