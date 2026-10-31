#!/usr/bin/env python3
"""Derives the kinekino logo icons and the situational mascot marks from the brand files.

Design rule (assets/mascot docs, part-layers: "정지 상태는 모든 weights 0, eyes.scale 0 → 단색
유지"): the logo is the mascot at rest, a plain V-notch body with no eyes, inner ears or band.
The mascot is the logo changing with the situation: a reveal shows eyes, inner ears and band,
and app-state combos (loading, error, offline, idle, ...) move them. So:

  logo (rest)  assets/design/brand-source/kinekino-mark.svg   (logo/kinekino-mark.svg, frame 1)
  idle         assets/design/brand-source/kinekino-mark-f_1.6.svg (eyes open, revealed rest)
  loading      assets/design/brand-source/kinekino-mark-f_3.3.svg (blink frame; loading blinks)
  error        assets/design/brand-source/combo_scared-v7.png (3D render of `scared`, the
               fatal-error combo; no render of `error`/worried eyes exists, nearest one)

Nothing is drawn here: shapes are copied out of those files, frames are rasterised as they are.

Outputs: assets/design/icons/logo.svg (no plate), docs/assets/img/favicon.svg (on the logo's
plate), assets/design/icons/mascot-{idle,loading,error}.png, assets/design/app-icon/*.png/.ico,
docs/assets/img favicon set and site.webmanifest.
Requires python3 + Pillow and Chrome/Chromium (CHROME). Run from the repo root.
"""
import json, os, re, subprocess, tempfile
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
BRAND = ROOT / "assets/design/brand-source"
ICONS = ROOT / "assets/design/icons"
APP = ROOT / "assets/design/app-icon"
SITE = ROOT / "docs/assets/img"
SIZES = [16, 24, 32, 48, 64, 128, 180, 192, 256, 512]
chrome = os.environ.get("CHROME", "google-chrome")

logo = (BRAND / "kinekino-mark.svg").read_text()
plate = re.search(r'<rect [^>]*fill="(#[0-9A-Fa-f]{6})"', logo).group(1)
body = re.search(r'<path [^>]*/>', logo).group(0)
# viewBox: the body's bounds (20..100 x 14..106) plus the 2-unit stroke and a small margin.
(ICONS / "logo.svg").write_text(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="14 8 92 104">{body}</svg>\n')
(SITE / "favicon.svg").write_text(re.sub(r'<svg [^>]*>', '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 120">', logo, count=1))

def shoot(svg_text, size, out, crop=None):
    with tempfile.TemporaryDirectory() as tmp:
        src = Path(tmp) / "s.svg"; src.write_text(svg_text)
        page = Path(tmp) / "i.html"
        page.write_text(f'<html><body style="margin:0;background:transparent"><img src="file://{src}" '
                        f'width="{size}" height="{size}" style="display:block"></body></html>')
        subprocess.run([chrome, "--headless=new", "--no-sandbox", "--disable-gpu", "--hide-scrollbars",
                        "--force-device-scale-factor=1", "--default-background-color=00000000",
                        f"--window-size={size},{size}", f"--screenshot={out}", f"file://{page}"], check=True,
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    Image.open(out).convert("RGBA").crop((0, 0, size, size)).save(out, optimize=True)

# An <img> renders the paused CSS animation at the frame's time, so the frames rasterise as drawn.
# Plate removed so the mark sits on the panel colour.
for name, frame in (("idle", "kinekino-mark-f_1.6.svg"), ("loading", "kinekino-mark-f_3.3.svg")):
    text = re.sub(r'<rect width="120" height="120"[^>]*/>', "", (BRAND / frame).read_text())
    shoot(text, 128, ICONS / f"mascot-{name}.png")

# 3D render sheet: last panel (t=2.0s), dark render background keyed out.
sheet = Image.open(BRAND / "combo_scared-v7.png").convert("RGBA")
panel = sheet.crop((770, 40, 930, 200))
bg = panel.getpixel((2, 2))
px = [(r, g, b, 0 if abs(r - bg[0]) + abs(g - bg[1]) + abs(b - bg[2]) < 24 else a) for r, g, b, a in list(panel.getdata())]
panel.putdata(px)
panel.crop(panel.getbbox()).save(ICONS / "mascot-error.png", optimize=True)

APP.mkdir(parents=True, exist_ok=True)
for n in SIZES:
    shoot((SITE / "favicon.svg").read_text(), n, APP / f"kinekino-{n}.png")
ico = [Image.open(APP / f"kinekino-{n}.png") for n in (16, 24, 32, 48, 64, 256)]
ico[-1].save(APP / "kinekino.ico", sizes=[i.size for i in ico], append_images=ico[:-1])
ico[-1].save(SITE / "favicon.ico", sizes=[(16, 16), (32, 32), (48, 48)], append_images=ico[:4])
for n in (16, 32):
    Image.open(APP / f"kinekino-{n}.png").save(SITE / f"favicon-{n}.png")
for n in (192, 512):
    Image.open(APP / f"kinekino-{n}.png").save(SITE / f"icon-{n}.png")
Image.open(APP / "kinekino-180.png").convert("RGB").save(SITE / "apple-touch-icon.png", optimize=True)
manifest = {
    "name": "kinekino", "short_name": "kinekino",
    "icons": [{"src": "favicon.svg", "type": "image/svg+xml", "sizes": "any"},
              {"src": "icon-192.png", "type": "image/png", "sizes": "192x192"},
              {"src": "icon-512.png", "type": "image/png", "sizes": "512x512"}],
    "theme_color": plate, "background_color": plate, "display": "browser",
}
(SITE / "site.webmanifest").write_text(json.dumps(manifest, indent=2) + "\n")
print("logo from kinekino-mark.svg; mascot states idle/loading/error")

