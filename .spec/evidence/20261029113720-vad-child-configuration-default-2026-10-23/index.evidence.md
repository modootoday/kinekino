---
id: evidence:kinekino/20261029113720
title: Parent-selected child configuration default 검증 — 2026-10-23
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Parent-selected child configuration default 검증 — 2026-10-23

최종 고정 시험 BIN의 고유 **190개**가 각각 exact 1 passed/0 failed/0 ignored로 통과했다. 기능적 source consumer의 타입/파일·pin·lease·오류 검증이며 actual child16/22, native ORT/ABI·모델 품질·normal 제품 parent 활성화의 근거가 아니다.

## 입력과 실행 범위

중앙 명시 배정으로 root `e9aee3ec4359e33b2c46f3a3bf219f5281e4dc91`, Meari `0b9ea031ce5c08a5bf99ed9852b668cf2bfce038`, root/S lock `E469A7204B9DC2AC82AC729E36FD8AC41449E1C382E9167A28A00A45C45789A2`에서 실행했다. UI 채택33과 원 후보26의 경로 교집합0, root preimage26 일치, S SDK39/UI33 source72 일치를 실제 확인했다. 10/21 일시정지와 실패·PAUSED 원문은 보존하고 10/23 재개 때 S26+r1/r2·index/docs·writer·live 자원을 다시 확인했다.

Gate는 `runtime/verification/vad-child-configuration-default-r1`이다. 기존 S `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1`, T `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/native`와 runner conservative2/mask3/CPU2/BelowNormal/descendant Job, offline/locked/j1/test thread1을 재사용했다. clean·새 target·광범위 touch·cache/registry 다운로드·새 scope·deadline은 없다. source가 같은 파일의 mtime을 보존하고 실제 변경 bytes만 기록했다.

| 고정 자료 | SHA256 |
|---|---|
| 원 candidate final-union26 | `CCB6B3B64255C13DF675F3AF21AA5B914CDD94299C28CFB06487365DD5C5F900` |
| 실제 최종 source28 | `AC87DE63212A5398EC05709AB4542EF7075BA897FED70D3E8ECDF0FD60A16069` |
| root 적용 전 presence/absence·pin28 | `549FDA2202C2EC50BD9D7EDF6893AB79DE14F3BF4D98515ADBF4875D03748F17` |

최종 source28은 원26의 명시 수정3경로와 추가 test-consumer2경로를 포함한다. 원26/issuer8/release6/SDK39 및 모든 수리 전 원문은 불변이다. `final-source28.json`은 각 실제 source 위치·bytes·SHA를, `final-root-before28.json`은 root 인수 전 상태를 기록한다. 최종 private gate를 public root에 적용하거나 stage/commit/push하지 않았다. root index 책임은 PM이다.

## 실제 검사·등록·고유 실행

Windows `x86_64-pc-windows-msvc` filtered offline metadata356에서 기존 UI default 대비 package/feature/dependency 차이0, active ORT0, lock 불변이었다. 최종 whole fmt16 receipt0(5054ms), 관련6 package default all-target strict17 receipt0(3479ms)다. P0/protocol contract, executor lib/product_guards, worker lib/bin libtest, root vad_parent_worker/vad_jobs/vad_worker_control_process의 시험 BIN9와 root CLI/GUI 보조 BIN2를 Cargo JSON의 package/target/kind/profile/features/fresh/executable로 구분해 고정했다. 보조 BIN2는 실행하지 않았다. P0/protocol 계약 BIN은 unchanged inputs의 fresh=true이고 나머지 시험 BIN은 실제 rebuild를 기록했다.

초기 등록264 중 ignored9(actual child8/CLI1)를 실행하지 않았다. 세 product source consumer에 공통 fixture27·조립4가 반복 등록되고 원 native control3도 같은 source로 반복 등록됨을 실제 path와 목록으로 대조했다. 반복 등록65를 제외한 고유190을 선택했다. 조립4는 root vad_parent_worker에서 한 번 실행했고 나머지 두 consumer의 타입/등록 증거와 구별했다. r5 뒤 변경된 consumer BIN3만 재빌드·재목록화해 이름 차이0을 확인했다. 이전 BIN의 첫 성공3을 승계하지 않고 각 최종 BIN으로 고유190을 모두 새 실행했다.

| 결과 분류 | 고유 통과 |
|---|---:|
| 기존 normal129 재실행 | 129 |
| 기존 경로 helper2 재실행 | 2 |
| 새 configuration codec | 7 |
| 새 configured suffix parser | 2 |
| 새 실제 product constructor·lease | 4 |
| 이전 release6 작성 후 첫 검증 | 5 |
| issuer8 Windows 순수 deployment 검증 | 2 |
| 직접 source consumer의 추가 파일 guard/admission 회귀 | 39 |
| 합계 | 190 |

