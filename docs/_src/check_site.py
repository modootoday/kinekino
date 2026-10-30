#!/usr/bin/env python3
"""Site checks (no Jekyll needed): _data/languages.yml is well formed; every page has a
translation for each `full: true` language (others need at least their home page), every page has a Markdown twin stub, every _data/i18n/<lang>.yml
has the same keys as the default language, and front matter has title/description/ref.
Optional: pass a built _site dir to check internal links. Exit 1 on any error."""
import os, re, sys
import yaml  # PyYAML
D = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
cfg = yaml.safe_load(open(os.path.join(D, "_config.yml")))
dflt = cfg["default_lang"]
LL = yaml.safe_load(open(os.path.join(D, "_data", "languages.yml")))
langs = [x.get("code") for x in LL]
full = {x.get("code") for x in LL if x.get("full")}
skip = set(cfg.get("exclude", [])) | {"_site", "_src"}
err = []
if len(set(langs)) != len(langs): err.append("_data/languages.yml: duplicate code")
if dflt not in full: err.append(f"_data/languages.yml: default `{dflt}` must be listed with full: true")
for x in LL:
    for k in ("code", "name", "en"):
        if not x.get(k): err.append(f"_data/languages.yml: {x} missing `{k}`")
    if x.get("dir", "ltr") not in ("ltr", "rtl"): err.append(f"_data/languages.yml: {x.get('code')} dir must be ltr|rtl")
def fm(p):
    s = open(p, encoding="utf-8").read()
    m = re.match(r"---\n(.*?)\n---\n", s, re.S)
    return yaml.safe_load(m.group(1)) if m else None
pages = {}
devpages = []
for root, dirs, files in os.walk(D):
    dirs[:] = [d for d in dirs if not d.startswith("_") and d not in skip and not d.startswith(".")]
    for f in files:
        if not f.endswith(".md"): continue
        p = os.path.join(root, f); rel = os.path.relpath(p, D); meta = fm(p)
        if meta is None: err.append(f"{rel}: no front matter"); continue
        top = rel.split(os.sep)[0]; lang = top if top in langs and top != dflt else dflt
        for k in ("title", "description", "ref"):
            if not meta.get(k): err.append(f"{rel}: missing `{k}`")
        if meta.get("dev"):  # internal page (styleguide): en only, no twin, must be hidden
            if meta.get("sitemap") is not False or not meta.get("noindex") or meta.get("nav_order"): err.append(f"{rel}: dev page needs sitemap: false, noindex: true, no nav_order")
            devpages.append(rel); continue
        pages.setdefault(meta.get("ref"), {}).setdefault(lang, []).append(rel)
        if not os.path.exists(p + ".txt"): err.append(f"{rel}: missing twin stub {rel}.txt")
for ref, by in pages.items():
    for l in langs:
        if l not in by:
            if l in full or ref == "index": err.append(f"ref `{ref}`: no `{l}` page (have {sorted(by)})")
        elif len(by[l]) > 1: err.append(f"ref `{ref}`: {len(by[l])} `{l}` pages {by[l]}")
def keys(o, pre=""):
    out = set()
    if isinstance(o, dict):
        for k, v in o.items(): out |= {pre + k} | keys(v, pre + k + ".")
    elif isinstance(o, list):
        for i, v in enumerate(o): out |= keys(v, f"{pre}{i}.")
    return out
I = os.path.join(D, "_data", "i18n")
base = keys(yaml.safe_load(open(os.path.join(I, dflt + ".yml"))))
for need in ("callout.note", "callout.caution", "status.available", "status.partial", "status.planned", "mock.label", "mock.alt.playback", "language", "languages.label", "nav.label", "nav.features"):
    if need not in base: err.append(f"i18n {dflt}: UI key `{need}` missing")
for l in langs:
    f = os.path.join(I, l + ".yml")
    if not os.path.exists(f): err.append(f"_data/i18n/{l}.yml missing"); continue
    k = keys(yaml.safe_load(open(f)))
    for m in sorted(base - k): err.append(f"i18n {l}: missing key `{m}` (falls back to {dflt})")
    for m in sorted(k - base): err.append(f"i18n {l}: extra key `{m}` not in {dflt}")
# --- jekyll-theme-modootoday (md-components v1) + kinekino extension (see .spec/guide/site-github-pages.guide.md) ---
# The theme's modootoday-check covers sections counts (item i = i-th `##`), _data/components.yml params/slots,
# Markdown purity, twins and llms. Here: the site's `hero:` and visuals (_data/site-components.yml), status vocab,
# same sections in every language, theme pin and language lists in sync.
REG = yaml.safe_load(open(os.path.join(D, "_data", "components.yml")))
EXT = yaml.safe_load(open(os.path.join(D, "_data", "site-components.yml")))
VOCAB = {"available", "partial", "planned"}
rt = str(cfg.get("remote_theme", ""))
if not re.fullmatch(r"modootoday/jekyll-theme-modootoday@(v\d+\.\d+\.\d+|[0-9a-f]{40})", rt): err.append(f"_config.yml: remote_theme must pin modootoday/jekyll-theme-modootoday@<tag|sha>, got {rt!r}")
mt = cfg.get("modootoday") or {}
if mt.get("languages") != langs: err.append(f"_config.yml: modootoday.languages {mt.get('languages')} != _data/languages.yml {langs}")
for f in ("modootoday/markdown/components.html", "default.html"):
    for d in ("_includes", "_layouts"):
        if os.path.exists(os.path.join(D, d, f)) and not (d == "_layouts" and f == "doc.html"): err.append(f"{d}/{f}: copied theme file; use the theme's")
