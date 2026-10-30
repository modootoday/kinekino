---
id: plan:kinekino/20261029113555
title: 전체 제품 통합 완료 조건
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 전체 제품 통합 완료 조건

12시간 검토 빌드의 산출물별 담당·선행 관계·공유 실행 자원과 미확정 시간은 [의존성 점검](review-build-dependency-check-2026-10-25.md)에 분리한다. 해당 문서는 검증 결과나 새 완료 범위를 소유하지 않는다.

2026-10-25 사용자가 확정한 우선 납기 목표는 [12시간 사용자 검토 빌드](user-review-build-12h-2026-10-25.md)다. 현재 실행 배정의 최신 상태와 인수 흐름은 해당 문서를 먼저 확인한다. 전체 범위는 이 문서에 유지하며 12시간 경과나 부분 테스트 통과로 완료 처리하지 않는다. [사용자 인수표](user-review-acceptance-2026-10-25.md)는 통합 실행본·실제 표본이 준비된 뒤 실제 경로를 채워 제공한다.

이 문서는 전체 제품의 완료 조건을 소유한다. 최신 root/Meari 핀·단계별 실제 결과·현재 담당은 [현재 조율](current-coordination.md)의 기준 및 ledger, 실행·편집 권한은 [실행 배정](execution-leases.md)을 따른다. 변경되는 상태를 여기 중복 기록해 오래된 기준을 만들지 않는다. 각 검증은 서로 다른 범위·시점의 근거이며 전체 제품 완료가 아니다. 타입검사·합성 회귀·실제 OS·미디어·사용자 품질 승인을 구분하고, 이 문서로 새 실행·다운로드·추론·수동 GUI 인수 결과를 생성하지 않는다.

## 필수 통합 게이트

AI 품질 최종 인수는 2026-10-24 사용자 결정에 따라 실제 제품 GUI의 사용자 테스트에서 수행한다. 개발 담당의 SDK/CLI 기능 검증과 자동 검사·실측은 선행 근거이며 이를 사용자 승인으로 대체하지 않는다. 실제 표본과 GUI 검토가 준비되기 전은 개발 미완료다. 세부 범위는 [모델 검토 정책](../../sots/player/application/model-quality-review.md)을 따른다.

