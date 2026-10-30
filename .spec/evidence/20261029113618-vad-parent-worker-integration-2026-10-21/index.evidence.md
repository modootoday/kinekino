---
id: evidence:kinekino/20261029113618
title: 부모 공급·공통 worker·SDK 합성의 실제 default 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 부모 공급·공통 worker·SDK 합성의 실제 default 검증

2026-10-21 PM soleheavy 배정으로 root `4d14ceecab4ed90fa59302a34818bb76f3c999ae`/Meari `9c1b6a339885ce9e2f593c322bc11ee213e0a0bf`에서 수행했다. 원parent1a→PM 승인11829→현재gitlink4d를 별도 기록하고 before54 실제 차이0을 확인했다. gate는 `runtime/verification/vad-parent-worker-integration-r1`이다. 기존 S/public-p3-parent-r1/native T, canonical runner EEA9 conservative2/CPU2/mask3/BelowNormal/descendant Job/offline/j1/thread1을 사용했다. live free RAM29952412KiB/GPU5727MiB/C173039124480B를 확인했다.

## 동결 입력과 최소 수정

원 Media52 `56BDF6C48B26372D3A062A0FBE031830C5838605E6233853B69CA43F67519A14` + SDK2 `EDF43CD37391520005A04968FE89D27E2A69FDEC7BDA46622AC6052BC7107582`의 union54 `028DC1081A317ACA7E3D120AEAB9A4D1E9A8C49F9E999D8EE3487E5E647396EE`를 합성했다. r2/r4 원문·SDK 저작권한은 보존했다. 공통 P0/protocol/executor/worker 단일 path/type graph와 executor binding 재노출을 실제 검사했다.

원 strict9190ms/101은 root 시험의 media-vad-protocol 직접 dev-dependency 누락 E0433이다. 별도 compile-r2는 기존 공통 protocol devdep1만 추가했다. 다음 strict15492ms/101은 새 root 시험 소비자의 공유 fixture 미사용·공개 mapping 필드·기존 unadmitted constructor 미사용이다. compile-r3는 공유 fixture entry/type을 사용하고 실제 profile/audio/destination 원 참조 매핑을 assert하며 SDK와 같은 기존 constructor 호환 함수 참조를 유지했다. allow/expect/skip·시험 약화·정책/Scope/Deadline 변경은 없다. 원 실패 로그/receipt를 덮어쓰지 않았다.

최종 compile-r3 final54 `B85078EFC069F70ACEA7CC23121455D63CE8DC5045FACDCAED01EAC714331C4C`는 원54 중 Cargo.toml·새 roundtrip 시험 두 경로만 override한다. 실제 final-source54 사본 manifest는 `9A56DB27D66EC21C36A432A4DE221CE7DA75469B0E1FBB5AE43637CE8E973B89`다. root 원 파일/index/lock은 수정하지 않았다.

## 실제 graph·검사·BIN

최초 metadata3936ms/0에서 snapshot lock은 root worker edge/local node와 optional ORT registry8 추가만 보였다. matrixmultiply0.3.11/ndarray0.17.2/num-complex0.4.6/ort·ort-sys rc14/portable-atomic1.15.0/portable-atomic-util0.2.8/rawpointer0.2.1의 name/version/source/checksum은 고정 standaloneDA809와 전부 같다. 기존 root registry tuple 변경0이다. 비교 root721/fixedORT80A를 보존했다. 수정 후 metadata의 lock 차이는 root common protocol edge1뿐이며 최종 lock은 `E469A7204B9DC2AC82AC729E36FD8AC41449E1C382E9167A28A00A45C45789A2`다.

actual resolve560에서 공통4 local ID는 각1개다. P0/protocol/executor feature[], worker의 empty default만 활성이고 native-ort/ort/ort-sys 활성0이다. 실제 compiler-artifact의 ORT0도 확인했다. lock에 optional ORT가 존재하는 사실을 native compile/load로 해석하지 않는다.