# kit 1.0 removes md-* classes and --md-* properties without aliases: site code must use mt-* / --mt-* only
for d in ("_sass", "_includes", "_layouts", "assets"):
    for r, _, fs in os.walk(os.path.join(D, d)):
        for fn in fs:
            if not fn.endswith((".scss", ".css", ".html", ".js")): continue
            fp = os.path.join(r, fn)
            for i, ln in enumerate(open(fp, encoding="utf-8"), 1):
                if re.search(r"--md-|(?<![\w-])\.md-[a-z]|class=\"[^\"]*\bmd-[a-z]", ln): err.append(f"{os.path.relpath(fp, D)}:{i}: md-* / --md-* removed in kit 1.0; use mt-*")
def body_of(p):
    return re.sub(r"\A---\n.*?\n---\n", "", open(p, encoding="utf-8").read(), flags=re.S)
def nocode(t):
    t = re.sub(r"^(```|~~~).*?^\1\s*$", "", t, flags=re.S | re.M)
    return t
def check_params(rel, where, it, params):
    for k, v in it.items():
        if k in ("c", "name"): continue
        if k not in params: err.append(f"{rel}: {where} `{it.get('c')}` has no param `{k}`"); continue
        al = (params[k] or {}).get("allowed")
        if al and v not in al: err.append(f"{rel}: {where} `{it.get('c')}.{k}` = {v!r} not in {al}")
    for k, ps in params.items():
        if (ps or {}).get("required") and k not in it: err.append(f"{rel}: {where} `{it.get('c')}` missing required `{k}`")
    if it.get("visual"):
        vs = EXT["visual"].get(it["visual"])
        if vs is None: err.append(f"{rel}: {where} unregistered visual `{it['visual']}`")
        elif vs.get("requires") and vs["requires"] not in it: err.append(f"{rel}: {where} visual `{it['visual']}` needs `{vs['requires']}`")
secs_by_ref = {}
for ref, by in list(pages.items()) + [("dev:" + r, {"en": [r]}) for r in devpages]:
    for l, rels in by.items():
        for rel in rels:
            meta = fm(os.path.join(D, rel)); b = nocode(body_of(os.path.join(D, rel)))
            for v in re.findall(r"`status:([^`]*)`", b):
                if v not in VOCAB: err.append(f"{rel}: status badge `status:{v}` not in {sorted(VOCAB)}")
            nh2 = len(re.findall(r"^## ", b, flags=re.M))
            S = meta.get("sections") or []; H = meta.get("hero")
            if not isinstance(S, list): err.append(f"{rel}: sections must be a list"); continue
            if S and len(S) != nh2: err.append(f"{rel}: {len(S)} sections items but {nh2} top-level ## headings")
            if H is not None:
                if not isinstance(H, dict) or H.get("c", "prose") not in EXT["hero"]: err.append(f"{rel}: hero must be a mapping with c in {sorted(EXT['hero'])}")
                else: check_params(rel, "hero", H, EXT["hero"][H.get("c", "prose")].get("params") or {})
            for i, it in enumerate(S):
                if not isinstance(it, dict): err.append(f"{rel}: sections[{i}] must be a mapping"); continue
                c = it.get("c", "prose"); spec = REG.get(c)
                if spec is None: err.append(f"{rel}: sections[{i}] unregistered component `{c}`"); continue
                check_params(rel, f"sections[{i}]", it, (spec or {}).get("params") or {})
            secs_by_ref.setdefault(ref, {})[l] = (H, S)
for ref, by in secs_by_ref.items():
    if ref.startswith("dev:"): continue
    vals = list(by.values())
    if any(v != vals[0] for v in vals): err.append(f"ref `{ref}`: sections differ between languages {sorted(by)}")
# Markdown purity is checked by the theme: modootoday-check docs <built-site> --twins.
if len(sys.argv) > 1:  # built site checks
    S = sys.argv[1]
    for ref, by in pages.items():
        for l, rels in by.items():
            for rel in rels:
                tw = os.path.join(S, re.sub(r"(^|/)index\.md$", r"\1", rel).replace(".md", "/") + "index.md")
                if not os.path.exists(tw): err.append(f"{rel}: built twin {tw} missing"); continue
                want = re.sub(r"\A\s*# .*\n", "", body_of(os.path.join(D, rel))).strip()
                if want not in open(tw, encoding="utf-8").read(): err.append(f"{rel}: twin body differs from source")
    for root, _, files in os.walk(S):
        for f in files:
            if f.endswith(".html"):
                h = open(os.path.join(root, f), encoding="utf-8").read()
                if re.search(r"\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]", h): err.append(f"{os.path.relpath(os.path.join(root,f),S)}: unconverted alert marker")
                if re.search(r"<code[^>]*>status:", h): err.append(f"{os.path.relpath(os.path.join(root,f),S)}: unconverted status badge")
if len(sys.argv) > 1:  # built site link check
    S = sys.argv[1]
    for root, _, files in os.walk(S):
        for f in files:
            if not f.endswith(".html"): continue
            h = open(os.path.join(root, f), encoding="utf-8").read()
            for u in re.findall(r'(?:href|src)="(/[^"#?]*)"', h):
                t = os.path.join(S, u.lstrip("/"))
                if not (os.path.isfile(t) or os.path.isfile(os.path.join(t, "index.html"))):
                    err.append(f"{os.path.relpath(os.path.join(root,f),S)}: broken link {u}")
            if '<html lang="' not in h: err.append(f"{f}: no <html lang>")
for e in err: print("error:", e)
print(f"{len(pages)} refs x {len(langs)} langs ({len(full)} full), {len(err)} errors")
sys.exit(1 if err else 0)
