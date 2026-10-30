---
id: evidence:kinekino/20261029113542
title: 원 leased regular-file fd 검증 checkpoint
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 leased regular-file fd 검증 checkpoint

## 2026-10-23 최종 다운로드 경계 게이트

현재 검증 입력은 SDK effective10 `F3C72C2E0EDBEBD6E8D3F566A0A18B58E77F94C771791B1B1B6829906C967587`이다. native rename의 UTF-16 NUL 경계와 목적지 FileID 일치를 검증했다. 원본 핸들의 hash가 다른 이름의 파일을 성공으로 보고하던 선행 실패는 삭제하지 않았다.

정확한 실행·보호·제약 정본: [STATE](../../../../runtime/verification/download-rename-boundary-r1/STATE.md). runtime 경로는 프로젝트 root 기준 `runtime/verification/download-rename-boundary-r1/`이다. 실행 영수증 16개 exit 0, 관련 시험 foundation 25/owned 22 및 별도 실제 미디어·취소 5개, doc 2개 통과. rootlib의 기존 713/5와 선행 실패는 별도 근거다.

정상 alias4는 원 bytes로 복원했으나 FileID는 새로 생성됐으며 deps/fixed 검증 산출물은 보존했다. T 최종 변경 exe5, 추가·삭제0, 과거 미귀속2 유지로 all_cache_protected=false. 담당 실행·핸들·writer·정리 대기0으로 슬롯을 반환한다. 제품 연결 source13/r2, 실제 provider 전체 지원 및 CLI 전체 인수는 미완료다.

스킬 출처와 적용 판단, 상세 hashes 및 원본 실패 위치는 위 STATE에 기록했다. 이번 기록은 전체 제품 목표 완료 판정이 아니다.

2026-10-23, **부분 검증이며 gate 미완료**다. root `8014701bc88d0439abf9db6a96b7b3d088bd12fa`/library `b532fe0eed06d8526b755b11bc36416070244ef7` 기준 private 후보를 기존 제품 S/T에서 시험했다. 원정보 관측과 입력 입장은 [별도 정본](ffprobe-owned-observation-2026-10-23.md)을 따른다. HTTP/provider/제품 CLI 완료를 뜻하지 않는다.

## 직접 근거와 판정

gate `runtime/verification/download-real-probe-r1`의 원 stdout/stderr/runner receipt와 `observer-r5`의 원 host journal·probe raw·close receipt가 직접 근거다. r5 effective5 manifest SHA `76942572C9FDF121AEB89C097C98C5F08ABCDB8785E7276111B292D6B4C71363`, 실제 Cargo JSON에서 고정한 media BIN SHA `C419F4DFAFA62B6EF77B837FDE11CC8478A63C0AD2595C22FD23ADD2DCCA474E`/3558400B다. 실제 등록23/ignored3을 확인한 뒤 같은 BIN으로 실행했다.

| 근거 | 실제 결과 | 입증 범위 |
|---|---|---|
| 23-digest-units | 6 pass, 712 filtered | legacy/checkpoint hash·원 취소/timeout 오류·read kind/raw OS·byte bound 경계 |
| 31-real-normal-observer | 1 pass, 7.40초 | 원 core/host/partial lease→실제 ffprobe regular stdin/fd, 같은 해시/932830B 결과의 단일 publication·단일 Completed |
| 32-real-truncated | 1 pass, 7.31초 | 같은 원 입력의 첫16B를 actual inspect가 거절, publication/output0 |
| 33-real-source-cancel | 1 fail: actual Failed, expected Cancelled | 원 외부 Cancelled 오류가 현재 source 경로에서 실패로 보존됨. 협력 취소 인수는 실패 |
| 34-owned-regression | 기존20 pass, real3 ignored | 원 synthetic child와 소유권·저장·오류 회귀. 실제 media 성공과 별도 |

31 inspect raw는 video0 h264720×1280/nb_read_packets288, audio1 aac48000Hz/6ch/nb_read_packets564, format12.000000초를 보고했다. capabilities/inspect는 각각 child0/familyEmpty/forcedfalse·stdin_eof_requested=true다. 32 inspect raw는 moov atom not found/invalid fd input, child1/familyEmpty/forcedfalse다. stdin EOF 요청과 물리 family·출력 EOF 검증을 구별하며 원 supervisor와 bounded raw read의 의미를 유지한다. 전체 decode/A-V sync/시각 품질 증거가 아니다.

원 tool은 고정 ffprobe SHA `efb014c8f39091c90f3616154fe20ab8470f5f943f6a48ec4fd30c479c7dd697`/82620928B다. 원 private portrait SHA `0fd9b2de74774b4d2dcea9f9b66466899b44aff1300b121d548e10af4c386483`/932830B, input manifest SHA `C90B2C168EA25C78889531008D89DFED79C7CD60CB3EF6AC733B23B8B1275C25`를 유지했다. 입력/도구 신뢰·권리·재배포 범위는 이 기술 시험으로 확대하지 않는다. 34만 synthetic pin0A3A/209408B를 소비했다.