최종 Cargo fmt4807ms/0, root+공통4 package default all-target Clippy `-D warnings`27367ms/0. contract2 build6638ms/0, executor/worker lib2 build8049ms/0, root test2 build124148ms/0이다. root build의 원 exec session5980을 유지·종료 회수했고 capacity 관측 중 재시작하지 않았다. 정확6 test BIN은 모두 fresh=false로 빌드해 원본/고정 사본 SHA를 대조했다. 실제 list는 P022/protocol28/worker10/executor43/rootparent30/rootSDK32다. root no-run이 만든 player/player-cli auxiliary2도 사본으로 보존했으나 실행0이다.

## 실제 고유81

| 직접 영향 | 실제 exact 결과 |
|---|---:|
| 역사64: P022+protocol28+workerpayload10+executor로 이동 binding4 |64pass|
| 새 parent producer→sidecar reader→worker/P0→공통 응답 |3pass|
| 기존 concrete producer/lease/취소/충돌 fixture |7pass|
| native preparation cutoff/inert/manifest 제한 |3pass|
| SDK 원scope/readmission/owned-host 계약 |4pass|
| 합계 |**81pass/0fail/0ignored**|

실제 full list에서 selected81 이름·BIN을 고정하고 같은 사본에 exact/nocapture/thread1로 각각 실행했다. inherited producer/P0를 다른 consumer에서 다시 세지 않았다. SDK3 stderr1590bytes는 실제 cancelled/source_expired/timeout 상태와 original host closeout 관측이며 오류 코드·상태와 통과 결과를 함께 보존했다. ignored actual CLI 시험은 실행하지 않았다.

새3은 원 admission/manifest/request/lease를 가진 producer가 쓴 sidecar를 concrete worker reader로 읽고 inert engine/P0/common response로 연결한다. Complete·positive Partial의 원 cancel/cutoff 유지·pre-engine Cancelled를 확인한다. 원 runtime bytes는 실행하지 않는 파일이며 actual_runtime은 absent다. 실제 main/process/loader·정상 AI 결과·cleanup ACK를 증명하지 않는다.

## 종료·보호·반환

98 runner receipt 모두 terminal이며96 exit0·원 strict 실패2 exit101이다. 원 exec 세션도 회수했고 regular stdout/stderr를 끝까지 읽었다. 이는 실제 AI pipe EOF나 family cleanup 검증이 아니다. 기록 PID195 중84972가 나중 chrome으로 재사용된 관측은 PID-REUSE.md에 분리했다: 원case07 terminal08:33:33Z 뒤 chrome 생성08:34:21Z다. 후자의 parent/executable/handle 미수집을 명시하고 사용자 process를 조작하지 않았다. PID 부재만으로 cleanup을 주장하지 않는다.

root1194/library473/index·docs4/S 비입력1168 차이0. S library는 승인473의7개 변경만 반영했고 최종473 해시 차이0. final54/원r4·S r4/과거 default BIN3 원본·사본 차이0. 기존 BIN48 고정 사본 차이0이며 원본 중 player/CLI 두 고유 경로만 의도된 no-run auxiliary 재빌드로 교체됐다(서로 다른 경로 표기4행). 새 executable artifact8 원본·사본 차이0이다. 원root lock721 불변, stage/commit/push0이다. 보호와 원receipt를 회수한 뒤 PM에 heavy 슬롯을 명시 반환했다.

## 남은 범위와 적용 스킬

부모 SDK 연결의 default 타입/파일 계약을 입증했다. operation 등록·실제 worker main/trusted loader/native ABI·ORT 모델 추론·원 process identity/signal/pipe EOF/family ACK·배포 권리/고지/최소OS·CLI 제품 인수·사용자 품질은 미완료다. 전체 AI/미디어 goal 완료로 올리지 않는다. 원 입력/오류/수리/final54는 PM root 채택의 검토 자료이며 공개 worker directory allow/최종 commit은 PM 소유다.

modoo `modootoday/software-qa/code-change-review-verification` 본문9F73865F…/references/verdict-table.md 9D9619A8…를 적용해 source-only 주장과 actual metadata/list/same-BIN 결과를 분리하고 원 오류를 보존했다. Rust-pro의 동일 Arc/제어/소유 경계를 유지했다. 신규 모델·다운로드·GUI/브라우저·실worker·ORT·VM/DPI 실행0이다.