추가39는 executor release/source/admission17, 공통 product file guard20, product_guards 고유2다. 작성 수/등록 수/고유 실행 수를 합성하지 않는다. 각 결과에는 exact 이름·BIN SHA·원 receipt/elapsed/terminal이 있으며 실제 `1 passed`까지 대조했다. 근거는 `final-test-inventory.json`, `duplicate-registrations.json`, `selected-final-exact190.json`, `final-coverage-categories.json`, `results-final-exact190.json`이다.

## 최초 실패와 최소 수리

| 원 실패 | 별도 수리와 판정 |
|---|---|
| 03 strict101: Windows observation 정렬 검사 manual_is_multiple_of | strict-r1 exact1, 같은 정렬 판정을 is_multiple_of로 표현. 원 native 판정 변경0 |
| 04 strict101: optional deployment path collapsible_if | strict-r2 exact1, 기존 선택·경로 비교를 let-chain으로 표현. 허용/거절 조건 변경0 |
| 05 strict101: vad_jobs의 새 constructor/method 미사용2 | consumer-r3 exact1, 기존 실제 조립·guard 시험4의 명시 source mount2줄 |
| 06 strict101: product_guards의 동일 미사용2 | consumer-r4 exact1, 같은 기존 시험4의 명시 source mount2줄. 세 consumer를 rg로 전부 확인 |
| 원 test004: 정상 조립 뒤 evidence count0 기대에 실제1 | fixture-r5 exact1, 기존 `.vad-worker.lock` 생성 계약을 그대로 두고 정확 lock1·소유 중 두 번째 writer 거절·executor 해제 뒤 재취득을 검증. negative의 빈 디렉터리 기대는 유지 |

정책 우회·가짜 let 참조·lint 억제·시험 삭제·새 timeout은 없다. 원 실패 stdout/stderr/receipt와 첫 BIN/partial4를 보존했다. 09 콘솔의 null 요약은 OrderedDictionary의 Select-Object 표시 문제였고 원 inventory JSON과 실제 고정 파일은 정상임을 재대조했다. 별도 capacity 도구 오류는 프로그램 실패로 세거나 기존 완료 명령의 재시작 사유로 쓰지 않았다.

## 실제 관측의 한계

default graph는 shared codec, 세 제품 test consumer, Windows verifier의 default port와 순수 validation, worker main의 default 분기를 컴파일한다. `src/application/audio_vad/product_release.rs`·`src/infrastructure/vad_product_composition.rs`는 normal root module/dev-dependency 승격 전이라 미마운트이며 issuer3 시험도 미등록이다. native-ort의 engine/분기는 cfg-off이므로 그 타입·native ABI·DLL 관측·실제 Windows baseline을 증명하지 않는다. 실제 main의 configured22 프로세스 입출력도 실행하지 않았다.

신뢰 전제는 기존 trusted product parent다. creator PID/creation/QPC의 일치가 승인된 제품 parent 인증이나 모델·native 재배포 권리를 뜻하지 않는다. PRODUCT_RELEASE 미설정·실제 제품 worker/manifest·고지/권리·최소 Windows·정상 SDK/CLI operation 활성화·고정 실제 미디어 품질 인수는 남아 있다. 이190으로 전체 AI/미디어 또는 사용자 품질 승인을 완료로 표시하지 않는다.

## 보호와 실행 마감

source 사후 감사에서 root tracked1267/library479/index-docs6/raw77/S noninput3162/active28의 차이0, S 새 비입력0, root/S lock 불변을 확인했다. BIN519 중 Cargo가 실제 재생성한 원본9경로와 그 CLI/GUI 별칭2경로만 바뀌었다. 최초 audit의 unexpected2/expected:false는 덮지 않았으며 각 별칭을 원 Cargo19 producer의 실제 경로·SHA와 동일 C volume/File ID로 결속했다. 같은 baseline bytes의 SDK 고정 사본은 그대로 보존됐다. hardlink 열거는 unsupported50이며 성공/실행 근거로 취급하지 않았다. 근거는 binary-alias-resolution/identity/volumes-r2와 원 protection-final-binaries다. 최종 고정11과 UI r9 사본은 차이0이다.

전체 runner receipt234는 terminal이며 nonzero5는 위 원 실패뿐이다. 현재 gate의 live process0, 원 명령 handle 모두 terminal을 확인하고 중앙에 shared/S/T 실행 소유를 명시 반환했다. 실제 child8/22를 실행하지 않았으므로 그 family/EOF/ACK를 이 terminal 결과로 승계하지 않는다. 원 admission_cases의 owned host closeout은 해당 통과 시험의 범위로만 기록한다. slot-return.json과 ownership-closeout.json이 최종 마감 근거이며 PID 부재만으로 cleanup을 주장하지 않는다.

modoo.today `modootoday/software-qa/code-change-review-verification` 본문 SHA256 `9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B`, 참조 `references/verdict-table.md` SHA256 `9D9619A829AE24D73645EF5F7C8126C8257CAEDDAC80800632866257008AD2A9`를 재탐색·본문/참조 읽기 후 적용했다. 실제05/06으로 consumer 미사용 주장을 confirmed, 기존 writer lock source로 empty 기대를 refuted, native/생산 인수는 unverified로 구분했다.
