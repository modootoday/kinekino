---
id: evidence:kinekino/20261029114243
title: 채택 출력 판정 SDK/wire/fresh CLI 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 채택 출력 판정 SDK/wire/fresh CLI 검증

2026-10-20. [계약14](accepted-output-review-contracts-2026-10-20.md)·[저장12](accepted-output-review-storage-2026-10-20.md) 이후 [계획](../../../plans/player/accepted-output-review-sdk.md)의 exact18 연결을 검증했다. configured Windows ProposalFiles의 공통 SDK/typed client/CLI에서 새3명령의 아래 동작을 확인했다. 합성 deferred 판정이며 사용자 품질 승인·모델 성능 성공·미디어 적용은 아니다.

## 입력과 버전

snapshot은 `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1`이다. Meari1df381/UI9r2/overlay22r2, SDK계약14 c494·저장12 cd6/r2와 기존 SDK/manifest/assets를 보호했다. `runtime/quality-review/sdk-output-review-wire-r1/`에 신규7 부재·기존11의 원 bytes/hash를 보존하고 exact18만 적용했다. 이번 경로와 중첩된 기존핀은 before18/final18로 추적하고 나머지는 보호핀으로 유지했다.

r1 pin은 `55E7D2D73DBF24CF021490E49B6A8BBBB12AAA30EDA9FA5565C73A42499127EF`다. r1 unit4/2 통과 후 CLI fixture 호출 기록에 Cargo의 실제 executable 경로만 추가한 r2 pin은 `9BB859EBC056EC110BFBBEA1BBC7BCFE9B60F366A90C35C1CD636301A27996F1`이다. CLI10 통과 뒤 unused ApplicationClient test import2줄만 제거한 r3 pin은 `3EBC2F908DBAD95BFD6E499C5808A774E5120C94A72AFF8C98B79763E70EBE7C`다. 각 `runtime/coordination/sdk-output-review-wire18-r{1,2,3}/`에 원본과 핀을 별도 보존했다.

r2는 증거 metadata, r3는 import-only 변경이며 제품 동작/시험 단언은 같다. 원 성공로그와 경고를 보존했고 import-only 수리 후 동일 시험 성공을 재합산하지 않았다. 최종 r3는 양feature all-target strict/fmt로 컴파일·lint했다.

## 원실행 결과

| 검사 | 결과 | receipt elapsed_ms |
|---|---|---:|
| 새 wire unit | 4 passed | 47782 |
| 새 schema unit | 2 passed | 1113 |
| fresh CLI r2 | 10 passed, 0 failed/ignored, 별도96호출 | 81449 |
| 변경 저장/session 회귀 | 16 passed | 4492 |
| 기존 proposal/historical | 7 passed | 1443 |
| 기존 proposals CLI/schema/caps | 1 passed | 19645 |
| 기존 candidate schema | 1 passed | 11114 |
| 기존 candidate CLI inspect/record/readback | 1 passed | 3793 |
| 최종 default all-target strict | 0 | 29892 |
| 최종 native feature all-target strict | 0 | 30368 |
| 최종 fmt --check | 0 | 1375 |

절대 snapshot cwd/manifest guard, offline locked-j1/threads1/affinity3/논리CPU2/BelowNormal/descendants job으로 순차 실행했다. canonical Rust runner SHA256은 `692C22751FAC7F9999F2D833571DAB6605AFD58E66711370494E62DD8A62C463`다. 원실행11 모두 terminal0/launcher-child22 PID 부재를 확인했다.

## 실제 CLI provenance와 범위

`cli-evidence-r2/`에 case10개의 store와 호출별 request/response/exit/rooted/executable을 보존했다. 매 호출 새 process를 spawn/wait_with_output으로 회수했다. `cli-call-summary.json`과 nocapture 로그의 합계는96이다. 시험10·CLI96회·세 task를 독립 품질표본으로 합산하지 않는다.

Cargo 실제 executable 경로는 `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/native/debug/player-cli.exe`이며 원 JSON의 separator 차이도 보존했다. 해당 snapshot의 kinekino-player compile 로그와 바이너리 SHA256 `45B6BEB2F0045357A2D9FD55253DC138C687B08C8DBA6F47F569D9CBCE6CF900`/28187136bytes/mtime UTC를 `cli-binaries.json`에 기록했다. 후속 기존CLI/strict/fmt 마감 뒤에도 hash가 같다.

10사례는 세 task 정상 target/기준 저장·fresh history, 동일 hash 다른 proposal, edit/reaccept stale·acceptance 부재, caller 기준 교환, hash version/publication 변조, source/generated/companion 교환, canonical publication 손실과 독립 history, 동일ID retry/충돌, declared/claimed/observed·caller clock 주입, discovery/schema/context·legacy 경계다. source 손실은 canonical publication 파일 누락/변조이며 외부 historical 파일 손실의 CLI end-to-end 결과가 아니다. explicit pinned verifier 후속과 ignored actual-nine은 실행하지 않았다.

inspection은 동일 session state/target/trusted 전체criteria와 범위 있는 관측을 반환한다. historical 없음은 evidence 검증 성공이 아니다. record만 현재 output expected_revision을 요구하고 decision_id를 도메인 중복키로 쓴다. transport idempotency/timeout은 Unsupported다. history는 current evidence를 재검증하지 않고 NotChecked 원기록을 읽는다. optional None store는 capability=false다.

acceptance 검사를 session open에서 usecase로 옮겼지만 옛 prepared append 거부·stale conflict·현재 acceptance 없음 NeedsInput·원record 불변을 확인했다. legacy schema/응답은 보존했고 global discovery 전체 bytes는 새 command/capability 때문에 달라진다. 전체 cargo test·사용자 의미 품질·외부 historical actual-nine·GUI/시각/미디어/모델 추론·전체 제품 지원은 이번 범위 밖이다.

최종 r3 frozen/live/snapshot18 차이0·비중첩 보호 차이0·CLI binary hash 차이0이다. final-source18/final-protected-differences=[]/final-owned=[]를 보존했다. index/stage/commit/push0이며 heavy/snapshot을 PM에 명시 반환했다.
