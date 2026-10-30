---
id: evidence:kinekino/20261029114553
title: 채택 출력 판정 저장 어댑터 focused 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 채택 출력 판정 저장 어댑터 focused 검증

2026-10-20. [계약14 검증](accepted-output-review-contracts-2026-10-20.md) 이후 [계획](../../../plans/player/accepted-output-review-sdk.md)의 infrastructure exact12를 실제 Windows 파일/잠금과 합성 입력으로 검사했다. 사용자 의미 품질 승인이나 신규 wire/CLI 지원이 아니다.

## 고정 입력과 최소 수리

snapshot은 `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1`이다. Meari `1df44812c11e61b7e8c000324e35ce461b0a53b0`381파일, UI9r2/overlay22r2, SDK계약14 `c49425c81f6dede4556e34cc8dd42281431b69de`와 기존 SDK3a/NativeChild/UI11/manifest/assets를 보호했다. infrastructure12만 overlay했으며 신규11파일의 이전 부재와 기존 mod 원본을 `runtime/quality-review/sdk-output-review-storage-r1/before12.json` 및 before-source에 보존했다.

r1 pin JSON SHA256은 `C323E584DE32280253E6B8CF185C0ACC62A7FAC190998F5C7E3D03A2C8F74FA9`이다. 첫 컴파일은 `SystemRecordClock`의 pub(super)가 조부모 proposals 조립에서 보이지 않는 E0603으로 101/11091ms였다. 원 source·핀·stderr/receipt를 보존한 뒤 clock.rs의 한 줄만 `pub(in crate::infrastructure::proposals)`로 수리했다. r2 source/pin은 `runtime/coordination/sdk-output-review-infrastructure12-r2/`, pin JSON SHA256 `48CA503F0417B5A8E2298254E36DB49DA8E8588B961B300F73BA02D14610DE0C`이다. 정책/추가경로/dependency/wire 변경은 없다.

## 원실행 결과

| 검사 | 실제 결과 | receipt elapsed_ms |
|---|---|---:|
| r1 신규16 | 컴파일101, 시험 실행 전 E0603 | 11091 |
| r2 신규 output_reviews | 16 passed, 0 failed/ignored, 365 filtered | 48790 |
| 기존 infrastructure proposals | 7 passed, historical1 포함 | 1529 |
| 기존 proposals integration | CLI/schema/caps1 passed | 78379 |
| 기존 model_review schema filter | 1 passed | 12899 |
| 기존 model_review CLI inspect/record/readback filter | 1 passed | 2069 |
| root default all-target strict | 0 | 27714 |
| root native feature all-target strict | 0 | 33911 |
| root cargo fmt --check | 0 | 1380 |

신규16·기존7·기존CLI/schema3은 각각 구분한다. contract14/provider4와 bridge3 no-run은 이번 변경 영향상 반복하지 않았다. explicit actual-nine ignored 실행·개인 미디어·새 GUI/렌더/재생은 수행하지 않았다. 모든 명령은 절대 snapshot cwd/manifest guard, 기존 canonical Rust runner SHA256 `692C22751FAC7F9999F2D833571DAB6605AFD58E66711370494E62DD8A62C463`, offline locked-j1/threads1/affinity3/논리CPU2/BelowNormal/descendants job으로 순차 실행했다.

## 입증한 동작과 한계

실제 두 ProposalFiles handle이 같은 writer.lock에서 경합하고 session drop 뒤 기존 읽기가 재개됨을 확인했다. 다른 root guard는 거부됐다. 현재 target 변경/동일 ID 다른 요청/trusted criteria 교환/evidence 누락·동일크기 raw 교환은 신규 게시 없이 실패했다. 원 요청 retry는 원 body/SHA/time을 반환하고 clock 호출을 늘리지 않았다. 저장 뒤 proposal/evidence/clock이 없어져도 독립 history가 stored 구조와 digest만 검사해 NotChecked로 읽었다.

capacity256·입력/파일 byte 제한, reserved/partial/temp/extra entry·digest/typed 변조·게시 후 exact body 교환을 검사했다. fault injection 오류는 원 code/remediation과 publication_may_exist/phase/잔존 경로를 유지했다. 게시 완료 후 주입 오류의 다음 동일 요청은 완전한 원 record에 수렴했고, 불완전 directory는 NeedsInput과 수동 복구 요구로 보존됐다. 프로세스 실제 crash/power-loss의 모든 위치와 외부 evidence 전역 트랜잭션은 보증하지 않는다.

기존7은 helper 추출 뒤 원 load/publish/CAS/partial·문서/receipt/historical 동작을 확인한다. legacy candidate 관련 새 시험은 명시 합성 fixture bytes 비변경 범위이며 기존 CLI/schema2는 원 candidate 경로의 별도 관측이다. 이 결과를 모든 파일/모델/CLI 기능의 전역 호환으로 확대하지 않는다. 실제 local SystemTime 구현은 기본 adapter의 정상 기록에서 사용됐으나 epoch 이전/overflow OS 시각을 강제로 재현하지 않았다. injected clock 오류는 별도 실패 경계를 확인한다.

신규 output-review application session/파일 어댑터는 이 focused 범위에서 동작하지만 새 SDK Request/schema/capabilities/typed CLI 명령은 아직 등록하지 않았다. technical acceptance·판정 저장·사용자 usable 의미 승인·미디어 적용은 각각 별개다. 자동 apply false/model sample 집계 제외를 유지한다.

## 마감

최종 r2 frozen/live/snapshot12 차이0과 보호 base 차이0이다. 원실행9개 모두 terminal(최초101 하나 포함), launcher/child18 PID 부재를 `final-receipts.json`, `final-owned.json=[]`으로 보존했다. index/stage/commit/push는 수행하지 않았다. heavy/snapshot을 PM에 명시 반환했다.

이전 Meari 종료감사 수량은 PM 추가 설명에 따라 현재 state-predicate 폴더20실행/40PID와 별도 고정9dc core-r1 1실행/2PID로 구분한다. 전체는21/42다. r2-repair.json의 당시 PM인계(전체 철회라는 표현)는 원자료로 보존하며 이 범위 설명으로 보완한다. 해당 생산자 실행을 이번 SDK9실행에 합산하지 않는다.
