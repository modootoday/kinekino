---
id: sot:kinekino/media-card-render-r4
title: media-card-render r4 공개 소스 채택 검토
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# media-card-render r4 공개 소스 채택 검토

2026-10-24, Media 담당. PM의 읽기·문서 전용 배정. 결론은 **소스 후보의 공개 경계는 확인했으며, 패키지 자체 라이선스 결정과 배포물별 의존성 고지는 미완료**다. 공개 채택·재배포 승인이나 crates.io 게시를 완료로 판정하지 않는다.

## 범위와 고정 입력

- 후보: `runtime/development/accepted-card-png-r4/raw/packages/media-card-render`의 정확18개 파일 = Cargo.toml 1 + Rust 17. 재귀 파일 수와 source18 전 항목 SHA256/길이를 재확인했다. 추가 파일0.
- source18: `BF39A0A8CE6D98FED9EB4D820CED4771B9ADD988E64CF4BEC0DF405B8FDB6CAE`.
- r3 parent18: `F62852CC275215AAE03ED8C75F0CA34514CA13C7A054ACEDC66393D9408F773B`; r4 변경은 tests.rs 1leaf의 Clippy 구문 수리. 변경 내용은 changes1.json과 HANDOFF.md 참조.
- 로컬 근거34개 경로·바이트·SHA256: [동일 책임의 evidence](public-source-review-2026-10-24.evidence.json). Cargo.lock은 관측 시점 파일이며 새 resolution·Cargo metadata·컴파일은 수행하지 않았다.

## 소스·fixture·개인 자료 경계

| 주장 | 판정 | 실제 근거와 한계 |
|---|---|---|
| 후보18은 runtime·바이너리·폰트를 포함한다 | 반증 | exact inventory는 Cargo manifest와 .rs뿐이다. target, 실행파일, 모델, 폰트, 미디어, lockfile, credentials, 운영 스크립트 없음. 후보가 runtime/development 아래 있는 것은 보관 위치이며 그 상위 runtime 전체를 공개할 근거가 아니다. |
| 테스트는 실제 개인 영상·자막에서 추출했다 | 소스에서 관측되지 않음 | tests/fixtures.rs의 frame/font bytes는 b"frame", b"font-regular" 등의 명시적 stub; 문구는 앞면 원문/뒷면 원문, 고정 fixture ID. tests/native_png.rs:138–150은 상수 RGBA16바이트로 2×2 PNG를 메모리 생성하고 :163–164는 짧은 합성 한국어·일본어 문구를 사용한다. 개인 미디어 파일을 포함하거나 읽는 코드가 이 후보에 없음. 일반적인 저작권·개인정보 전수 보증은 아니다. |
| 모든 폰트가 독자 작성 fixture다 | 반증 | native_png.rs:21–66은 외부 common font root에서 Pretendard 3개 TTF와 LICENSE 2개를 읽어 고정 해시를 확인한다. :134–137은 ignored test와 MEDIA_CARD_RENDER_FONT_ROOT 명시 입력; 절대 경로 필수, 자동 다운로드 없음. 폰트 bytes는 source18에 없다. stub의 "fixture metadata only; not a real font"와 실제 OFL 폰트를 구별한다. |
| renderer는 외부 프로젝트 코드를 복제한 묶음이다 | 확인되지 않음 | 이 세션에서 작성한 r1→r4 책임별 Rust 모듈·합성 fixture이며 native.rs는 공개 Meari API 호출, png_encoding.rs는 png crate 어댑터, markup.rs는 직접 작성한 템플릿/escape 구현이다. 외부 소스 파일·코드 주석의 upstream 복제 고지는 후보18에 관측되지 않는다. 인터넷 유사도 조사·권리 양도 증명은 수행하지 않았으므로 독창성/권원 전체를 인증하지 않는다. Meari 내부의 명시적 upstream fork는 아래 별도 경계다. |
| 비밀정보·개인 실행 경로가 공개된다 | 제한적 확인 | source18에서 include_bytes!/include_str!, process Command, private-key header, api-key/password/secret/access_token/Authorization 패턴 검색은 일치0. native 테스트가 받는 환경변수는 공통 폰트 root 1개다. tests.rs의 example.invalid URL은 HTML escape 공격 fixture로 assert된다. 이 정적 패턴 검사는 일반적 비밀 전수 탐지 보증이 아니다. |

application/assets.rs의 font.license.is_none() 검사는 **라이선스 문자열 존재** 조건일 뿐 법적 권리 검증이 아니다. 실서비스의 frame/문구/폰트는 host가 전달하는 asset이며, source-only 합성 fixture 검토로 향후 사용자 콘텐츠 재배포 권리를 승인하지 않는다. native 시스템 폰트 검색도 Meari NOTICE에서 별도 명시된다.

## 의존성과 기존 notice 소비

