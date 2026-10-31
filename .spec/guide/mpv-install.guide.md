---
id: guide:kinekino/mpv-install
title: mpv 설치 — Unix 재생 백엔드
status: active
created: 2026-10-30
updated: 2026-10-31
---
# mpv 설치 — Unix 재생 백엔드

kinekino는 mpv 바이너리를 번들하거나 저장소에 커밋하지 않는다. `kinekino-platform-unix::mpv`는 다음 순서로 찾는다: 명시 경로(`MpvOptions::binary`) → `$KINEKINO_MPV` → `PATH`.

## 시스템 패키지(권장)

- Debian/Ubuntu: `sudo apt install mpv ffmpeg`
- Fedora: `sudo dnf install mpv ffmpeg` (RPM Fusion 필요)
- macOS: `brew install mpv ffmpeg`

## 사용자 공간 설치(Debian .deb, root 불필요)

1. `apt-get download mpv libmpv2` 및 누락 의존성 .deb를 받는다.
2. `dpkg-deb -x <pkg>.deb ~/.localdeps`로 풀고, `~/.localdeps/usr/bin/mpv`를 `LD_LIBRARY_PATH=~/.localdeps/usr/lib/x86_64-linux-gnu`와 함께 실행하는 wrapper를 `~/.localdeps/bin/mpv`로 둔다.
3. `PATH`에 `~/.localdeps/bin`을 앞에 두거나 `KINEKINO_MPV`로 wrapper를 가리킨다. `mpv --version`으로 확인한다.

## 시험

- mpv 시험은 헤드리스(`--vo=null --ao=null`)이며 ffmpeg로 생성한 4초 합성 영상(testsrc+sine)만 쓴다. 네트워크·실제 콘텐츠를 쓰지 않는다.
- mpv/ffmpeg가 없으면 시험은 skip하지 않고 실패한다.
- 검증 버전: mpv 0.40.0 (loadfile 옵션 인자는 index 인자 `-1`이 필요한 0.38+ 형식).

## libmpv(렌더 API) 고정 설치 — root 불필요

`kinekino-libmpv`는 libmpv를 실행 중 `dlopen`으로 읽는다(`$KINEKINO_LIBMPV` → 플랫폼 soname `libmpv.so.2`/`libmpv-2.dll`/`libmpv.2.dylib`). 저장소에 바이너리를 넣지 않는다. 개발 장비 기준 설치는 Debian 13(trixie) 패키지를 `/workspace/.localdeps/mpv/`에 풀어 쓴다.

| 패키지 | 주소(deb.debian.org/debian/pool/main/…) | SHA-256 |
|---|---|---|
| libmpv2 0.40.0-3+deb13u1 | `m/mpv/libmpv2_0.40.0-3+deb13u1_amd64.deb` | `62e6dca41f7b28a6d5e28ce9e15ebb7c27dae0f0b194df4715816ab0720e49a4` |
| mpv 0.40.0-3+deb13u1 | `m/mpv/mpv_0.40.0-3+deb13u1_amd64.deb` | `425c74e88f4051759b60f9d55f0947b2595d9362a5e163bdeefaf94100680259` |
| libarchive13t64 3.7.4-4+deb13u1 | `liba/libarchive/…` | `12e06195a899e8db371803ed70111fc7302573d3ed84e967d3d48ef2d543ace8` |
| liblua5.2-0 5.2.4-3+b3 | `l/lua5.2/…` | `94315c693da499bb61ee613741f286547f24c3c96dea39496ba5306cddce2087` |
| libmujs3 1.3.6-1 | `m/mujs/…` | `c0ba25b8fa478fabbeccfd97ddc2ef06f55eeb9b0274aae1f343a4fe59e10ea2` |
| libpipewire-0.3-0t64 1.4.2-1 | `p/pipewire/…` | `68522d7dff91b0f759fe58e685e19422bc2bb43dc8d6fd41f025954da09a3d0f` |
| libsixel1 1.10.5-1 | `libs/libsixel/…` | `651685e081f51e16447bdd44c4d3cded9a3f64a0993b6a65a55bc0a566bd81d7` |
| libsndio7.0 1.10.0-0.1 | `s/sndio/…` | `4ddb8b7c9cafbea65b65727dbba207e3a33c08457ef6c179b23296cb9d5b52dc` |
| libuchardet0 0.0.8-1+b2 | `u/uchardet/…` | `eb772a1649d900ac46ffeab61a8b911e81d571f88ef5dcd0eed9aad5b45b003e` |
| libva-wayland2 2.22.0-3 | `libv/libva/…` | `0ed2b236615b7dabee911e66152b3fb61572e6305811a9646cb4ef9828e3b78e` |

