---
id: evidence:kinekino/20261029113653
title: 전용 worker r4의 default CPU 계약 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 전용 worker r4의 default CPU 계약 검증

2026-10-21 중앙 명시 배정으로 root `1a919dc3b51d57474a48c5b1b3eb30b78a278d53`·Meari `b7ed5aba442bc143a9380536a850c4624a119a31`에서 실행했다. 원 근거는 `runtime/verification/vad-worker-default-r1/`다. 새 제품 worker가 아니라 ORT 없는 Rust 시험호스트3을 빌드·실행했다. 기존 S의 Meari가4331인 사실은 root 새b7과 분리 보호했다.

## 입력과 실제 빌드

원 r4 source40 `07CBBDF7F6EF910E40997271F7CB04BEF1F84D4F34FD153591EC2CA9A065BC85`, 외부 contracts/guard10 `69DF4A9361A6605D421236E5C053B137B129B576353C7BF5FB746F17C760809A`, 작성 expected64 `39BAF4103351499D067DD44D173A3ABCB83E50D973C36DBBA3049BEDA1DE6CAB`를 그대로 사용했다. 부모 payload r2·최신 SDK/Meari 기능은 합성하지 않았다. source 수리0이다.

live RAM/선택 GPU free VRAM/C 기준과 root/library/양 index/docs2·S·기존 BIN 보호를 확인한 뒤, canonical runner EEA9의 conservative2/mask3/논리CPU2/BelowNormal/descendant Job·offline/-j1로 수행했다. 원 40개는 같은 S/public-p3-parent-r1/runtime/development/vad-worker-r4 경로에 고정했으며 기존 native T/debug를 재사용했다. stage 원문과 root/r4 원문은 수정하지 않았다.

| 단계 | 원 관측 |
|---|---|
| offline metadata |351ms/0. workspace 원3 package/local path 확인, standalone lock 생성|
| registry·feature 대조 |generated lock DA809AB9…9E9D, registry37은 기존 root/고정 ORT 비교 lock과 tuple/checksum 차이0. active graph25에 ORT/ort-sys0|
| Cargo fmt --all --check |6369ms/0|
| 정확3 package default all-target strict |6783ms/0|
| P0/protocol contract no-run |11226ms/0|
| worker libtest no-run |77616ms/0|

모든 후속 Cargo는 offline/locked/no-default-features를 사용했다. root lock721586F9는 불변이다. standalone lock을 수동 편집하거나 root/기존 다목적 lock을 이식하지 않았다. 생성 lock의 원문과 registry-tuples.json·metadata를 보존했다. optional ORT lock resolution과 실제 compile을 구별하며 artifact/build-script 기록의 ORT·ort-sys compile 및 DirectML link는0이다.

## 실제 결과

고정 BIN3은 fresh=false로 이번 Cargo가 생성했고 원본/사본 SHA를 대조했다. 실제 목록을 원 expected64와 대조해 중복·누락0을 확인한 뒤 같은 BIN에 full name별 exact/thread1로 실행했다.

| 대상 | 고유 실제 결과 | 의미 |
|---|---:|---|
| P0 contract |22pass|기존 strict 취소 정책, 별도 worker Partial 관측, 원 flag/cutoff·유효 score/range·읽기 EOF 취소|
| protocol contract |28pass|원 descriptor SHA/context·strict wire·bounded response·취소 오류·관측값 absent|
| worker libtest |14pass|binding4·payload/source ownership·runner→P0→공통 wire·Windows ordinary-file3|
| 합계 |**64pass/0fail/0ignored**|수리 없이 원 작성64와 일치|

실행 순서/이름과 개별 stdout/stderr/receipt는 cases64.json/results64.json 및 case01~64에 있다. 각 호출은 실제1pass/0fail/0ignored를 확인했다. 각 stdout·stderr regular file을 끝까지 읽었으며 stderr 총0bytes다. pipe EOF나 실제 AI process family closure를 별도 관측한 것으로 바꾸지 않는다.

BIN SHA는 p0 `6D2148E9599D1CEEC8B980ED82E5265AA3618C3DB42DEE371BB56C5E9BCE3B4F`, protocol `6402C8CD029B72FC3F38B680079834F4E4942C3794D03AFD64420C37E10AC00F`, worker-libtest `782A9271BC415397C98B744739373BE5CAE71BB2CA6F42FE84F8782C0A6A6EE1`다. actual AI worker/model을 실행한 BIN이 아니다.

## 종료·보호·반환

원 runner72 receipt는 모두 terminal0이다. build의 원 exec session18598/23759도 종료 회수했다. 기록된 launcher/child 고유144 PID는 마감 시 부재였으며 이는 원 receipt/handle 종료와 별개의 보조 근거다. root1203·새 library471·기존 S1659·양 index/docs2 네 파일·기존 BIN48의 보호 차이0이다. 원 r4/raw/S40과 새 BIN3 원본/사본 해시 차이도0이다. target hash 읽기는 build/link 종료 뒤에만 수행했다.

heavy/shared 슬롯을 PM에 명시 반환한 뒤 private 정본을 마감했다. closeout.json/protection-after.json/receipts.json/validation-summary.json/case-io64.json에 원 수치와 한계를 기록했다. index/stage/commit/push0이다.

## 남은 경계

이 결과는 default path의 실제 타입/계약·일반 파일 수명·synthetic engine ports 검증이다. native-ort adapter/LoadedRuntimeAdmission는 default feature에서 제외돼 타입/ABI/link/loader를 입증하지 않는다. main/trusted release, parent payload r2와 SDK caller, 실제 signal·원 process identity·terminal/EOF/family cleanup, 실제 모델·제품 등록·배포 provenance/license/notices·최소 OS·사용자 품질 검토는 계속 후속이다. Partial 관측이나 통과 수를 Job 완료/취소 ACK·전체 목표 완료로 승격하지 않는다.

modoo code-change-review-verification의 원 call-path/fixture 주장을 실제 list64·동일 BIN exact 결과와 대조하는 방식을 적용했다. 이전 direct Rustfmt34와 이번 Cargo strict/실행64를 구별했고 원 source/외부 dependency/lock/사본/보호 근거를 보존했다. 신규 추론·다운로드·GUI/미디어/VM/DPI·시스템 DLL/전역 환경 변경은0이다.
