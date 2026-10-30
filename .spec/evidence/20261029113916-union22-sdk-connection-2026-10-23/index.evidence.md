---
id: evidence:kinekino/20261029113916
title: UI 복귀·슬롯 수명·JOB 상세 SDK 연결 actual138
status: active
created: 2026-10-29
updated: 2026-10-29
---
# UI 복귀·슬롯 수명·JOB 상세 SDK 연결 actual138

2026-10-23 PM 단일 heavy/shared 배정에서 최종 Native libtest 한 BIN으로 정상112+ignored exact26, 중복 없는138개가 통과했다. 신규 작성12개는 복귀2/lifecycle4/JOB6이며 수리 중 신규 case는 추가하지 않았다. [준비 계획](../../../plans/player/presentation/union22-r2-validation.md)과 [JOB 범위](../../../plans/player/presentation/job-details-read.md)를 따른다. 전체 제품 구현·전체 cargo test·기본 JOB route 연결·GUI/시각 인수 완료가 아니다.

## 고정 입력과 결과

root5a322e037e624e5d1ab4caaaa4c36d49953b4878, Meari0b9ea031ce5c08a5bf99ed9852b668cf2bfce038, root/S lock E469A7204B9DC2AC82AC729E36FD8AC41449E1C382E9167A28A00A45C45789A2에서 실행했다. root/index/library는 변경하지 않았다. 기존 S/T와 conservative2/mask3/CPU2/BelowNormal/descendant Job, offline/locked/j1/thread1 및 단계별 RAM16GiB/GPU free3GiB/C free12GiB admission을 유지했다.

근거 gate는 runtime/quality-review/ui-union22-r2다. 원 선택 union22-r2 AC8C5D64AA1DF8105E71F5A7FC891AD68C414DB32AD5736E54858DAB1916A59A와 before22 5507C3EAE290E9F5F8BBEBD31806738CA38F4C2E0B171D5423A51AF12BDD02FC를 보존했다. PM이 추가 제품 leaf1을 허용해 최종은 기존13/신규10의23개다.

- selected-source23-r5.json: 36DF48FF61142ACF91BE430781832881F572EC137D8030316754DBE5CC6C31FD.
- before23.json: BBD620A5EDB347C6401E7E245E592BD50A1515CCE8AFE3E421E7497023DD397A. 원22와 추가 read-panel.tsx 원본을 연결한다.
- fixed native-libtest-r5.exe: 8D8BA629AEB10D8F31DB3BDE18A65CBB5611D597CA3AB47B759CCBB2437DD0DF,171483648 bytes. Cargo22 JSON artifact에서 고정 복사했다.
- 실제 전체 등록1029/ignored40 중 선택 normal112+ignored26=138. 최종 normal21개 명령·ignored26개 명령의 pass 이름138, unique138, 중복0, 누락0, 예상 밖0이다. actual-r5-audit.json/actual-unique138-r5.json/registry-r5.json에 기록했다.

전체 S fmt20와 Native all-target strict21, build22가 최종 r5 근거다. default all-target strict15는 제품 최종 수정 r4의 근거이며 이후 r5는 native-proposal-window 하위 테스트의 거절 기대만 바꿨다. default metadata25는 같은 S/T와 lock을 확인하며 worker feature는 default만 있고 engine-ort는 없다. 원 default strict04 및 각 실패 JSON도 보존했다. 이는 project 전체 테스트를 실행한 결과가 아니다.

정상 필터22 중 viewer-launch-negative-list는 등록 목록 확인 전용이다. 유일한 등록 case가 ignored이며 승인 exact26 밖이므로 0-test 실행을 성공으로 계상하지 않았다. 나머지21개 필터는 normal112를 중복 없이 나눈다. 기존 actual116·registered inventory126 결과를 승계하지 않았다.

## 최초 실패5와 수리

1. S fmt01이 명시 path로 로드되는 product_surface의 하위 모듈을 잘못 찾았다. gate r1은 두 모듈에 실제 product_surface/ 하위 경로를 명시한다. fmt02는 실수로 root cwd에서 수행한 check여서 root-only로 분리했고 S 통과 근거는03이다. skip_children으로 전체 fmt 실패를 우회하지 않았다.
2. Native strict05의 JOB projection unwrap_or(String::new()) 두 곳은 gate r2의 unwrap_or_default()로 수정했다. 제품 동작 변경은 없다.
3. Native strict07은 private immutable_storage_paths를 assembly 테스트에서 import해 E0603으로 실패했다. 공개 jobs::read_only는 store/blob seed를 노출하지 않는다. PM 승인 gate r3는 원 Rust guard source를 test-local 모듈로 include해 기존 RootedJobPaths 위임을 유지한다. 제품 가시성 확대·새 guard 정책·동일 production adapter instance 검증 주장은 없다.
4. r3 BIN33AF9D16에서 normal112+ignored20 통과 뒤 lifecycle21이 mixed child placement로 실패했고 뒤5개는 미실행이었다. 고정 형제 사이의 직접 keyed slot 제거가 Meari0b9의 의도된 guard에 걸렸다. Meari 담당의 packages/meari/.spec/evidence/reviews/product-slot-mixed-placement-2026-10-23.md 실제 호출 검토를 읽고 적용했다. PM 승인 gate r4는 viewing/read-panel.tsx에 고정 무스타일 div를 두고 그 유일한 자식으로 slot을 옮긴다. 부모 fixed identity와 내부 only-keyed0↔1 계약을 사용하며 library/guard/migration API는 바꾸지 않는다. 실제 r4 lifecycle4가 통과했지만 이어 return25가 실패했다.
5. r4 BIN323F3D92의 return25는 disabled dispatch를 Ok(false)로 기대했다. 실제 Native source_callbacks는 정확 Invalid 오류로 거절한다. 승인 gate r5는 테스트 두 기대를 해당 variant와 동일 진단으로 수정했다. disabled DOM/빈 queue/SDK 불변/dirty veto/route guard 검증은 유지했다. 제품 정책은 바꾸지 않았다. 원25 실패·원 closeout도 보존했다.

