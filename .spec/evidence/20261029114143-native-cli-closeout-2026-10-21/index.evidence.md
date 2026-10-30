---
id: evidence:kinekino/20261029114143
title: Native-only CLI 원 종료 owner의 no-media 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native-only CLI 원 종료 owner의 no-media 검증

2026-10-21 root `3a2870b847dfbc616c49e6bbbbd6cc23b9ca5f38`, Meari `4331ff3ea5cad71a7159bdb9e3f86fbfccb16220`에서 실행했다. 원 증거는 `runtime/verification/native-cli-closeout-union5-r1/`이다. 원 host/skip owner 보존과 오류 전달의 구체 no-media 범위이며 실제 mpv·LocalHost·normal CLI 실행이나 active engine 정리 성공은 아니다.

## 변경과 첫 실패

source1의 LocalHost 시작 오류 처리는 원 skip admission을 닫고 기존 failed_export_setup으로 명시 shutdown과 최초 오류/secondary 진단을 보존한다. 정상 EOF는 원 transport/SDK join 뒤 원 NativePlaybackHost+SkipHostOwner를 기존 PendingDependency의 구체 native carrier로 넘긴다. 반환된 cleanup 오류·panic payload·원 min-constrained 관측 cutoff를 같은 carrier에 남기고 기존 observe_dependency/finish_dependency/DependencyIncomplete를 소비한다. 새 process controller·공개 SDK 정책·parent 권한은 만들지 않았다.

원 source4와 시험 delta2를 명시 override1로 고유5(기존3/신규2)로 합성했다. 최초 source5 `75E85E80C70C80F49E601888B1E05367E1DDDFC4641A4ED0AA9ED87D7738537B`와 raw/parents/before/override를 보존했다. 최초 default all-target strict2242ms/101은 새 시험의 `.err().expect()`에 대한 Clippy err_expect 한 건이었다. 시험1파일을 동등한 `expect_err()`로 고쳤으며 정책·owner·warning suppression 변경0이다. 최종 source5 `20D798FBAD7DE85CC896080F4E19EE675203EE0152E35B80C4A1D8A0E6EB6A50`, 원문은 final-source5/와 repair-r1/에 있다. 루트 채택은 PM 책임이다.

기존 lock `721586F9C83ADA7CF504E5A5AE0CB168F66B017C3956CD84387D794E4EAE2528`을 그대로 offline/locked 사용했다. 새 dependency·lock 재생성·다운로드0이다.

## 실제 검사와 시험

수리 후 default all-target strict147373ms/0, 최종 fmt5193ms/0, 최소 CLI unit no-run23524ms/0이다. 새 unit BIN `7A548D76C8B671AF16693C30CEB492B2C49E5E0257F5C93A81CBA77E2CB1CAEE`(13688832bytes)를 고정하고 원본·사본 SHA를 대조했다. 실제 목록28개에서 expected7의 정확한 이름 전부를 확인해 각 `--exact --nocapture --test-threads 1`로 실행했다. 고유7pass/0fail/0ignored이며 각 나머지27개는 필터로 제외했다.

| 범위 | 실제 결과 | 입증 범위 |
|---|---:|---|
| 구체 no-media native carrier |3pass|실제 host/skip owner·pin guard/Weak·원 cutoff·원 transport 오류 보존|
| 기존 native 시작 오류 helper |1pass|명시 callback 한 번과 최초 오류 우선; 실제 LocalHost 시작 실패 주입은 아님|
| 기존 dependency driver |3pass|같은 owner의 pending·두 번째 poll·first/secondary 오류; 구체 active native 정리의 대체 근거는 아님|

새 시험은 실제 CatalogFiles/NativePlaybackHost/EpisodePlaybackSessions/SkipHostRegistry+원 SkipHostOwner를 구성했다. 작은 inert ordinary 파일을 mpv.exe라는 이름으로 생성하고 실제 SHA를 constructor에 주었다. PE를 검증하거나 실행하지 않았으며 playback.open을 호출하지 않아 engine은 deferred/no-media다. 원 registry Weak와 실제 executable write-open의 Win32 sharing violation32로 carrier의 원 owner·pin guard 유지를 확인했다.

no-media 성공은 실제 concrete poll의 receipt=null이며 child 종료 receipt가 아니다. 과거 cutoff는 같은 owner를 반환하고 더 늦은 constraint로도 cutoff를 갱신하지 않았으며 Timeout과 원 guard를 유지했다. 마지막 carrier의 명시 no-media fixture disposal은 active cleanup 증명으로 사용하지 않는다. 원 transport 오류는 PermissionDenied/request_id/details 전체 JSON이 실제 no-media completion 뒤에도 같았다. 시험3의 cleanup_complete=true 진단은 이 no-media dependency 관측에 한정한다.

## 종료·보호와 남은 범위

원13개 runner receipt는 모두 terminal(12개0·최초101 1개), 원 실행 세션도 종료 회수했다. 기록 고유PID26개 부재는 보조 근거로만 사용했다. root tracked1172/추가11·Meari456(기존 docs2 포함)·양 index·원 부모 raw·S core932/선택 외 tracked1169/실제 library456·고정 BIN 보호 차이0이다. 기존 UI40817FFE와 사본·normal CLI5DB·Meari 고정14쌍을 유지했고 normal dev BIN 변경0이다. 세 ordinary fixture의 원 파일 사본과 stdout/stderr를 보존했다. heavy/shared를 명시 반환한 뒤 문서를 마감했다.

native.shutdown은 여전히 동기 lock/child wait/IO join을 수행한다. 이 carrier가 hard timeout이나 선점을 제공하지 않는다. 기존 NativePlaybackHost::Drop의 추가 shutdown도 남는다. 반환된 native cleanup Err·panic·active child의 물리 종료 후 진단 오류·blocked joins·raw native family 종료는 이번 시험에서 유발하지 않았다. 원 carrier의 abnormal process exit를 실제 child-family 종료로 표시하지 않는다.

normal CLI5DB는 이전 VAD 소비 검증의 바이너리이며 이번 native 수정본으로 재생성·실행하지 않았다. 이번 결과는 새 CLI unit BIN의 정확7개에 한정한다. [후속 계획](../../../plans/player/native-cli-owner-closeout.md)의 실제 native/오류 경로와 제품 인수는 남아 있으며 전체 goal은 계속 진행 중이다.
