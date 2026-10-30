---
id: plan:kinekino/20261029113952
title: Git 기준선과 공개 재현성
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Git 기준선과 공개 재현성

사용자 후속 지시: 파일별 allowlist 확대는 최종 정책으로 삼지 않는다. 공개 전용 source/tests/contracts/assets와 비공개 evidence/runtime/model을 디렉터리 책임으로 분리하고, 공개 전용 디렉터리를 허용하는 짧은 ignore 규칙으로 전환한다. 기존 exact 목록은 현 체크포인트 inventory 근거로 보존한다. 이미 공개 전용인 src는 규칙만 단순화할 수 있으며 불필요한 이동을 하지 않는다. 실제 경로 재배치는 담당별 include/fixture/링크 영향 검토와 현 UI 입력 반환 후 단계별로 수행한다.

2026-10-20. 사용자 지시: 맥락·단계별 커밋 및 최소 공개면. PM은 root index를 직렬 배정하고, Meari는 자체 index를 소유한다. push는 하지 않는다. 과거 미커밋 작업의 성공 이력을 재구성하지 않고 현 검증 기준선을 기록한다.

## 인수한 기준선

| 저장소 | 커밋 | 범위 |
|---|---|---|
| root | `0bbf496` | 프로젝트 지침·명시 공개 추적 정책·텍스트 속성 |
| Meari | `473a57f` | 공개면·NOTICE·Paint 라이선스 |
| Meari | `d0eb687` | 현 검증 source344 중 변경289파일 기준선 |
| Meari | `2c08832` | 공개 API·지원 범위·남은 제한 문서 |
| root | `1d4fac9` | Meari gitlink의 로컬 기준선 인수 |
| root | `66728a9` | media-contracts/file-publication 현재 기준선 |
| root | `a671e7b` | media-subtitles/jobs 현재 기준선; S2 강한 취소·cleanup 보장은 미완료 |
| root | `d240ada` | media-catalog 현재 기준선; 기존 manifest EOF 공백 보존 |
| root | `8f7df67` | UI 선언61·SVG66·plugin11 exact 허용; 미래 SVG wildcard 제거 |
| root | `cf3279a` | Rust UI plugin Cargo/Rust11 현재 기준선 |
| root | `e82f753` | 필수 자산78개 및 pinned asset bytes 보존 속성; 원 OFL whitespace 보존 |
| root | `1f4c0fc` | 공개 전용 source/tests/package src/자산 디렉터리 허용으로 ignore99줄 단순화 |
| root | `d85112e` | 디렉터리 기반 공개 정책을 최상위 지침에 반영 |
| root | `541bf69` | presentation 현재223 기준선; Inspector draft3 등록은 미컴파일·미검증으로 구분 |
| root | `2232b67` | proposal/conversion 독립 공개 계약4파일; 새 identity 미검증 |
| root | `a40b718` | root SDK/CLI359+필수 legacy 자산5 현재 기준선; 수정 후 기능 미검증 |
| root | `6d9f666` | bridge 공개 소스/테스트28 현재 기준선; 추가 private compile 계약5곳 수리 전 |
| root | `1fd8ca1` | bridge document-session/intent-dispatch 독립 계약과5참조8파일 수리; 검증 대기 |
| root | `2efd2f1` | model-review-bundle 도구20파일 기준선; 고정6d9f666 의존 snapshot12pass/1ignored·strict/bin 검증, 실제9/품질 승인 별개 |
| root | `c175c4a` | 합성 ReviewPackage 시험의 동일값 slice 참조3곳 최소수리 |
| root | `abc304e` | 공개 baseline에 빠진 player-ui-plugins 통합시험2파일 복구 |

Meari HEAD clean을 PM이 확인했으며 담당의 source344 재해시 delta[]를 인수했다. 새 submodule 커밋은 미push이므로 원격 재현 완료가 아니다. root 제품 소스는 아직 대부분 미추적이며 fresh clone 빌드/시험 성공을 주장하지 않는다.

## 진행 순서