| 의존 경계 | 로컬 파일에서 확인한 선언·출처 | 소비/미확정 경계 |
|---|---|---|
| meari / optional meari-native | 각각 Cargo.toml에 MIT OR Apache-2.0 및 repository https://github.com/modootoday/meari. 각 패키지 LICENSE-MIT, LICENSE-APACHE, NOTICE 존재; include 목록에도 포함 | 상위 Meari의 선언을 새 renderer의 자체 라이선스로 상속한다고 추정하지 않는다. 실제 배포 시 선택한 라이선스에 맞는 고지와 포함물 확인 필요. |
| Meari native 전이 의존 | 기존 NOTICE가 Stylo/traits MPL-2.0, AnyRender/Vello CPU, optional GL의 외부 Skia, system-fonts를 구분 | native=false 기본 경로와 native=true 배포물의 포함 범위는 다르다. 모든 전이 소스/정적 링크/플랫폼별 고지를 이 소스 검토에서 인수하지 않았다. |
| Meari paint fork | packages/meari/packages/meari-paint/README.md 및 NOTICE: blitz-paint 0.3.0-beta.4, DioxusLabs/blitz revision 92b11f1bd8ab7975411328b0406a95a09445b4e7; 원 MIT/Apache 텍스트 유지, Meari 변경 명시 | 외부 복제 소스는 Meari 책임의 명시적 fork다. renderer18 안에 재복제되지 않았다. Meari 수정 이력·notice를 제거하거나 renderer 저작물로 재표시하지 않는다. |
| png | 후보는 =0.18.1 고정. 로컬 registry png-0.18.1/Cargo.toml: MIT OR Apache-2.0, repository https://github.com/image-rs/image-png; 두 license 파일 확인 | 소스를 vendor하지 않고 Cargo 의존으로 소비. 의존 crate의 라이선스가 renderer 자체 라이선스를 결정하지 않는다. |
| serde / serde_json | 후보는 각각 version="1". 관측 root/Meari Cargo.lock은 serde1.0.229, serde_json1.0.154. 해당 로컬 registry Cargo.toml은 MIT OR Apache-2.0, repository https://github.com/serde-rs/serde 및 https://github.com/serde-rs/json, 두 license 파일 확인 | 넓은 semver 선언이므로 향후 실제 SDK lock 선택이 다르면 다시 확인해야 한다. 후보 자체에 lockfile은 없다. serde_derive 등 전체 전이 그래프 검토 완료를 뜻하지 않는다. |
| media-contracts (dev) | 로컬 Cargo.toml의 publish=false, license 없음 | 프로젝트 내부 계약 crate. 테스트 소스를 공개할 경우 해당 소스/라이선스 정책도 프로젝트 결정과 일치시켜야 한다. Meari의 dual license를 임의 적용하지 않는다. |
| Pretendard / JP 외부 fixture | assets/fonts/pretendard/LICENSE.txt 및 LICENSE-JP.txt는 SIL OFL1.1, Kil Hyung-jin 저작권 및 Reserved Font Name을 명시. 두 파일 해시는 native fixture의 pin과 일치 | exact18은 폰트/고지 파일을 새로 번들하지 않는다. 폰트 재배포 시 공통 자산 계약·원 고지·원 이름 조건을 별도로 유지한다. 시스템 대체 폰트와 렌더 결과의 권리 전체를 승인하지 않는다. |

기존 Meari NOTICE는 의존성의 권리·라이선스를 유지하고 실행파일 번들의 완전한 의존성/폰트 검토를 별도로 요구한다. renderer에 기존 NOTICE를 단순 복사해 모든 의존성을 포괄했다고 주장하지 않는다. 이 검토는 로컬 upstream metadata 관측이며 최신 외부 법령이나 법률 적합성에 대한 의견이 아니다. 네트워크 조회0.

## 필요한 결정과 소유권

1. **사용자/프로젝트 권리자의 자체 라이선스 결정 필요.** renderer Cargo.toml에 license/license-file 없음, exact18에 LICENSE/NOTICE 없음, root Cargo.toml에도 license 없음. publish=false는 registry 게시 제한이며 재배포 허락의 선언이 아니다. MIT/Apache 등 임의 선언은 하지 않았다. PM이 프로젝트의 승인된 라이선스 방침·권원과 연결해 확정해야 한다.
2. **PM은 공개 채택/최소 공개 경계와 root index를 소유.** source18에 해당하는 책임별 공개 디렉터리만 검토하며 runtime·.spec 전체 허용으로 확대하지 않는다. 이번에는 .gitignore/manifest/index/stage/commit/push 변경0.
3. **SDK는 실제 조합 lock/feature와 통합 검증을 소유.** 이 문서의 후보 고정성과 runtime/모델 제외 확인은 Cargo 시험 성공·SDK 기능 인수·GUI 시각 인수의 근거가 아니다. SDK sole-heavy 유지, 새 컴파일·시험 실행0.
4. **Meari/공통 자산 담당은 각 기존 고지·fork·폰트 provenance 소유.** 공개 소스 채택과 별개로 바이너리 배포 시 실제 포함물별 고지를 완성해야 한다. 전체 SBOM·전이 라이선스 closure·법적 권원 인증은 미완료로 남긴다.

## 스킬 적용과 검증 제한

modoo.today 도메인 탐색을 수행했으나 전자상거래·개인정보·크롤링 법률/시각 asset QA 결과는 이 소스 검토에 부적합하여 채택하지 않았다. 직접 관련된 `modootoday/software-qa/code-change-review-verification` 본문과 이전에 읽은 필수 `references/verdict-table.md`를 적용해 요청된 주장을 actual path/관측/미확인으로 구분했다. 본문 SHA256 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B, 참조 SHA256 9D9619A829AE24D73645EF5F7C8126C8257CAEDDAC80800632866257008AD2A9. 새 독립 보안/법률 감사를 추가하지 않았다.

검증은 로컬 파일 읽기·정적 검색·SHA/길이 재확인과 문서 작성에 한정했다. source18·원 실패 증거·Cargo·SDK 조합·공개 경계는 변경하지 않았다. 상세 source-only 기술 인수는 기존 r4 HANDOFF를 유지한다.
