---
id: plan:kinekino/20261029113857
title: Viewer fault adapter의 Jobs 로컬 바인딩 복구
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Viewer fault adapter의 Jobs 로컬 바인딩 복구

2026-10-24 PM exact1 private 배정. UI C3의 `callback reference is not exported` setup 실패에 대해 정상 `ViewerCallbacks::export`와 cfg(test) `install_fault_adapter`의 HostCallbacks 재조립 차이를 확인했다. 정상은 onOpenJobs의 Action local key를 저장하지만 helper에는 누락돼 있었다.

`runtime/development/viewer-fault-adapter-jobs-r1/HANDOFF.md`에 before/raw/delta·원 F300 대조·스킬 적용·미검증을 기록했다. leaf SHA256 E7F05CC7B96DE598B4DA3E347DFEA90FC2142D0F9F494D3A18DB167C6894D344, 후보 lifecycle.rs SHA256 310699309627AC7A31F98D7BA4C2A0124E94B70540E4613B22F7D60060A58D75다. 해당 helper에 정상과 같은 bind_local과 jobs_entry_key 갱신2줄만 추가했다.

private 파일별fmt/공백 검사 통과이며 실제 C3 해결·타입·기능은 미검증이다. 기존 production/Jobs 정책/guard/시험기대·원 실패·cleanup을 유지한다. canonical/S/T/Cargo/CLI/GUI/stage/commit/push0. launcher7은 독립 동결 상태로 유지하며 UI 단독 실행 슬롯의 후속 composite 검증은 PM 별도 배정이다.
