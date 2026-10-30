---
id: plan:kinekino/20261029113909
title: R1 fixture repair3 endpoint-first 시점 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R1 fixture repair3 endpoint-first 시점 검토

effective9 SHA80B6F1C083F5B368D93F5164204E6ACA981EA3A541041CFACA423D5181E2854C/raw9 일치. PM은 원 OrderedClose의 endpoint join→callback→Jobs→Card 순서와 Observed hold가 endpoint 전송 앞에 있음을 대조했다. delta1은 tests.rs의 실제 close progress 진단과 첫 poll 뒤 원3초 end 안에서 Closing을 관측하는 시험 수정이다.

즉시 shutdown_started/admission 거절·원 inspection 보유·notComplete, 실제 Closing 이후 기존 단언·원 inspect1/hold·cleanup을 보존한다. 첫 poll에 특정 progress variant를 강제하거나 새 deadline/worker/시도를 만들지 않는다. 제품 순서·정책·guard 변경0이다. 원18의 지나치게 이른 상태 기대와 실패 자료를 보존한다. 확정 P1/P2 발견0이며 새 타입·실제 인수는 아직0이다.

같은 UI 슬롯의 한leaf S적용·새fmt/strict/BIN/list 후 원18 exact부터 재검증하고 남은 R1/영향 회귀를 수행한다. 기존 modoo QA의 원타임라인/owner·미검증 분리 기준을 적용했다.
