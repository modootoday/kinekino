---
id: adr:kinekino/20261029113515
title: R4 실제 controls와 post-sync ACK의 증거 경계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R4 실제 controls와 post-sync ACK의 증거 경계

2026-10-19. PM 배정과 SDK 담당 합의로 구현한다. [계약·관측 상태](../../../sots/verification/ai/generation-gemma270-runtime-controls-r4.md), [실행 계획](../../../plans/gemma-runtime-controls-r4.md)을 분리한다.

R3 startup/cohort recorder 완결은 확인했지만 최종 scheduler/prefix/speculative/flavor getter와 flush 성공 후 외부 대조 가능한 ACK가 없다. argv/env는 configured 증거이며 이 부족분을 false/0/default로 채울 수 없다. R3 full CLI는 입력 artifact 미공급으로 본체에 도달하지 않았으므로 실제 strict join 거부로 표현하지 않는다.

R3를 수정하지 않고 별도 R4 source/workcopy·target·source closure·엔진·worker pin을 만든다. raw schema3 event 계약은 유지하고 별도 controls/commit/ACK schema1과 새 source profile을 명시한다. 기존 SDK의 FEA admission과 deny_unknown_fields를 우회하지 않는다. 제품 SDK/UI 코드는 각 소유 세션이 검토·구현한다.

실제 scheduler 보유 구성과 prefix manager의 구성 용량/disabled/두 cache 수락 값을 각각 읽는다. speculative는 실제 모델 getter, flavor는 성공한 GGUF native Gemma3 Text branch·pipeline의 kind/category/device 및 외부 trusted engine pin으로 정규화한다. env4와 실제 pool/OS thread cap을 구별하고 미관측은null/unavailable로 유지한다. 한 engine session의 생성 후 및 startup/cohort 요청 dispatch 전후 snapshot을 대조한다.

완결 raw write/truncate/sync_all 성공 이후 create_new commit을 쓰고 sync한다. 그 성공 뒤 별도 create_new ACK를 쓴다. revision/raw SHA·bytes/last ordinal과 run/session/engine/model/source·controls SHA를 결합한다. raw 자기선언만으로 마지막 sync가 증명됐다고 하지 않는다. ACK 자신의 sync 성공 역시 ACK 자기 필드로 독립 증명하지 않는다. source-reviewed producer의 성공 후 순서와 collector 실제 readback·owned child wait·outer 종료를 대조하며 OS 전원 손실 내구성이나 독립 capture 인증으로 확대하지 않는다.

worker는 실제 HTTP 해석값·launch 명시 overrides/removal·child와 outer exit를 구별한 RunReceipt v1과 정확한6 evidence role을 조립한다. 원문 sidecar bytes는 UTF8 문자열과 SHA로 보존하고 별도 support-paths를 제공해 새 SDK profile이 재해시/strict decode할 수 있게 한다. raw/plan/case fixture/receipt input mapping은 외부 pin으로 결합하며 journal 자신의 선언만 복사해 인증을 승격하지 않는다.

원문 R3 덮어쓰기, argv/default를 resolved로 대입, raw 안의 flush=true만 신뢰, 마지막 ACK를 반복 덮어쓰는 방식은 채택하지 않는다. 새 source admission·실제 모델 실행·SDK full join·비간섭·품질 검토는 별도 gate로 남긴다. 모델 추론과 다운로드는 PM이 단일 worker로 집행하며 개발 세션은 새 추론 없이 구현·검증·pin을 인계한다.
