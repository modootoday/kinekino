---
id: guide:kinekino/site-github-pages
title: kinekino 홈페이지(GitHub Pages·Jekyll) 구조와 다국어 규약
status: active
created: 2026-10-30
updated: 2026-10-30
rel:
  governs: [docs]
---

# kinekino 홈페이지(GitHub Pages·Jekyll) 구조와 다국어 규약

## Overview
`docs/`를 GitHub Pages 소스(`main`의 `/docs`, Actions 없음)로 쓰는 정적 사이트 시안. Specfold 사이트(`/workspace/specfold-src/docs/`)의 규약을 그대로 따르고, kinekino 고유 요소(마스코트, 다국어, AI 네이티브 CLI 안내)를 더했다. 도메인은 Cloudflare 존 `kinekino.com`(읽기 조회로 확인)이며 `docs/CNAME` 한 줄. 아직 배포하지 않았다.

## Steps

### 인계받은 Specfold 규약
- 테마 없음, 자체 `_layouts`(default, doc, twin)와 `_includes`. 플러그인은 GitHub Pages 허용 목록의 `jekyll-seo-tag`, `jekyll-sitemap`, `jekyll-relative-links`만.
- kramdown(`input: GFM`) + rouge, `permalink: pretty`, `url`은 커스텀 도메인.
- `exclude`를 지정하면 기본값이 대체되므로 기본값을 다시 나열한다. kinekino는 저장소 문서 폴더 `spec/`, `continuation/`도 제외한다.
- `Gemfile`(`github-pages ~> 232`, webrick)은 로컬 미리보기 전용. `Gemfile.lock`, `_site/`, `.jekyll-cache/`는 커밋하지 않는다(`docs/.gitignore`).
- 본문은 표준 Markdown + front matter만. 에이전트용 마크다운 트윈은 페이지마다 `<이름>.md.txt` 스텁(`layout: twin`, `twin_of`, `permalink: <url>index.md`). `llms.txt`, `llms-full.txt`는 `nav_order`가 있는 페이지로 자동 생성.
- Specfold와 다른 점: Tailwind 빌드 대신 손으로 쓴 `assets/css/site.css` 하나(빌드 도구 없음).

### 다국어 규약 (리서치 근거)
| 결정 | 근거 |
|---|---|
| 언어별 디렉터리: 기본 언어 `en`은 `/`, 한국어는 `/ko/` | 플러그인 없는 Jekyll 다국어의 일반 기법(letrastudio/jekyll-i18n, ranbureand/multilingual-experiment). GitHub Pages는 허용 목록 밖 플러그인을 실행하지 않으므로 `jekyll-polyglot`은 Actions 빌드가 필요해 쓰지 않는다 |
| front matter `ref`로 번역 짝 연결, `lang`은 `_config.yml` defaults(경로 `ko`)로 지정 | 같은 기법. `jekyll-seo-tag`는 `page.lang`으로 `og:locale`을 낸다 |
| `<html lang>`과 `hreflang` alternate(자기 자신 포함 모든 짝 + `x-default`=기본 언어 페이지), 절대 URL | Google Search Central "Localized versions": 각 버전이 자신과 모든 버전을 나열, 완전한 URL, 양방향이어야 함, `x-default`는 일치 언어가 없을 때의 대체 |
| sitemap은 `jekyll-sitemap` 그대로(`xhtml:link` 없음) | Google은 HTML, HTTP 헤더, sitemap 세 방법을 동등하게 보고 하나만 쓰길 권함. HTML 방식을 택했다 |
| 자동 언어 리다이렉트 없음. 전환기에서 고른 언어만 `localStorage`(`kinekino.lang`)에 기억 | 자동 리다이렉트는 크롤러와 사용자 선택을 막는다(Google 권고: 사용자가 언어를 바꿀 수 있게 링크 제공). 기억한 값으로도 이동시키지 않는다 |
| UI 문자열과 홈 섹션은 `_data/i18n/<lang>.yml`, 누락 키는 기본 언어로 폴백(`t.key | default: t0.key`) | 언어 추가 = 데이터 파일 하나 + `/<lang>/` 폴더 + `_config.yml` `languages` 한 줄 |
| 트윈과 `llms.txt`도 언어별: `/ko/<page>/index.md`, `/ko/llms.txt`, `/ko/llms-full.txt` | `docs-list.html`이 `lang`으로 거른다 |

