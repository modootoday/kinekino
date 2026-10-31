#!/usr/bin/env python3
"""Derives every kinekino icon file from the finished mascot logo.

Source of truth: assets/design/brand-source/kinekino-mark-f_1.6.svg, a verbatim copy of the
eyes-open front frame (t = 1.6 s) of the mascot's animated logo, the same character the rigged
3D mascot is built from. Nothing is drawn freehand here: body, inner ears, lower highlight band,
tail tip and eyes are read from that file's paths and recombined.

The 16 and 24 px icons use a simplified variant of the same shapes (no tail tip, ears and eyes
scaled up around their own centres) so the eyes and inner ears stay visible at that size.

Outputs (all regenerated, safe to delete and rerun):
  docs/assets/img/favicon.svg             full-colour mark on the dark rounded plate
  assets/design/icons/mascot-color.svg    full-colour mark, transparent background
  assets/design/icons/mascot-mark.svg     one-colour mask (eyes and inner ears cut out)
  assets/design/icons/mascot-small.svg    simplified variant for 16/24 px (title bar, small app icons)
  assets/design/app-icon/kinekino-<n>.png, kinekino.ico
  docs/assets/img/favicon.ico, favicon-16/32.png, apple-touch-icon.png, icon-192/512.png,
  site.webmanifest

Requires python3 + Pillow and Chrome/Chromium (CHROME, default google-chrome).
Run from the repo root: python3 tools/brand-icons/derive.py
"""
import json, os, re, subprocess, sys, tempfile
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "assets/design/brand-source/kinekino-mark-f_1.6.svg"
ICONS = ROOT / "assets/design/icons"
APP = ROOT / "assets/design/app-icon"
SITE = ROOT / "docs/assets/img"
SIZES = [16, 24, 32, 48, 64, 128, 180, 192, 256, 512]

svg = SRC.read_text()
colours = dict(re.findall(r"--(\w+):(#[0-9A-Fa-f]{6})", svg))
plate_colour = re.search(r'<rect [^>]*fill="(#[0-9A-Fa-f]{6})"', svg).group(1)
paths = re.findall(r'<path (?:class="(\w+)" )?d="([^"]+)" fill="var\(--(\w+)\)"', svg)
eyes = re.findall(r'<ellipse cx="([\d.]+)" cy="([\d.]+)" rx="([\d.]+)" ry="([\d.]+)"', svg)
body = [d for c, d, f in paths if f == "body"]
ears = [d for c, d, f in paths if f == "ear"]
band = [d for c, d, f in paths if f == "jaw" and not c]
tail = [d for c, d, f in paths if f == "jaw" and c == "tail"]
if len(body) != 1 or len(ears) != 2 or len(band) != 1 or len(tail) != 1 or len(eyes) != 2:
    sys.exit(f"unexpected mascot shape in {SRC}")
body, band, tail = body[0], band[0], tail[0]
C = {k: colours[k] for k in ("body", "ear", "jaw", "ink")}

def pts(d):
    return [tuple(map(float, p)) for p in re.findall(r"([\d.]+) ([\d.]+)", d)]

def scaled(d, k):
    p = pts(d); cx = sum(x for x, _ in p) / len(p); cy = sum(y for _, y in p) / len(p)
    return "M" + " L".join(f"{cx+(x-cx)*k:.2f} {cy+(y-cy)*k:.2f}" for x, y in p) + " Z"

def ell(cx, cy, rx, ry):
    return f'<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}"/>'

def mark(ear_d, eye_list, with_tail=True):
    s = (f'<defs><clipPath id="b"><path d="{body}"/></clipPath></defs>'
         f'<path d="{body}" fill="{C["body"]}" stroke="{C["body"]}" stroke-width="4" stroke-linejoin="round"/>')
    s += "".join(f'<path d="{e}" fill="{C["ear"]}" stroke="{C["ear"]}" stroke-width="1.5" stroke-linejoin="round"/>' for e in ear_d)
    s += f'<g clip-path="url(#b)"><path d="{band}" fill="{C["jaw"]}"/></g>'
    if with_tail:
        s += f'<path d="{tail}" fill="{C["jaw"]}" stroke="{C["jaw"]}" stroke-width="1.5" stroke-linejoin="round"/>'
    s += f'<g fill="{C["ink"]}">' + "".join(ell(*e) for e in eye_list) + "</g>"
    return s

def doc(inner, vb="10 6 100 106", plate=False):
    bg = f'<rect width="120" height="120" rx="26" fill="{plate_colour}"/>' if plate else ""
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb}">{bg}{inner}</svg>\n'

small_eyes = [(cx, cy, f"{float(rx)*1.8:.1f}", f"{float(ry)*1.5:.1f}") for cx, cy, rx, ry in eyes]
full = mark(ears, eyes)
small = mark([scaled(e, 1.45) for e in ears], small_eyes, with_tail=False)
(SITE / "favicon.svg").write_text(doc(full, "0 0 120 120", plate=True))
(ICONS / "mascot-color.svg").write_text(doc(full))
(ICONS / "mascot-small.svg").write_text(doc(small))
SMALL_PLATED = Path(tempfile.mkdtemp()) / "small.svg"
SMALL_PLATED.write_text(doc(small, "0 0 120 120", plate=True))
holes = " ".join(ears) + " " + " ".join(
    f"M{float(cx)-float(rx)} {cy} a{rx} {ry} 0 1 0 {2*float(rx)} 0 a{rx} {ry} 0 1 0 {-2*float(rx)} 0Z" for cx, cy, rx, ry in eyes)
(ICONS / "mascot-mark.svg").write_text(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="10 6 100 106"><defs><mask id="m">'
    f'<path d="{body}" fill="#fff" stroke="#fff" stroke-width="4" stroke-linejoin="round"/>'
    f'<path d="{holes}" fill="#000"/></mask></defs>'
    '<rect x="0" y="0" width="120" height="120" fill="#fff" mask="url(#m)"/></svg>\n')

chrome = os.environ.get("CHROME", "google-chrome")
APP.mkdir(parents=True, exist_ok=True)

def raster(src, size, out):
    with tempfile.TemporaryDirectory() as tmp:
        page = Path(tmp) / "i.html"
        page.write_text(f'<html><body style="margin:0;background:transparent">'
                        f'<img src="file://{src}" width="{size}" height="{size}" style="display:block"></body></html>')
        subprocess.run([chrome, "--headless=new", "--no-sandbox", "--disable-gpu", "--hide-scrollbars",
                        "--force-device-scale-factor=1", "--default-background-color=00000000",
                        f"--window-size={size},{size}", f"--screenshot={out}", f"file://{page}"], check=True,
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    Image.open(out).convert("RGBA").crop((0, 0, size, size)).save(out, optimize=True)

for n in SIZES:
    raster(SMALL_PLATED if n <= 24 else SITE / "favicon.svg", n, APP / f"kinekino-{n}.png")
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
    "theme_color": plate_colour, "background_color": plate_colour, "display": "browser",
}
(SITE / "site.webmanifest").write_text(json.dumps(manifest, indent=2) + "\n")
print("derived from", SRC.relative_to(ROOT))
