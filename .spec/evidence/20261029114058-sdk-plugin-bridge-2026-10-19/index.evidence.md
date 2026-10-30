---
id: evidence:kinekino/20261029114058
title: 실 SDK·파일 저장소의 제품 플러그인 연결 r1
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실 SDK·파일 저장소의 제품 플러그인 연결 r1

2026-10-19 PM 배정 슬롯에서 tools/player-ui-sdk-bridge를 검사했다. 제품 host 한 벌 `src/assembly/meari/subtitle_conversion.rs`를 probe가 그대로 컴파일하고 실제 ApplicationClient→제품 PluginAdapter→Meari registry→SDK SubtitleFiles를 연결했다. root 제품 모듈/Cargo 등록은 변경하지 않았다.

## 명령과 보존

manifest는 tools/player-ui-sdk-bridge/Cargo.toml이며 offline/공유 target/-j1/BelowNormal을 유지했다. 초기 테스트는4/4였으나 rustc1.99.0의 AtomicU64::fetch_update 폐기 경고가 있었다. 정상 종료 후 컴파일러 안내대로 try_update 한 줄을 수정하고 최종 소스를 재검사했다. 초기 원로그와 수리 후 로그를 분리했고 수리 전 전체 lib.rs byte snapshot 미보존 사실은 checkpoint repair-note.md에 명시했다.

| 최종 검사 | 결과 |
| --- | --- |
| cargo fmt --check | exit0 |
| cargo test --offline --locked -j1 | exit0,4/4,11.40초 |
| cargo clippy --offline --locked --all-targets -j1 -- -D warnings | exit0,1분10초 |
| cargo build --offline --locked -j1 | exit0,1.55초 |

초기 의존성 compile4분53초를 포함한 건강한 프로세스를 중단/재시작하지 않았다. 전체 종료 뒤 슬롯을 반환했다. `runtime/quality-review/player-ui-sdk-bridge-r1/`에 logs, lock-seed, source-final, fixed binary, CLI fixture/입출력·SHA를 보존했다. 고정 exe SHA256은 `cd41c1218567a9864b8c13e3499192800418cf78e6613f096f6192189fd3745f`다.

root lock의 읽기 seed 전·복사 후·해결 중·최종 SHA는 모두 `0cf44760431b804cdf18afd50aa82f57b50579cfe1465f9f6d6e9ee7b1aa96cc`였다. 독립 probe lock은 offline으로 실제 해결한 뒤 `79068df012f747587d24dad4b72806062f79b7aab14e29d650f5de41cfda0725`이며 이후 locked 검사를 사용했다. 원본 root lock을 수정하지 않았다.

## 실제 SDK 시험4개

실제 임시 파일을 사용하는 SDK client에서 새 VTT publication·원본 SRT 불변·기존 출력 충돌과 출력 불변을 확인했다. VTT voice 손실은 direct client와 plugin 경유 ApiError 전체가 같고 동의 전 출력 없음/명시 allow_loss 뒤 새 SRT 출력을 확인했다. SDK의 미지원 idempotency context를 plugin이 제거하지 않고 Unsupported로 전달하는 것도 확인했다.

동시에 살아 있는 두 host와 종료 후 재생성 host는 서로 다른 instance/registry namespace와 SDK request ID를 가진다. canonical SDK context validation 후 caller namespace+전체 ticket으로 bounded request ID를 만들며 revision/idempotency/timeout은 그대로 전달한다. scope StateProgram은 Bridge 수명 동안 보유한다. root SDK/CAS/변환 정책을 probe나 plugin에 복제하지 않았다.

## 고정 CLI 실제 파일 결과

동일 고정 exe에 checkpoint의 canonical JSON request를 전달했다. renderer_verified/browser_verified는 모두 false다.

| 요청 | 실제 결과 |
| --- | --- |
| 한국어 SRT→새 VTT | exit0, cue1/55bytes/UTF-8/overwrite=false, 실제 파일 생성 |
| 같은 출력 경로 재요청 | exit5, RevisionConflict, 기존 출력 byte/SHA 불변 |
| voice 손실 VTT→SRT, allow_loss=false | exit3, NeedsInput·손실 목록/원래 remediation 보존, loss-output.srt 없음 |

SRT 원본 SHA는 `a69cc02829e5b0c023bd925fd1421f4d62950508649f6a75be595488430ab3da`, VTT 원본은 `2f4189e60928ef3fc8a299bc3c98ee268ae806105d80f1ae11d4c25080adbfc9`로 전후 동일했다. 출력 VTT의 충돌 전후 SHA는 `abc5fe916913139dcb29201768144f8c904f4aa19b5cbfb13bbbb24874d91d97`로 동일했다. SDK 오류의 request ID는 실제 ticket/host별로 구별됐다.

## 판정의 한계

이 결과는 실제 SDK/파일 저장소와 제품 플러그인/공유 host 연결을 입증한다. 제품 root 등록·TSX callback/source IR·Native GUI·Web bridge/권한/브라우저·자산·시각 인수는 아직 아니다. provider/schema/contract pin은 probe가 정본 schema/source bytes에서 만든 metadata이고 production trust 검증을 뜻하지 않는다. 다음은 기존 실제9 산출물의 원문 비교→사람 수정→명시 채택을 같은 host/plugin 경계와 공통 Client로 소비하는 제품 구현이다. provenance 영속 API가 없는 필드는 unknown/unbound로 유지하고 품질 승인/자동 적용으로 표시하지 않는다.
