---
id: evidence:kinekino/20261029113901
title: 캡처 원 host IPC와 typed adapter 실제 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 캡처 원 host IPC와 typed adapter 실제 검증

2026-10-21 중앙 실행 배정으로 root `1a919dc3b51d57474a48c5b1b3eb30b78a278d53` /
Meari `b7ed5aba442bc143a9380536a850c4624a119a31`에서 수행했다. 이 기록은
캡처의 편집 가능 전환이나 전체 제품·시각 인수가 아니다.

## 입력과 실행

원 IPC 후보 E23 source1과 UI adapter B155 source3를 무수리 합성한
[source4](../../../../runtime/quality-review/ui-capture-ipc-adapter-r1/source4.json)의 SHA256은
`24872F1E1D8045956AC13E6D3B600A166EF3CFC03E316A057E47253DE8781578`이다.
기존2/new2, 겹침0, 현재 root와 원 before 불일치0, rebase0이다. 원 후보·부모는 보존했다.
기존 S/native T만 사용했고 S의 옛 Meari456을 채택471 bytes로 한 번 반영했다.
변경50과 이전 각 경로의 해시·mtime을 별도 기록했다. root/library 원본은 수정하지 않았다.

canonical runner conservative2/CPU2/mask3/BelowNormal/descendant Job,
offline/locked/j1을 사용했다. `cargo fmt --all --check`, default와
`native-proposal-window` 각각의 `cargo clippy --all-targets -- -D warnings`,
최소 Native libtest 빌드가 모두 통과했다. 새 Native BIN은
`84A51F144DE28B5EB91ABBEA1688DF90232DBB88ED3018C3658432F261CDA202`,
168,397,312 bytes이며 원본·고정 사본이 일치한다. 빌드/link 중 target BIN 해시는 읽지 않았다.

## 실제 결과와 범위

실제 `--list` 948개에서 정확한 adapter7·IPC3 이름이 모두 존재했다.
같은 고정 BIN으로 adapter7, 직접 영향 회귀51, 별도 IPC3가 통과했다.
고유61 pass/0 fail이며 수리는 없었다. 회귀 필터에 나타난 ignored6 중
이번 IPC3는 별도 실행했고, 나머지 ignored3은 실행하지 않았다. 전체 시험 실행 주장이 아니다.

adapter7은 실제 `ApplicationClient` 기본 typed method와 기록 응답을 사용해
원 anchor·성공 continuation, 요청 중 선택 변경 거절, SDK 오류 원형 보존,
원 pre-accept payload 재시도, 다른 owner 응답 반환, 선택 교체 초기화를 확인했다.
기록 응답은 backend 권한·원자성이나 실제 host dispatch의 증거가 아니다.
SDK backend의 별도 실제 CLI 인수는 root541b4c8에 채택된 선행 근거다.

IPC3는 각기 별도 runner/process에서 `--exact --ignored --test-threads=1 --nocapture`로
실행했다. 공개 capture route 거절과 보존된 inspection의 편집기 바인딩 거절 후,
원 host가 `Complete`를 반환했다. 세 시험 모두 원 LocalHost의 pipe/sdk Joined,
dispatch Idle, SDK execution worker stopped, unacknowledged0을 확인했다.
세 번째는 의도한 본문 panic `retaining viewer case assertion injection`을 잡고
그 정확한 payload를 보존하면서 같은 host의 종료를 완료했으며 최종 시험은 통과했다.
로그의 panic-hook 문자열을 시험 실패로 간주하지 않았다. close panic/error는 모두 0이었다.

host 생성 전 setup 실패, 종료 deadline·secondary-close fault 주입의 복구는
이번 실증에 포함되지 않는다. 실패 carrier의 보유 범위는 원 owner/payload를
시험 결과 보고까지 유지하는 것이며 프로세스 밖 영속 보유를 뜻하지 않는다.
자연 Drop이나 PID 부재만을 종료 증거로 쓰지 않았다.

## 종료와 보호

[최종 receipt](../../../../runtime/quality-review/ui-capture-ipc-adapter-r1/receipts-final.json)
17개가 terminal0이고 원 장기 도구 핸들5가 terminal0/EOF로 반환됐다.
관측된 소유 PID37개는 부재했으며 이를 descendant Job/원 receipt와 함께 대조했다.
원 endpoint·worker·host 종료의 근거는 각 IPC 로그와 동일 owner의 typed close receipt다.

root1822/library471/양 index·docs2/S 비입력2618/raw4/부모/before 및 원 사본4,
기타 보호 BIN48에 예상 밖 해시·mtime 차이는 없었다. S 추가 파일은 원 입력의
new2뿐이었다. Native canonical BIN의 의도된 CCDB→84A5 교체1은 원 CCDB 사본과
구분해 보존했다. 보호 상세는
[대조 기록](../../../../runtime/quality-review/ui-capture-ipc-adapter-r1/protection-after.json)과
[최종 입력 기록](../../../../runtime/quality-review/ui-capture-ipc-adapter-r1/final-input-check.json)을 따른다.
heavy/shared 슬롯은 PM에 명시 반환했다.

## 남은 인수

읽기 전용 제한은 유지했다. 실제 제품 execution owner로 adapter 요청을 전달하고
완료·취소·close/abandon을 회수하는 연결, 캡처 편집 가능 전환, 사용자 전체 경로,
GUI/창/렌더/브라우저 및 전체13화면군 시각 일치는 별도 미완료다.
실제 미디어·추론 worker/model·VM·DPI는 실행하거나 변경하지 않았다.
[원 IPC 계획](../../../plans/player/presentation/ipc-capture-negative-gate.md)과
[캡처 권한 복구](../../../plans/player/presentation/capture-live-binding-recovery.md)를 따른다.