### 검증
- `python3 docs/_src/check_site.py [_site]`: 모든 `ref`가 `languages`의 각 언어에 정확히 1개, 모든 페이지에 트윈 스텁, i18n 키가 기본 언어와 일치, front matter `title/description/ref`, 빌드 산출물을 주면 내부 링크와 `<html lang>`. 오류가 있으면 종료 코드 1. PyYAML 필요.
- 빌드: Specfold의 Ruby 3.3.6 환경(`source /tmp/mm/rbenv.sh`, `/tmp`라 재시작 시 사라질 수 있음) 후 `cd docs && BUNDLE_GEMFILE=$PWD/Gemfile bundle exec jekyll build -d /tmp/kk-site`, 끝나면 `Gemfile.lock` 삭제.

### 다이나믹 요소와 접근성
- 마스코트는 인라인 SVG(정면, 단색 둥근 눈, 일체형 몸체): 숨쉬기, 꼬리, 깜박임은 CSS, 포인터 따라보는 눈과 살짝 패럴랙스는 `assets/js/site.js`(외부 CDN 없음). 프리셋 갤러리는 `assets/mascot/sheets`에서 자른 PNG. GLB(약 5MB)와 three.js는 쓰지 않았다.
- `prefers-reduced-motion`이면 모든 애니메이션과 JS 모션을 끈다. JS 없이도 내용이 다 보인다. 건너뛰기 링크, `aria-current`, 대비 높은 어두운 테마, 760px 이하 단일 열.

### 언어 선택기 (수십 개 언어 대비)

- 언어 정의는 `docs/_data/languages.yml` 한 곳: `code`(BCP 47), `name`(자기 언어 이름, endonym), `en`(영어 이름), `full`(true면 모든 ref 번역 필수, false면 `index`만 필수), 선택 `dir: rtl`. `_config.yml`에는 언어 목록을 두지 않는다.
- 헤더에는 버튼 하나만(지구본 + 현재 언어 endonym, `aria-haspopup="dialog"`·`aria-expanded`). 목록은 `_includes/lang-menu.html`이 body 끝에 한 번만 렌더 → 언어 수와 무관하게 헤더 크기 고정(더미 32개 언어에서도 헤더 HTML 바이트 동일 확인).
- 목록: 3열(모바일 760px 이하는 전체화면 시트·2열) 스크롤 그리드, endonym·영어 이름·코드로 검색, 현재 언어 `aria-current` 강조. 번역이 없는 페이지는 그 언어 홈으로 연결하고 ⌂ 표시와 title로 알린다.
- 부분 번역 언어의 내비·CTA 링크는 `_includes/url.html`(ref → 현재 언어, 없으면 기본 언어 URL)로 만든다. `llms.txt` 링크는 full 언어만 언어별 경로를 쓴다.
- 키보드: Esc 닫기, 화살표(열 수 기준, rtl이면 좌우 반전), Home/End, 검색창 Enter = 첫 결과, Tab 포커스 트랩, 바깥 클릭 닫기. `prefers-reduced-motion`이면 애니메이션 없음.
- JS 없음: 버튼 대신 `<noscript>` 링크가 네이티브 `<details id="langs">`로 이동. 자동 리다이렉트 금지 규칙은 그대로(선택만 localStorage에 기억).
- rtl: `<html dir>`이 언어 데이터에서 나온다. 화면 밖 요소에 물리적 `left:-999px` 금지(rtl에서 가로 스크롤 생김) → `inset-inline-start` 사용.
- 검증: `_src/check_site.py`가 코드 중복, 필수 필드, dir 값, 기본 언어 full, full 언어의 모든 ref·부분 언어의 index, 홈·선택기 i18n 키를 검사. 언어를 늘릴 때는 더미 30개로 빌드·검사·열린 상태 스크린샷을 찍고 더미는 커밋하지 않는다. 헤드리스 Chrome의 `--screenshot`은 rtl 페이지를 어긋나게 찍으므로 ltr iframe으로 감싸서 찍는다.

### 홈 구성: 제품 중심 (이전 결정, 홈은 이제 `index.md` + `sections:`) (미디어 플레이어 사이트 리서치, 2026-10-30)