절차: `curl -fLO <주소>` → `sha256sum -c`로 위 값 확인 → `dpkg-deb -x <deb> /workspace/.localdeps/mpv/root` → `LD_LIBRARY_PATH=$R/usr/lib/x86_64-linux-gnu:$R/usr/lib/x86_64-linux-gnu/pipewire-0.3`(gate.sh가 설정). libmpv2 해시는 2026-10-31 deb.debian.org 원본을 다시 받아 일치를 확인했다.

시험: `kinekino-libmpv` 시험은 `vo=libmpv` 소프트웨어 렌더(`MPV_RENDER_API_TYPE_SW`, `rgb0`)와 `ao=null`로 화면 없이 돈다. ffmpeg 합성 영상(좌 빨강·우 파랑)으로 색·letterbox를 확인한다.

## CI(수동 실행 전용)

`.github/workflows/ci.yml`은 `workflow_dispatch`로만 돈다. 테스트 job은 cargo test 전에 러너에 설치한다: Ubuntu `apt-get install mpv libmpv2 ffmpeg xvfb`, macOS `brew install mpv ffmpeg`(`KINEKINO_LIBMPV`를 brew의 `libmpv.2.dylib`로 지정). 러너 설치만이며 저장소·배포본에 바이너리를 넣지 않는다. 도구가 없으면 시험은 실패한다.

## 플레이어 창 스모크

`kinekino watch <file> --smoke --capture out.png`: 창을 열고 영상 프레임이 올라오면 PNG를 쓰고 닫는다. 화면이 없는 Linux에서는 `xvfb-run -a`로 감싼다.

### 플레이어 창 동작

- **파일 열기:** Ctrl+O(macOS Cmd+O). 물리 키 기준이라 한글 입력 배열에서도 동작하고, Alt 조합·키 반복은 무시한다.
- **컨트롤 자동 숨김:** 전체 화면에서 재생 중이고 패널·상태 메시지가 없으면 입력이 2.5초 없을 때 컨트롤을 숨긴다. 포인터·버튼·키·휠 입력으로 다시 보인다. 일시 정지·미디어 없음·오류 상태에서는 숨기지 않는다.
- **자막 오버레이:** mpv 자체 자막 렌더링은 끄고(`sub-visibility=no`), libmpv `sub-text` 속성을 읽어 현재 자막(최대 두 줄)을 영상 위 가운데에 Meari UI로 그린다. 에피소드를 바꾸면 지운다.

### 라이선스 고지

mpv 소스는 주로 GPL-2+/LGPL-2.1+이고, Debian 바이너리는 GPL-3+ 라이브러리(libsmbclient)와 링크되어 GPL-3+로 배포된다(`usr/share/doc/libmpv2/copyright`). kinekino(MIT)는 libmpv를 배포하거나 정적 링크하지 않고 사용자가 설치한 라이브러리를 실행 중 읽기만 한다. 배포본에 libmpv를 동봉하려면 별도 라이선스 검토(LGPL 빌드 선택 포함)가 필요하며 사용자 승인 전에는 하지 않는다.