| 게이트 | 현재 근거 | 완료를 입증할 후속 증거 | 담당 |
|---|---|---|---|
| Source 세 분류 종단 CLI | same-endpoint-three-categories-r1에서 방영중/종영/극장판의 실제 provider→Frame-host→native 재생 위치·PNG·종료를 관측했다. 원 입력·고정 CLI·family/EOF 근거는 현재 조율 ledger에 보존 | 최신 변경의 직접 영향이 있으면 해당 경로를 재검증한다. 광고 차단 전체 범위·stale/만료/실패 복구·음성 품질·GUI 동등성을 이 성공으로 확대하지 않는다. 세부 잔여 계약과 현재 주입 기준의 지원 범위를 종단 연결 | SDK/provider·supervisor 담당 |
| 실제 OP/ED 제출 | [실제 미디어 r4](../../sots/verification/application/native-export-actual-media-2026-10-21.md)의 과거 c7fe 체크포인트에서 동일 engine/resolution·원 guarded submit1→native ACK→별도 paused landing→terminal consume/NotFound→EOF 정리를 실제1회 인수했다. 최신 소스의 A/B/C/wire·원 Sessions 연결도 SDK가 읽기 대조했다. 최신 커밋의 새 실측이라는 주장은 아님 | [실제 endpoint-loss r1](../../sots/verification/application/native-export-delivery-loss-actual-r1-2026-10-24.md)에서 r6 고정 validation CLI의 원 submit1·preframe 응답 유실·ref 없는 origin-list 복구·원 ACK/paused landing/consume·Final/정상 EOF를 exact1 인수했다. 이는 opt-in validation artifact의 단일 실제 시나리오이며 기본 배포·전체 오류 행렬 완료가 아니다. 동일 instance negative/override·자동 tick/detect·음악 판정은 별도이며 일반 seek_submission=false를 임의 해제하지 않음 | SDK/스킵 |
| 다운로드·제작 executor | 계약·명령목록과 일부 독립 제작 실험 | download/cards/shorts/subject/assets 각각 source/revision·출력열림·취소·충돌·부분결과·재시작·저장/재열기의 실제 SDK/CLI. 독립 산출물을 제품 기능으로 계상하지 않음 | SDK/제작 |
| 저장 프로젝트와 live 편집 권한 | SDK 실제 CLI의 origin/source/state CAS·반복 편집·exact accept retry·legacy 거절에 더해, [명시 entry의 실제 guarded IPC](../../sots/verification/presentation/capture-positive-entry-2026-10-21.md)에서 구성 저장소·원 ExecutionOwner를 통한 편집/채택/원 요청 재시도·stale 거절·fault/close를 인수했다 | origin 없는 과거 capture는 읽기 전용으로 보존한다. 검증된 opt-in entry를 제품 Card 입력·mount/frame·route에 연결하고 원문 비교·사용자 수정·명시 채택·저장/재열기까지 같은 SDK 경로로 인수한다. 구체 adapter ACK 실패 복구와 실제 GUI/시각 승인은 별도다. IPC fixture의 통과만으로 기본 제품 사용자 흐름을 완료하지 않는다 | SDK/UI |
| 기본 Native 통합 | C21 편집/품질 UI·명시 NativeProposal과 원 owner/ACK/LocalHost 근거에 더해, A 공통 표시12·D 기본 chooser/VIEW 여정·B 실제 persisted 결과 재열기3+카드 회귀1 및 fileguard 순수5/실제 filesystem3를 각 고정 입력 범위에서 인수했다. [B](../../sots/verification/presentation/persisted-review-reopen-b4-2026-10-24.md), [fileguard](../../sots/verification/presentation/default-native-file-guard-2026-10-24.md) | 사용자 재시도 동작과 같은 SDK/LocalHost/engine·checked parent/실 child surface·capture/crop·편집·close/recovery를 기본 player.exe에 연결. 실제 OS수명/청취/overlay·사용자 시각 승인 별도 | UI/SDK/Meari |
| AI 제품 IO·검토 | HF 고정·독립 모델실험·proposal/품질판정 경계. 10/21 사용자지시로 단계 재개 | 하드웨어예산의 단일worker·제품연결 뒤 pinned 모델입력/출력·provenance→job 오류/취소→사람원문비교/수정/명시채택·저장. 독립 모델실험/합성 deferred를 사용자품질승인으로 대체하지 않음 | AI/SDK/UI |

## 기능 범위의 주의점

- 공통 폰트의 opt-in 체크포인트는 Meari `5893d448ade4c8b7cb5f9102c2a8fdb76797fe76`, root gitlink `7af987b27c07bf6211d31a31683e166bb93a3f8f`에 채택됐다. [Native r2 실제 시험](../../../packages/meari/.spec/evidence/verification/shared-font-native-r2-2026-10-25.md)의 Core4/Native2 근거는 고정 리소스·TTF parse/metadata·CSS 전달 범위다. 기본 GUI의 폰트 선택·다국어 glyph/shaping/fallback·Web 동등성·archive/bare checkout은 별도이며, 이 커밋을 제품 시각 인수로 계상하지 않는다. 후속 SDK 검증은 직접 의존하는 새 Core 소스도 반영해 현재 root와 맞춰야 한다.

- 2026-10-25 PM은 수동 스킵 검증 소스 exact5를 root commit `8ba92768cdb5a0ba32d1d73fb0fabd662664ba11`에 채택했다. [root 통합 근거](../../sots/verification/application/manual-native-skip-root-integration-r2-2026-10-25.md)는 fmt/default strict와 동일 시험 소스의 기존 CPU/N/V/O 근거 재사용을 구분한다. 이는 새로운 GUI 기능 연결이 아니다. [공통 재생 assembly 추출 계획](../architecture/native-playback-factory-2026-10-25.md)의 private 후보를 작성 중이며, 추출 후 새 CLI의 소유권·실패 정리 및 실제 N/V/O를 별도 인수해야 한다. 기존 성공은 새 조립 코드의 실행 증거로 승계하지 않는다.

- [실제 V/O r2](../../sots/verification/application/manual-native-skip-owner-actual-r2-2026-10-25.md)는 같은 기본 CLI/원 영상으로 exact1/9409ms 통과했다. override·새 pass·단일 manual admission·stale/중복/사후 취소와 정책 세션 종료 뒤 원 row/ACK/consume 보존, 원 host EOF 뒤 이전 endpoint not_sent, 새 NoMedia host의 이전 issuer/session 거절을 확인했다. 원 family55/EOF277/PID57 부재·writer0을 인수했다. 실제 in-flight 취소·global native attempt counter는 관측하지 않았으며 자동 OP/ED 감지·음악 판정·복구 기능이나 전체 스킵 완료로 확대하지 않는다. 검증 소스의 root 통합은 별도 단계다.

