---
id: evidence:kinekino/20261029114817
title: SDK 공개 입력 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SDK 공개 입력 검증

2026-10-20 관측. [분리 계획](../../../plans/player/public-contracts-split.md) 및 [공개 경계 결정](../../../adrs/player/adr-public-source-and-contract-boundaries.md)과 구분한다. 전체 제품 완료나 모든 기능 조합의 검증이 아니다.

## 고정 입력

실제 cwd는 저장소 밖 `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-sdk-1fd8ca1`이었다. bridge는 그 아래 `tools/player-ui-sdk-bridge`를 cwd로 사용했다. shared checkout의 private 자료를 상대 cwd로 읽는 실행이 아니다. 처음 snapshot에는 `.spec`과 runtime이 없었고 시험 중 synthetic runtime 출력만 생성했다.

| 입력 | 식별 |
|---|---|
| root archive | `1fd8ca1849eb0fd0012f478fa6acf98a43595476`, SHA `4BC3C8C074015E6CF6949E604886B2D2735D700787AC6118D3FE00A8C6987C0F` |
| 원 Meari archive | `2c08832`, SHA `EB667C7BDEE7FD5240ECD113750AFA5DA1E5B4857E2DDD1EF64EFE615747D6DB` |
| overlay 1 | `c175c4a`: tests/model_review.rs의 불필요한 slice3개 수리 |
| overlay 2 | `abc304e`: plugin 통합 tests2파일 복구 |
| overlay 3 | `cf8bc9b`: bridge actual R3의 명시 runtime/opt-in 분류1파일 |

최종 overlay는4파일이며 `runtime/quality-review/sdk-public-input-r1/input-checkpoint.json`에 개별 SHA를 기록했다. 기록 파일 SHA는 `CF7C009DD0FA3058352CD56C4B397C30D15F9086DD4D7F93DDA3DF151E5AB3EB`, 마지막 overlay 재해시 차이는0이다. tracked Inspector3는 포함했고 미등록 OBJ7·격리 Meari P1b는 제외했다.

실행 runner SHA는 `692C22751FAC7F9999F2D833571DAB6605AFD58E66711370494E62DD8A62C463`다. 최초 경로와 지정 frame-document-cli-r1 경로의 binary가 동일함을 해시로 확인했다. CPU affinity3(논리 CPU2개), BelowNormal, descendant Job 적용을 receipt에서 확인했다. root/bridge target 캐시를 분리했고 offline·locked·j1·test thread1을 유지했다.

## 결과와 원 실패

| 관측 범위 | 결과 | 시간 ms |
|---|---|---:|
| 기본 root fmt | 종료0 | 11025 |
| 기본 전체 test | 435 pass / 23 ignored, 종료0 | 520329 |
| 최초 전체 strict | redundant_slicing3곳, 종료101 | 33492 |
| 최소 수리 후 전체 strict | 종료0 | 30933 |
| 최종 형식 model_review 집중 | 8 pass, 종료0 | 19413 |
| 복구한 plugin2target | 6 pass, 종료0 | 34058 |
| 복구한 plugin2target strict | 종료0 | 5519 |
| 복구·수리 후 root fmt | 종료0 | 2531 |
| native feature historical 필터 | root9 pass / 2 ignored, 종료0. 다른 workspace의 일치 필터 범위도 통과 | 212136 |
| native feature lib/bins strict | 종료0 | 22147 |
| bridge 선택6target native feature | 22 pass / 1 ignored, 종료0 | 449345 |
| bridge 같은6target strict | 종료0 | 53401 |
| bridge fmt | 종료0 | 3253 |

최초 strict 실패 원본은 보존했다. 수리는 동일 값의 slice 표현만 정리했으며 정책·assertions는 유지했다. strict 재검사는 포맷 전 동등3표현 수리였고, 최종 형식은 관련8회귀와 fmt로 확인했다. 최종 review 파일 SHA는 `A66D4F644C05423B46D2CB78B38C1428D703481056E51AC8B364360D81FA3A23`다.

이전440과 최초435의 차이는 plugin tests2target 각3개가 미추적으로 snapshot에 누락된 것과 새 입력 거부 회귀1개 추가다: `440 - 6 + 1 = 435`. 원 파일을 추적해 해당6개를 복구했다. 기본435와 복구6의 고유 범위는441이며 한 번에441개를 실행한 기록이 아니다. model_review8 및 반대 feature 재검증을 새 고유 시험으로 더하지 않는다. 패키지 공개 closure는 src·tests·Cargo 발견 target을 함께 대조해야 한다.

Bridge 대상은 `document_repair_sdk`, `native_document_sdk`, `original_review_sdk`, `proposal_view`, `callback_sdk`, `sdk_files`다. 전체 bridge off/on suite가 아니다. 실제 private R3 자료를 쓰던 기존 테스트는 고정 package/item과 assertions를 보존한 채 ignored opt-in으로 분류했고, 절대 `R3_REVIEW_WORKSPACE`를 요구한다. 이1파일은 native historical 실행 종료 뒤 snapshot에 반영했으며 SHA는 `E56932C69A460B91DA9481A7D2242DE3C6B1720B9E98B859B1B6EDA1FB606324`다.

## 종료와 한계

원 exec들은 모두 terminal이며 13receipt의 launcher/child PID가 마지막 확인에서 모두 부재했다. 각 index 배정은 staged0으로 반환했고 single heavy를 PM에게 명시 반환했다. 원로그·receipt·archive·핀은 비공개 `runtime/quality-review/sdk-public-input-r1/`에 보존했다.

actual9·actual R3·실사이트·OS 창·새 AI 추론은 이번 gate에서 실행하지 않았다. R3 누락·상대 경로 거부는 현재 코드 guard이며 별도 실제 opt-in 실행은 미검증이다. 결과는 지정 공개 개발 입력의 기본/집중 검증이며 전체 제품 기능·전체 GUI 시각 일치·모델 품질·원격 push/clone 가용성·후속 P3 소스의 검증을 입증하지 않는다.