1. SDK가 media-contracts/file-publication, media-subtitles/jobs, media-catalog의 exact Cargo/Rust 목록으로 현재 기준선을 커밋한다. 별도 검증 성공을 전체 제품 재현으로 확대하지 않는다.
2. root application/domain/infrastructure/조립·CLI와 UI 기준선의 필수 의존 closure를 합의한다. 기존 exact 목록은 현 inventory로 보존하되, 공개 전용임을 확인한 소스 디렉터리는 전체 허용한다. 비공개 자료는 별도 디렉터리에서 제외하며 새 파일마다 예외를 늘리지 않는다.
3. 비공개 fixture 의존을 Rust의 공개 synthetic fixture와 명시 로컬 통합 검증으로 구분한다. 기존 실제 미디어 관측은 보존하고 공개 예제로 그대로 복사하지 않는다. 현재 UI heavy 입력을 바꾸지 않고 후속 SDK 슬롯에서 수리한다.
4. 최종 추적 목록에 비밀정보·개인 미디어·runtime/model/cache/scripts/.spec/.agents가 없는지 확인하고 필수 fmt/test/clippy 및 격리된 공개 입력 재현을 담당이 검증한다. 새로운 cold build는 중앙 슬롯 배정 뒤 수행한다.

## 실제 발견된 재현 차단

컴파일 시 비공개 `.spec` 파일을 읽는 테스트:

- `src/application/ai_provenance/integration_tests.rs:39,42`
- `src/assembly/meari/async_proposals/tests/policy.rs:333`
- `src/infrastructure/historical_evidence/tests.rs:64`
- `src/infrastructure/proposals/tests/historical.rs:30,33`
- `tests/model_review.rs:148`

`src/application/workspace_read/preview_mask.rs:122`는 아직 공개 allowlist에 없는 demo mask SVG를 읽는다. PM이 소스의 include 경로를 직접 확인했다. 현재 파일이 존재해 로컬 시험이 통과한 사실과 공개 checkout 재현성은 별개다.

UI 담당의 exact source61/SVG66 목록은 `runtime/quality-review/studio-frame-sdk-r1/git-resource-review.md`와 인접 inventory에 있다. 추가 compile 의존: `src/assembly/meari/proposal_manifest.rs`와 `tools/player-ui-sdk-bridge/src/lib.rs`의 private `.spec` 계약 include. 필요한 공개 계약 리소스를 명시 경로로 분리하고 `.spec` 전체를 허용하지 않는다. 사용자 지시로 제작한 synthetic PNG4는 제품의 직접 include 자산이며 provenance/hash와 함께 정확 파일 단위로 검토한다. 단순히 추가 형식적 승인이 없다는 이유만으로 필수 자산을 영구 제외하지 않으며 실제 영상·미선택 파일과 구분한다.

## 공개 계약과 테스트 입력 분리

정확 변경 범위, 독립 계약 4종, actual/mock/review fixture 구분, 커밋 근거와 남은 검증은 [공개 계약 분리 계획](public-contracts-split.md)에서 관리한다. 공개 재현성 검사는 고정된 tracked snapshot 입력으로 수행하며 미검증 변경을 기존 검증 성공으로 표현하지 않는다.

## 남은 게이트

PM은 디렉터리 규칙의 공개 신규 UI/아이콘/선택 패키지 Rust/도구 Rust 허용과 private `.spec`/runtime/scripts/unknown package/tool downloads/.env/미선택 demo v1 거부를 `git check-ignore`로 확인했다. 별도 제품 빌드는 하지 않았다. 데모 anime 디렉터리에 선택·미선택 버전이 혼재해 PNG4 예외는 분리 전까지 유지한다. source inventory 목록은 공개 승인 규칙이 아니라 체크포인트 근거다.

- [x] SDK media 패키지 커밋·index 반환 인수(59파일 해시 차이0 담당 인계, PM log/index 확인)
- [ ] 필수 플러그인/TSX/CSS/route/자산 최소 공개 목록 확정
- [ ] 비공개 compile fixture의 공개 synthetic 분리 및 실제 로컬 통합 검증 보존
- [ ] root 제품 기준선 커밋·필수 검사·공개 입력 재현
- [ ] 사용자가 별도 요청할 경우에만 원격 push와 submodule 원격 가용성 검증