- `manual-native-skip-negative-actual-r1`의 기본 CLI 실제 N1은 초기 관측에서 실패했다(outer101/5078ms, 0pass/1fail). 원 응답은 engine/resolution 일치·has_error=false지만 connected=false와 재생 정보 null이었다. 이 원 실패는 보존한다. 후속 readiness 분류 수리의 순수3/회귀38 통과 뒤 [실제 N r2](../../sots/verification/application/manual-native-skip-negative-actual-r2-2026-10-25.md)가 exact1/8227ms로 통과했다. 동일 instance에서 unverified·stale·foreign binding·사전 취소·일반 미지원·원 origin 빈 목록을 확인했고 원 family38/EOF192/PID40 부재·writer0을 인수했다. 이는 명시 기본 CLI의 N 범위이며 V/O·전체 스킵 오류 행렬·사용자 품질 승인은 별도다. 초기 실패를 제품 연결 고장이나 입력 만료로 확정하지 않는다.

- 후속 [현재 비계측 N1](../../sots/verification/presentation/current-native-noargs-env-off-2026-10-24.md)은 daff354의 정상 native 빌드 `712143D4…`에서 제품 argv 없이 진단 옵션을 제거한 실행 1회를 통과했다. 원 State/receipt는 physical_closed·normal_success·physical_completion, child exit0/Empty/비강제와 명시 ReleaseClosed/FinishHost·host0을 기록한다. 최종 slot `C5BD1858…`의 EOF14·기록 PID13 부재·writer0을 인수했다. 현재 기본 실행의 창 종료 근거이며 과거 비계측 실패의 원인 소급 확정, 첫 DOM·SDK join·영상·시각 품질 증거가 아니다. 원 실패 기록은 보존한다.

- 후속 [selector r2 configured C](../../sots/verification/presentation/default-native-viewer-selection-c-r2-2026-10-24.md)는 새 CLI의 실제 프로젝트 조회 뒤 configured 실행의 SDK endpoint·정상 native return·worker 종료 및 원 exit0/Empty/EOF를 인수했다. media_started=false/capture=null/renderer_frame_observed=false이며 미디어·시각 검증과 구별한다. N/C 계측 성공 뒤 UI 입력 seam 검증은 별도로 남는다.

- 후속 [selector r2 실제 N](../../sots/verification/presentation/default-native-viewer-selection-n-r2-2026-10-24.md)은 계측된 no-args 실행에서 실제 viewer 선택·CloseRequested/Destroyed·exit0/Empty/EOF·정상 host 정리를 인수했다. 원 실패 기록은 유지한다. configured C, 비계측 실행, 첫 프레임·시각·전체 기능 검증으로 확대하지 않는다.

- 기본 Native의 부분 여정 인수와 일반 실행 종료는 구분한다. [기본창 N 최초 실패](../../sots/verification/presentation/default-native-nc-first-2026-10-24.md)는 Close/exit cutoff를 실제 관측했으며 C 실행은 중단됐다. 승인된 비정상 정리는 정상 종료 인수가 아니다. 후속 계측은 원인 조사이며 원 비계측 실행의 해결 증거를 별도로 요구한다. [감독 도구의 forced-success 차등](../../sots/verification/production/owned-test-runner-forced-success-os-cli-2026-10-24.md)은 도구의 성공 오판 수정을 입증하며 제품 창 종료 문제의 해결 증거로 승계하지 않는다.
- guarded legacy 생성자의 disabled scope와 일반 seek_submission=false는 유지한다. 명시 standalone host의 CLI 조립·NoMedia 빈 목록 인수에 더해 위 실제 미디어 r4에서 원 producer 결과·동일 instance ACK/landing/consume를 별도로 인수했다. 빈 목록 시험을 그 근거로 승격한 것이 아니다. [phase2](native-result-export-phase2.md)의 초기 미구현 주장은 선행 계획이며 최신 A/B/C/wire 소스와 구별한다. 실제 endpoint 응답 유실 후 원 origin-list 복구는 위 r6 validation 고정 바이너리 exact1 근거로 인수했으며 일반 제품 지원 승격·negative 행렬은 별도다. parent-client checked borrow의 OS1 인수도 제품 child·입력·visible 렌더·영상 권한을 입증하지 않는다.