| 사이트 | 관찰 | 가져온 것 |
|---|---|---|
| [IINA](https://iina.io/) | 한 줄 가치 제안 + 큰 앱 스크린샷, 요구사항 붙은 Download, "free and open source" | 히어로 구성, 신뢰 문구 |
| [mpv](https://mpv.io/) | 태그라인 + 6개 기능 그리드, 최소 페이지 | 짧은 기능 카드 |
| [VLC](https://www.videolan.org/vlc/) | 다운로드 버튼에 버전·OS·크기, OS별 다운로드 블록, 소스 링크 | OS별 상태·다운로드 블록 |
| [Jellyfin](https://jellyfin.org/) | 기능 그리드, 클라이언트 플랫폼 목록, 스크린샷, 신뢰 그리드(라이선스·무료·프라이버시) | 신뢰 스트립 |
| [Kodi](https://kodi.tv/) | 기능 목록, OS 목록 + Download Now, 100% 오픈소스·통계 | 플랫폼 목록 |
| [Infuse](https://firecore.com/infuse) | "A player that plays it all." + 블록별 기능, 스펙 표 | 한 줄 가치 제안 + 이중 CTA |
| [Stremio](https://www.stremio.com/) | "Available on" 플랫폼 배지, FAQ | 플랫폼 배지형 표시 |
| [Plex](https://www.plex.tv/) | 콘텐츠 카탈로그 중심 | 참고하지 않음 |

- 히어로: 한 줄 가치 제안, Download/CLI CTA, 플레이어 UI 목업(실제 스크린샷이 없으므로 "UI mockup, not a screenshot" 라벨 필수). 마스코트는 목업 속·빈 다운로드 상태에 작게만.
- 섹션: 핵심 기능 → 크로스 플랫폼 → AI-native(CLI, MCP는 계획에 없음을 명시) → 다운로드(공개 빌드 없음) → 로드맵. 모든 상태(done/partial/planned)는 `.spec` 계획과 `kinekino capabilities` 출력에 맞춘다. 라이선스·저장소 링크는 정해지기 전엔 쓰지 않는다.

### 컴포넌트 규약 (md-components v1 draft + kinekino 확장, 2026-10-30)
본문 규칙과 테마는 공개 킷 `modootoday/jekyll-theme-modootoday`(md-components v1 final)를 `remote_theme`로 쓴다(`_config.yml`, 커밋 SHA 고정; v1.0.0 태그가 나오면 태그로). 킷 파일은 복사하지 않는다: 레이아웃 `default`/`twin`, `modootoday/markdown/components.html`, 헤더·언어 메뉴·테마 토글은 테마 것. 사이트는 훅 `head-custom.html`·`footer-custom.html`, `_data/nav/main*.yml`, 섹션 확장 `_layouts/doc.html`+`_includes/site-product.html`(시각 요소·hero)만 갖는다. `sections:` 항목 i는 i번째 최상위 `##`(머리 부분은 세지 않음), 머리 부분은 `hero:`로 꾸민다. 레지스트리는 킷 형식 `_data/components.yml`, 사이트 확장은 `_data/site-components.yml`. 킷 결함·부족은 `.spec/guide/kit-gaps-modootoday.guide.md`(사본: `/workspace/shared/kit-feedback/`)에 적고 킷 저장소는 고치지 않는다.

kinekino 확장(키트에 아직 없음, 제안 상태):
- **상태 배지**: 코드 스팬 `` `status:available|partial|planned` ``을 `_includes/site-product.html`이 `span.badge.s-*`로 바꾼다. 라벨은 `_data/i18n/<lang>.yml`의 `status.*`. 어휘 밖의 값은 `check_site.py` 오류.
- **섹션 선택**: 본문은 `##`마다 잘리고(키트와 같은 `split: '<h2 '`), 앞머리 `sections:` 목록의 i번째 항목이 i번째 부분(0 = 첫 `##` 앞)의 컴포넌트를 정한다. 항목 생략 시 `prose`. en/ko의 같은 `ref`는 같은 `sections:`를 써야 한다(검사).
- **장식**: `visual: player-mock` + `scene:`, `mascot: true`처럼 앞머리 파라미터로만 켠다. 본문에는 HTML이 없다.
- **목록 제목**: `- **제목**: 설명`의 `: `는 카드에서 숨긴다(쌍둥이 .md는 원문 그대로).
- **CSS**: `docs/_sass/_tokens.scss`(색·간격·글자 크기, 다크 기본 + `prefers-color-scheme: light`)와 `docs/_sass/components/_<이름>.scss` 한 파일씩. GitHub Pages 내장 sass 변환만 쓴다.

컴포넌트 계약은 `docs/_data/components.yml`이 정본이다(파라미터, enum, 필수 여부, i18n 키, 접근성 메모). 모든 컴포넌트 예시는 `/styleguide/`(`dev: true`, `noindex`, `sitemap: false`, 메뉴·llms 목록 제외, en만)에서 본다.

| 컴포넌트 | 파라미터 | 접근성 | i18n |
|---|---|---|---|
| hero | visual(필수), scene(필수) | h1 하나, CTA는 진짜 링크 | — |
| feature-grid | cols 2/3 | 카드 전체 클릭 영역은 제목 링크 하나 | status.* |
| split | visual, scene | 그림은 figure role=img + alt | mock.* |
| platform-list | — | 목록 의미 유지 | status.* |
| timeline | — | 순서 있는 목록 | — |
| download-cta | mascot | 마스코트 aria-hidden | — |
| player-mock(visual) | scene(필수) | 내부 aria-hidden, "UI 시안" 라벨 노출 | mock.* |
| callout(키트) | — | role=note, 제목 텍스트 | callout.* |
| status-badge | — | 색 + 기호(●◐○) + 텍스트 | status.* |

리서치(2026-10-30):

| 출처 | 배운 점 | 채택 |
|---|---|---|
| https://jekyllrb.com/docs/includes/ | include 파라미터는 Liquid 전용, .md 본문에 쓰면 GitHub 화면이 깨짐 | 레이아웃에서만 include |
| https://jekyllrb.com/docs/collections/ | 컬렉션은 목록형 콘텐츠용 | 기각(페이지 수가 적고 ko/en 짝 검사와 겹침) |
| https://kramdown.gettalong.org/syntax.html | IAL `{: .x}`, `markdown="1"` | 기각(GitHub 렌더에서 글자로 남음) |
| https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax#alerts | `> [!NOTE]` 5종 | 채택(키트 include로 변환) |
| https://just-the-docs.com/docs/ui-components/callouts/ | 콜아웃을 IAL로 표시 | 기각, 표기는 GitHub 알림으로 |
| https://www.docsy.dev/docs/content/shortcodes/ | 숏코드형 컴포넌트 | 기각(본문 표준성 손상) |
| https://primer.style/ | 상태 라벨은 색만으로 구분하지 않음 | 배지에 기호+텍스트 |
| https://pages.github.com/versions/ | github-pages 232 = Jekyll 3.10, kramdown 2.4 | Gemfile 고정, 키트 검사로 확인 |

Markdown/HTML 점검(2026-10-30, `docs/**/*.md` 29개):

| 항목 | 결과 |
|---|---|
| 본문 HTML 태그, Liquid, IAL, `{#id}`, 정의 목록, `<details>` | 0 (modootoday-check) |
| 빌드 결과에 남은 `[!NOTE]` 류 | 0 |
| 빌드 결과에 남은 `<code>status:` | 0 |
| 쌍둥이 `/x/index.md` 본문 = 원본 본문 | 28/28 |
| HTML로 남은 것 | `_includes/sections/player-mock.html`, `cat.svg`, 레이아웃(본문 밖) |

검사: `python3 docs/_src/check_site.py SITE`(언어·i18n 키, hero·visual 확장, 배지 어휘, ko/en 섹션 동일, 쌍둥이, 잔여 표기, 링크, 테마 고정), 킷 `modootoday-check docs SITE --twins --strict`(섹션 수·파라미터·슬롯, 금지 문법, 쌍둥이 바이트 동일, llms). 킷 체커는 킷 체크아웃의 `exe/modootoday-check`를 `BUNDLE_GEMFILE=docs/Gemfile bundle exec ruby`로 실행.

### 내용 원칙
- CLI 페이지는 실제 빌드의 `kinekino --help`와 `kinekino-app` `entry`에서 확인한 명령만 적는다. MCP 서버는 로드맵 plan(`20261029151512-kinekino.plan.md`)에 항목이 없어 "planned"로만 표기한다.

## Record
- 2026-10-30: 시안 작성(`feat/site-mockup`). 스크린샷 `/workspace/kinekino-site-mockup/`.

## List renderers (r4)
Body Markdown stays standard GFM lists; only CSS (`_sass/components/_lists.scss`) and `site-product.html` change rendering, scoped to `.c-body` of sections other than feature-grid/platform-list/timeline/hero.

| Markdown | Detected by | Renders as |
|---|---|---|
| `ul` whose items contain a `` `available` `` / `partial` / `planned` badge | `ul:has(>li>.badge)` | bordered rows: status icon (✓ ◐ ○), text, right-aligned badge (symbol + label); badge wraps below text ≤640px |
| `ol` | element | CSS-counter stepper with rail |
| plain `ul` | fallback | rotated-square marker, spacing |
| nested `ul` | `li ul` | indent + dash marker, muted |
| `ul` whose every item is only a link | `li.c-lk`, added by site-product.html only when the item is exactly one link (Liquid: no second `<a `, nothing after `</a>`) | pill chips with → |

Fallback: no `@supports`; every `:has()` rule is a separate rule that browsers without `:has()` (before Chrome 105, Safari 15.4, Firefox 121) drop whole, so the plain, nested and ordered rules (no `:has`) still apply and lists stay readable. `[x](y) more text` stays a plain list. Stepper rail uses `--mute` (about 9:1 dark, 6.9:1 light against the section background). Badges come from the kit include with `badges=true` and are mapped to `.badge.s-*`.
