---
id: guide:kinekino/kit-gaps-modootoday
title: jekyll-theme-modootoday defects and gaps
status: active
created: 2026-10-30
updated: 2026-10-30
---
# jekyll-theme-modootoday: defects and gaps found while converting kinekino.com (2026-10-30)

Consumer: /workspace/kinekino-wt-site/docs (branch feat/site-mockup), remote_theme pin
6c0a1bc842bb93f8098a3e90f9d8f1cd6f7e8d73. Result: modootoday-check --twins --strict OK (29 pages), but the site
still needs its own doc layout and section include. Read-only report; the kit repo was not modified.

## Gaps (site had to work around)
1. **No section visual / per-section hook.** `sections:` items accept params but md-sections.html has no way to render
   a site include next to a section (kinekino: `visual: player-mock, scene: …`). Site keeps its own
   `_includes/site-product.html` and `_layouts/doc.html` that re-implement the `\n<h2 ` split. Proposal: optional
   `visual:` param → `{% include sections/<visual>.html section=… %}`, or a `section-custom.html` hook per section.
2. **No hero item.** The part before the first `##` cannot be styled from front matter (kinekino uses a site-only
   `hero: {c, visual, scene}` key the checker ignores). Proposal: document a `hero:` key (not counted in `sections:`).
3. **Header brand slot.** `.mt-site-title` is text only; no logo/icon include (site uses a CSS ::before favicon).
   Proposal: `modootoday.logo` or `header-brand.html` hook.
4. **`seo: false` drops hreflang too.** Sites that bring jekyll-seo-tag lose the theme's hreflang alternates and
   must re-emit them (kinekino `_includes/alternates.html`). Proposal: split `seo` (OG/canonical) from `hreflang`.
5. **Two language lists.** `modootoday.languages` duplicates a site's own `_data/languages.yml`; nothing checks they
   agree (site check_site.py now does). Proposal: read `_data/languages.yml` when present, or check consistency.
6. **Theme toggle is a text button** ("Dark mode") with no icon option; on mobile it competes with the nav for room.
7. **Token override needs specificity hacks.** Kit scheme selectors (`:root[data-theme]`, media `:root:not(...)`) outrank
   a plain `:root` override in site CSS; site used `:root:root:root`. Proposal: document a supported override
   pattern (e.g. a `$mt-semantic-light/dark` Sass map, or `@layer` the kit tokens).
8. **`.mt-main` assumes a reading column** (max-width/padding). Full-bleed landing pages must reset it; no
   `layout: landing`/`page.full_width` option.

## Defects / doc issues
9. Task/docs refer to `docs/api/`; the API spec is a single file `docs/api.md`.
10. `--github-pages` requires a committed Gemfile.lock; sites that gitignore it cannot use the check. Suggest a
    clearer message or a `--github-pages=232` explicit version form.
11. Badge HTML still carries `md-badge md-badge-*` aliases while the checker warns on `md-*` in site code; fine for
    1.x but a site cannot opt out of alias output (`modootoday.aliases: false`?).
12. `sections:` item count must equal `##` count exactly; previous draft allowed fewer (default prose). Breaking for
    draft users — worth a line in docs/upgrade.md with a migration script.
