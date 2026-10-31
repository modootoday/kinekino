#!/usr/bin/env python3
"""Derives every kinekino icon file from the one mascot symbol.

Source of truth: docs/assets/img/favicon.svg (the cat symbol: ears, body, two eyes; the same
mark the 3D mascot in assets/mascot is built from). Nothing is drawn here: the outline and eye
coordinates are read from that file and either recoloured, cut out as a mask, or rasterised.

Outputs (all regenerated, safe to delete and rerun):
  assets/design/icons/mascot-mark.svg   one-colour mask version (eyes cut out) for the
                                        player's CSS mask icons (title bar, start screen)
  assets/design/icons/mascot-color.svg  full-colour copy for illustrations (empty/error states)
  assets/design/app-icon/kinekino-<n>.png  app/window/tray icon, n = 16..512
  assets/design/app-icon/kinekino.ico   Windows app icon (16, 24, 32, 48, 64, 256)
  docs/assets/img/favicon.ico, favicon-16/32.png, apple-touch-icon.png (180, dark plate),
  icon-192.png, icon-512.png, site.webmanifest

Requires python3 + Pillow and a Chrome/Chromium binary (CHROME, default google-chrome) for
rasterising SVG. Run from the repo root: python3 tools/brand-icons/derive.py
"""
import json, os, re, subprocess, sys, tempfile
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "docs/assets/img/favicon.svg"
ICONS = ROOT / "assets/design/icons"
APP = ROOT / "assets/design/app-icon"
SITE = ROOT / "docs/assets/img"
PLATE = "#141414"  # player chrome background, used only behind apple-touch-icon
SIZES = [16, 24, 32, 48, 64, 128, 180, 192, 256, 512]

svg = SRC.read_text()
outline = re.search(r'<path d="([^"]+)" fill="(#[0-9a-fA-F]+)"', svg)
eyes = re.findall(r'<ellipse cx="([\d.]+)" cy="([\d.]+)" rx="([\d.]+)" ry="([\d.]+)"', svg)
if not outline or len(eyes) != 2:
    sys.exit(f"unexpected symbol shape in {SRC}")
body, colour = outline.groups()

def ellipse_path(cx, cy, rx, ry):
    return f"M{float(cx)-float(rx)} {cy} a{rx} {ry} 0 1 0 {2*float(rx)} 0 a{rx} {ry} 0 1 0 {-2*float(rx)} 0Z"

mask = ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">'
        f'<path fill="#fff" fill-rule="evenodd" d="{body} {" ".join(ellipse_path(*e) for e in eyes)}"/></svg>\n')
(ICONS / "mascot-mark.svg").write_text(mask)
(ICONS / "mascot-color.svg").write_text(svg if svg.endswith("\n") else svg + "\n")

chrome = os.environ.get("CHROME", "google-chrome")
APP.mkdir(parents=True, exist_ok=True)

def raster(size, out):
    with tempfile.TemporaryDirectory() as tmp:
        page = Path(tmp) / "i.html"
        page.write_text(f'<html><body style="margin:0;background:transparent">'
                        f'<img src="file://{SRC}" width="{size}" height="{size}" style="display:block"></body></html>')
        subprocess.run([chrome, "--headless=new", "--no-sandbox", "--disable-gpu", "--hide-scrollbars",
                        "--default-background-color=00000000", f"--window-size={size},{size}",
                        f"--screenshot={out}", f"file://{page}"], check=True,
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    Image.open(out).convert("RGBA").crop((0, 0, size, size)).save(out, optimize=True)

for n in SIZES:
    raster(n, APP / f"kinekino-{n}.png")
big = Image.open(APP / "kinekino-256.png")
big.save(APP / "kinekino.ico", sizes=[(s, s) for s in (16, 24, 32, 48, 64, 256)])
big.save(SITE / "favicon.ico", sizes=[(16, 16), (32, 32), (48, 48)])
for n in (16, 32):
    Image.open(APP / f"kinekino-{n}.png").save(SITE / f"favicon-{n}.png")
for n in (192, 512):
    Image.open(APP / f"kinekino-{n}.png").save(SITE / f"icon-{n}.png")
plate = Image.new("RGBA", (180, 180), PLATE)
inner = Image.open(APP / "kinekino-128.png")
plate.alpha_composite(inner, (26, 22))
plate.convert("RGB").save(SITE / "apple-touch-icon.png", optimize=True)
manifest = {
    "name": "kinekino", "short_name": "kinekino",
    "icons": [{"src": "favicon.svg", "type": "image/svg+xml", "sizes": "any"},
              {"src": "icon-192.png", "type": "image/png", "sizes": "192x192"},
              {"src": "icon-512.png", "type": "image/png", "sizes": "512x512"}],
    "theme_color": PLATE, "background_color": PLATE, "display": "browser",
}
(SITE / "site.webmanifest").write_text(json.dumps(manifest, indent=2) + "\n")
print("derived from", SRC.relative_to(ROOT), "colour", colour)