## 실패와 최소 수리 이력

최초01 fmt 실패는 모듈 순서만 r2에서 수정했고 원문을 보존했다. 실제 정상08/진단15/수리24는 원8초 시험 observer가 원15초 attempt cutoff 전에 host shutdown을 요청한 실패다. 15 old production prepare 약8.287초 중 control1151회 누적2.330초를 측정했으며 residual을 순수 SHA 비용으로 확정하지 않는다. 공통 CNG hash 수리 뒤 24 prepare 약3.943초/control1264회 누적3.608초였고 capability는 정상 종료했지만 inspect 전에 시험 observer가 끝났다. 실제 성공 이전 수리를 성능 보장으로 표시하지 않았다.

승인된 r5는 8초를 중간 관측으로 남기고 **같은 원 attempt의 15초 절대 cutoff**까지 관측한다. operation deadline 갱신·재submit·새attempt·checkpoint 빈도 완화0이다. 기존 실패/BIN/trace를 보존한다. 31 자체는7.40초에 끝났으므로 성공을 observer 변경 하나의 효과로 단정하지 않는다.

33 원 job은 사용자취소true/publicationfalse/output없음, 오류는 Cancelled-code `Original source cancelled`였다. probe는 시작하지 않았고 원 host closeout은 Complete/first_error null이다. 실패 기대값을 실제 Failed에 맞춰 완화하지 않는다. source 단계 취소를 실제 ffprobe child 취소/drop/unwind의 증거로 쓰지 않는다.

## 미채택 취소 수리와 다음 gate

승인 exact4는 application/download의 ports·transfer·owned_session과 기존 real fixture leaf다. 원 AttemptScope만 opaque cooperative marker를 발급하고, 기본 owned read가 모든 외부 ApiError를 Failure로 보존한다. sync 계약·bytes/hash/seal 순서·jobs core/CAS는 유지한다. 외부 Cancelled와 사용자취소의 경합, 원 close 실패의 취소 ACK 금지, 기본 read adapter의 오류6필드 원형 보존을 별도 회귀로 작성했다.

최신 source-only 입력은 `runtime/development/download-local-stop-r2/effective-source8.json` SHA `4B6E2EED3AD68C14CD66529D13890C9BDDCD747221B289FB0AFF72F264E41842`다. r1 delta4/union8과 r2 delta1은 불변 보존한다. leaf fmt/check만 통과했으며 최신 후보의 컴파일·실제 목록·시험은0이다. 예상 media26/ignored5와 root digest6/worker3/기존synthetic20을 실제 목록과 고정 BIN으로 구별해 검증해야 한다. 기본 adapter의 독립 control 시험은 원scope/host/CAS를 입증하지 않는다. blocked transport 자체 오류는 Failed로 남으며 협력 취소는 원checkpoint를 호출하는 adapter 범위다. probe 경합·foreign marker·deadline 만료의 새 actual은 없다.

## 자원·보호·미완료

기존 S/T·runner EEA9DEE6·conservative2/CPU2/BelowNormal/descendant Job/offline/locked/j1/thread1을 사용했다. 처음과 재개 baseline·사전 BIN/normal aliases/preimage·Cargo JSON producer를 보존했다. 최종 전체 보호 대조는 아직 끝나지 않았다. 역사 미귀속 cache alias2/all_cache_protected=false를 현재 delta 귀속으로 해소하지 않는다.

`observer-r5-partial-heavy-return.json` SHA `68E1C231FE25B983258C47B1A4987BE259ACE84A8E58443C73A554C47460BB25`에서 본인 원process/session/handle/sharedwriter/cleanup0을 확인하고 heavy를 일시 반환했다. gate 완료 반환이 아니다. UI가 독립SDK5 backup→rootbase+UI46→원SDK5복구 책임을 인수했다. 이후 SDK는 private source-only이며 실행 재배정 전 Cargo/S/T/실제 도구0이다.

원취소 수리 실제 인수·최종 fmt/test/clippy·보호 마감·생산 executor 등록·HTTP 원 response/runtime/resolver 물리 ACK·provider auth/headers/HLS/resume/remux/modes/batch·제품 CLI와 전체 자막/제작/AI 목표는 계속 남는다. 사용자 품질·시각 인수와 SDK 기능 검증도 별개다.

적용 스킬: modoo.today `modootoday/software-qa/test-fixture-and-mock-build` SKILL SHA `a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100`—원 계약·오류·정리·실제와 synthetic 증거 구분. `code-change-review-verification` SKILL SHA `9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B`, verdict ref SHA `9D9619A829AE24D73645EF5F7C8126C8257CAEDDAC80800632866257008AD2A9`—작성/실제등록/실행·원 실패·부분 판정과 미완료 경계 기록에 적용했다.