최종 r5 BIN에서 전체138을 다시 실행했다. 이전 BIN의 부분 통과를 최종 통과 수로 더하지 않았다. 모든 원 실패 및 r1..r5 leaf/override/선택 manifest/BIN/log를 별도 보존한다.

## 입증 범위와 정리

실제 Meari 파서/Native document에 대한 slot empty→owned→empty, true→true DOM/instance 유지, focus/AccessKit 제거, 오래된 callback 거절, 실패 시 last-good 보존을 검증했다. 복귀2는 실제 SDK/LocalHost/callback/props commit을 사용하되 frame notice는 synthetic policy 입력이다. 화면이 실제 렌더됐다는 근거가 아니다.

JOB6는 실제 read-only SDK/store DTO와 기존 SdkPort/TypedAsyncSdkPlugin/AsyncSdkPump/Native document 경로, 기본 Unsupported 표시, pending query close 및 오래된 callback 거절을 검증한다. 취소는 query RPC 취소이며 durable jobs.cancel 지원이 아니다. 기본 StudioApplication JOB route/list/watch/cancel/retry/openfolder/단계 이력은 미연결이다. mount failure recovery와 진단 Write/flush fault 주입도 미검증이다.

최종 fixture closeout 로그28개 모두 complete=true다. 기존 assertion-failure cleanup 시험의 의도된 primary_panic=true1개는 test pass와 구별한다. JOB3는 primary_panic=false/cleanup_error=None이다. ignored26의 원 close/SDK assertions도 통과했다. runner receipts118은 전부 terminal이며 budget 불일치0, 첫 nonzero5는 보존했다. 원 tool sessions는 모두 exit/EOF를 관측했다. PID 숫자5개가 이후 다른 wsl/node/cmd/conhost에 재사용된 것은 pid-number-collisions.json에 분리했으며 PID 부재로 정리를 입증하거나 외부 프로세스를 조작하지 않았다.

## 보호와 미해결 캐시

protected-before3271의 변경은 Cargo22가 산출한 canonical Native libtest1(기존0979EC35→최종8D8BA629)뿐이고 나머지3270은 불변이다. 과거 고정 BIN과 r3/r4/r5 복사본을 보존했다. S-before2414 중 비입력2401은 hash/UTC mtime tick 차0이며, 최종23 raw/S hash 불일치0·root preimage 불일치0·lock 불변이다. 최초 감사의 JSON 날짜 자동 변환에 따른 문자열 비교 false positive는 source-protection-initial-date-comparison.json으로 보존하고 UTC tick 비교로 정정했다. 실제 소스·mtime 복구/수정은 하지 않았다.

선행 gate cacheunknown4709 별칭2는 여전히 미해결이다. 이번 before/after는 두 파일 모두2D2094C73DC9FA25162BFA1B2466F7C5C5E0390ED79E637914F6E86D6BC6CCAA로 동일하다. Cargo strict JSON은4709 build-script fresh=true, test build JSON은별도cc2f build-script fresh=true를 기록한다. media-vad-worker-json-producers.json과 cache-unresolved2-reobserved.json은 현재 관측만 보존하며 과거0C80→2D20 변경 producer를 확정하지 않는다. 전체 보호 true를 선언하지 않는다. 추가 Cargo/복구/삭제/clean 없이 반환한다.

무스타일 wrapper도 grid 구조를 바꾸므로 CSS·시각 동등성은 미검증이다. 수동 GUI/창·브라우저·영상·모델·네트워크·DPI/VM은 실행하지 않았다. 전체 화면 인수·기본 제품 연결·사용자 품질 승인은 후속이다.

## 적용 스킬

본문을 읽은 modoo.today modootoday/software-qa/test-fixture-and-mock-build를 원 fixture 소유권·실패 뒤 정리·primary/cleanup/diagnostic 분리·미측정 결과 구분에 적용했다. modootoday-free/design-delivery-plus/frontend-first-journey는 기존 공통 프레임/typed port와 미지원 기본 경로의 정직한 표시에 적용했다. JS/가짜 backend/새 정책 복제는 도입하지 않았다. 실제 추가 참조가 없는 두 스킬과 별도로 Meari 담당의 위 실제 계약 검토를 적용했다.
