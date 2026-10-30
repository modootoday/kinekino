---
id: plan:kinekino/20261029113941
title: Native-only CLI의 시작 오류와 원 종료 owner
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# Native-only CLI의 시작 오류와 원 종료 owner

현재 SDK source-only 후속이다. 실행 권한은 [현재 배정](execution-leases.md)만 따르며 UI의 공유 슬롯을 사용하지 않는다. VAD43·실제 CLI1의 성공을 native 종료 증거로 재사용하지 않는다.

최신 상태: [native closeout CPU 검증](../../sots/verification/application/native-cli-closeout-2026-10-21.md)에서 root3a2870b/Meari4331ff3·구체 no-media3/기존 helper1/직접 driver3의 정확7pass/0fail/0ignored를 확인했다. 최초 Clippy err_expect101과 시험1파일의 동등 수리를 보존했고 최종5 `20D798FBAD7DE85CC896080F4E19EE675203EE0152E35B80C4A1D8A0E6EB6A50`·before5/raw를 PM에 인계했다. 원13receipt 종료·보호0·명시 반환을 마쳤다. 아래 source-only/미실행 표현은 준비 당시 원문이며, active native cleanup·실제 LocalHost startup 실패·normal 수정 CLI 실행은 여전히 미검증이다.

## 먼저 닫을 시작 오류 보고 틈

`src/bin/player-cli/native_host.rs`의 LocalHost 시작 오류는 host/skip 조립 뒤 `?`로 반환해 NativePlaybackHost::Drop의 shutdown 결과를 버렸다. endpoint 공개나 playback.open 전이므로 실제 active mpv 누수를 관측한 것은 아니다.

독립 source1 `0BAA3A3EAFCCC0D96EA6F7FFE8FF03CECAE5EFBF7EA60D225F3348862A6CF2EC`는 Err에서 원 skip admission을 닫고 기존 failed_export_setup으로 명시 shutdown·최초 오류/secondary 진단을 연결한다. 원 SkipHostOwner는 호출 동안 유지된다. 원문·before·gate 제안은 `runtime/coordination/native-cli-startup-closeout-r1/`에 있다. rustfmt만 통과했으며 Cargo/실행0이다. 기존 helper exact1은 재사용할 수 있지만 실제 LocalHost 시작 실패 분기를 주입한 시험은 아니다. 새 공개 factory나 무의미한 test abstraction을 추가하지 않는다.

## 원 EOF owner의 현재 흐름과 독립 후보

현재 transport는 원 LocalHost join 전 NativePlaybackHost+SkipHostOwner tuple을 보관한다. join 뒤 synchronous host.shutdown 오류가 반환되면 Complete(error)로 보고한 다음 tuple을 Drop한다. NativeEpisodeEngine은 실패한 Active 또는 failed-start recovery를 내부에 유지하지만 그 바깥 host owner가 사라지면 보관 수명이 끝날 수 있다. 실제 child exit/worker join·진단 publication 실패와 단순 timeout을 구별해야 한다.

source4 `583967837A0146966D896473632BE75BD13DFAFBD618915BAEF94908A64E4BDA`는 그 tuple을 기존 PendingDependency의 구체 native carrier로 옮긴다. 반환된 cleanup 오류·panic payload·min-constrained 원 observation deadline을 같은 carrier에 남기고 기존 observe_dependency/finish_dependency/DependencyIncomplete를 소비한다. transport/setup 최초 오류는 기존 공통 driver의 우선순위를 따른다. 이 후보는 source1의 native_host 경로를 명시 override해 계승하며 두 manifest를 단순 더하지 않는다. 원문·before·정확 흐름은 `runtime/coordination/native-cli-retaining-closeout-r1/`이다. 실제 모듈 그래프의 recursive rustfmt만 통과했으며 컴파일·시험·OS 실행0이다.

source4는 반환된 오류 뒤 owner 유지의 최소 리팩터 후보다. 기존 native.shutdown 자체는 동기 lock/child wait/IO join을 수행하므로 hard timeout이나 nonblocking poll을 제공하지 않는다. 기존 Drop의 추가 shutdown 가능성도 남는다. 원 cleanup이 끝나지 않은 abnormal process exit는 raw legacy mpv child-family 종료 증명이 아니다. 이 한계를 없앴다고 표시하지 않는다.

## 검증과 더 뒤의 범위

2026-10-21 source-only 시험 준비를 마쳤다. 원 source4는 유지하고 실제 no-media host/원 SkipHostOwner·Weak/실제 pin 파일 쓰기 잠금의 새 시험3개를 독립 delta2 `EE7E60F69BB84E9B49994E4C33D214E2709A9953F66A61698DC993B7F91BC5B4`로 작성했다. 최소 완료(None)·과거 cutoff의 같은 owner/min 제약 유지·원 transport 오류의 request_id/details 전체 보존을 대상으로 한다. inert 파일을 실제 mpv/PE로 실행하지 않으며 playback.open·child·LocalHost·GUI·모델0이다. expiry의 명시 no-media fixture disposal은 active cleanup 증거가 아니다.

최종 입력은 `runtime/coordination/native-cli-closeout-union5-r1/`의 고유5 `75E85E80C70C80F49E601888B1E05367E1DDDFC4641A4ED0AA9ED87D7738537B`다. source4+시험 delta2, cfg(test) 선언의 명시 override1·부모2·원 root before 기존3/신규2를 고정했다. 실제 CLI 모듈 경로 전체의 recursive rustfmt는 통과했으며 컴파일·실등록·실행0이다. 예상 정확7은 새 concrete3+기존 startup helper1+직접 영향 dependency driver3이며 한 CLI unit BIN에서 확인할 후보로만 기록한다. UI 채택 뒤 before와 실제 등록 분모를 다시 대조하고 배정에 따라 실행한다. 구체 returned cleanup Err/panic/error-after-active-close는 이 fixture로 검증하지 못하므로 잔여다.

1. source1부터 current root/Meari·before/index/docs2/BIN/lock·자원을 대조하고 배정된 최소 fmt/strict/CLI unit BIN·정확 helper1을 인수한다. 기존43/151·UI 전체 회귀를 추가하지 않는다.
2. source4는 별도 concrete native owner 타입/호출·반환 오류/원 panic/expiry 보존을 검증한다. 실제 engine 종료가 필요한 경우 새 짧은 native OS 배정을 받는다. 단순 fake dependency 시험이 구체 native owner를 입증하지 않는다.
3. 기존 ParentCleanupTask의 thread_completion/JoinTicket은 UI observation 재사용 후보지만 ParentBinding/epoch 권한을 임의 standalone CLI에 만들지 않는다. 별도 worker로 동기 shutdown을 옮겨도 원 CLI host·skip owner와 dispatcher/join을 함께 유지해야 하며 hard preemption이 되지는 않는다.
4. active child/family의 강제 종료 보장이 필요하면 현재 raw Child launch와 기존 owned-process WindowsFamily를 비교하는 별도 migration 근거가 먼저 필요하다. 새 process controller나 재발견한 PID 제어로 대체하지 않는다.

구현→직접 영향 검증→원 handle/receipt/EOF·보호 대조→정본 기록→명시 반환을 단계마다 마친다. 현재 구현 후보를 제품 지원·실제 native 누수 복구·전체 종료 완료로 올리지 않는다.