- 자막 convert/export_range의 현재 저장·CLI 인수와 ASS/OCR·ASR·화자분리·alignment 제품 executor는 구별한다. 외부corpus ignored는 성공이 아니다.
- snapshot의 fixed SHA PNG→명시 pixel crop 인수와 현재 실제 capture→crop 연속 gate, 텍스트/리사이즈/마스크/HDR/색상·정확프레임PTS·GUI편집은 구별한다.
- 커서는 mpv의 `input-cursor=yes/cursor-autohide=no` 소스 수정과 실제 parent/child hover·이동·focus·전체화면 유휴/복귀를 구분한다. 과거 egui 조작부 표시 수리를 현재 Meari child 커서 어댑터나 실제 OS 인수로 승계하지 않는다. 원 parent lease의 존재만으로 cursor 제어 권한이 생기지 않는다.
- catalog·jobs·projects·proposal·snapshot·native session 명령은 주입 상태에 따라 지원된다. 명령 목록 존재만으로 기본 SDK/file CLI/provider host/native host/endpoint가 동일하게 지원한다고 보지 않는다.
- source의 초기 미지원 문구와 최신 실행 근거가 충돌하면 실제 dispatch·현재 주입과 기능별 최신 검증 시점을 먼저 대조한다. 전체 미구현을 일부파일 검색만으로 추론하지 않는다.

## 연결 문서

[세 작업 기본 UI 잔여 seam 조사](../../../packages/meari/.spec/evidence/reviews/product-review-edit-accept-reopen-seams-2026-10-24.md)는 기존 비교·수정·저장·별도채택·SDK 재열기 구현의 소스 근거를 구분한다. 중복 구현 대신 기본 chooser/Open/재열기, 설정 SDK·기본 Native 실행 조립, task별 same-item 결과 표시, 복귀 후 fresh chooser/old carrier 폐기 연결을 후속 인수한다. 이 조사는 실행 증거가 아니며 Accept의 문서/output revision 생성과 미디어 렌더링·자동 적용을 구별한다.

[원문 비교·수정·명시 채택의 기본 경로 감사](presentation/proposal-default-route-gap-audit.md)는 자막 교정·카드 문구·하이라이트의 기존 SDK/CLI·eframe·명시 NativeProposal 근거와 NativeViewer 잔여 조립을 구분한다. 기존 기능을 재구현하지 않고 공통 typed 선택·컨트롤러·원 owner/dirty/CAS/오류 보존을 재사용한다. 현재 Card만 연결된 opt-in capture를 세 task의 기본 Native 통합 완료로 확대하지 않는다. Jobs49는 root89f59f5에 채택됐고 [retained Jobs 검증](../../sots/verification/presentation/jobs-retained-source49-2026-10-23.md)에 최종 library1019pass/54ignored 및 별도 r9 실제 OS3의 귀속을 보존했다. 이는 Jobs list/details/복귀와 원 SDK owner·checked frame proof·원 close 경로의 근거이며 자막 교정·하이라이트 route 또는 수동 시각 인수가 아니다. 이 실제 HostTurn의 commit/continuation/retained observation 계약은 후속 세 route에도 적용해 검증한다.

[현재 조율](current-coordination.md), [Native shell](presentation/default-native-shell.md), [영상 합성 비교](native-video-composition-comparison.md), [SDK surface gate](native-surface-rust-gate.md), [스킵 host gate](manual-skip-host-gate.md), [snapshot](snapshot-export.md).

기능 정본은 .spec/sots/player/application/api 및 production 하위의 catalog/native session/downloads/snapshots/cards/shorts 계약과 AI model-status·model-quality-review를 따른다. 세부 최신 근거는 담당 정본에서 인수하고 이 문서는 원요구의 통합 게이트만 소유한다.

Pages·VM·DPI·새장시간시험·수동GUI검토 보류를 유지한다. AI전체보류는10/21 사용자지시로해제됐고 [하드웨어배분](ai/resource-allocation-2026-10-21.md)의 단일worker·작업별중앙배정으로진행한다. 다른 보류를 이 계획의 완료 조건 때문에 임의 해제하지 않는다.

