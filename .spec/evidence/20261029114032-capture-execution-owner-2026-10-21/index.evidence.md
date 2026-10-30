---
id: evidence:kinekino/20261029114032
title: Capture ExecutionOwner 검증 — 2026-10-21
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Capture ExecutionOwner 검증 — 2026-10-21

## 관측 결과

Root `57db58c3d4b34554b0a347b04afacc751b682feb`, Meari `9c1b6a339885ce9e2f593c322bc11ee213e0a0bf` 기준이다. 원 source13 A046은 보존하며, 검증 최종13 SHA256은 `837600FD8456D46D45502731CD18D75BC82872DEDDEE8EE61DD0E66A35DDA401`이다. 입력·원본·개별 stdout/stderr/receipt는 `runtime/quality-review/ui-capture-execution-owner-r1/`에 있다.

`cargo fmt --check`, default/Native all-target clippy `-D warnings`, Native libtest no-run 빌드가 통과했다. 최종 실행 파일은 `kinekino_player-694dc45cc9ec9e94.exe`, SHA256 `4DD03B0989849A7A14B578BF33766C0F2B450D3302CF6DC570353C96FECA227E`, 168755200 bytes다. gate `artifacts/native-libtest.exe`에 고정했다. 실제 목록은 959 tests다.

직접 영향 회귀51 + 기존 capture adapter7 + 실제 ExecutionOwner worker9 + 별도 프로세스의 ignored LocalHost2/기존 IPC3 = 고유72건 통과, 실패0이다. 일반 필터에서 나타난 ignored8 중5건을 별도 실행했고 나머지3건은 실행하지 않았다. IPC5는 의도한 assertion panic을 보존하는 기존 시험이며 최종 시험 결과는 통과다.

IPC5건 모두 원래 호스트의 pipe Joined / SDK Joined / dispatch Idle / worker_stopped=true / unacknowledged=0 / close_panic=false / close_errors=0 / Complete를 관측했다. Raw terminal을 UI에 자동 적용하지 않고 명시적 consume/discard, ACK, attachment release, 같은 owner의 종료로 이어가는 계약을 시험했다.

## 보존한 실패와 최소 수리

1. 원 fmt1: path override 안의 하위 module 상대 경로 오류. r1은 `execution_owner.rs`의 명시 경로만 추가했다.
2. 원 build101 E0432: root/S의 export는 동일했으나 Cargo가 현재 worker 모듈을 포함하지 않는 과거 Media rlib2를 fresh로 재사용했다. 해당 fingerprint 전체를 보존하고 정확 lib marker2만 제거했다.
3. 후속 build101 E0463: 과거 의존성을 연결한 executor의 fresh artifact. 같은 방식으로 exact marker1만 제거했다. 같은 S/target에서 재빌드 성공으로 확인했다. source/API/lock/profile 변경이나 전체 clean은 없다.
4. IPC1 최초101: 공통 fixture는 반환 전 read attachment를 해제한다. r2는 기존 IPC와 같은 resume/reproject를 수행하고 실제 exclusive read 소유권을 먼저 확인하도록 시험 준비를 고쳤다. 제품 동작이나 본 assertion을 완화하지 않았다. 실패에서도 원 host Complete를 확인했다.

원 실패·원 실행 파일 C624·원 source13은 삭제하지 않았다. mirror는 같은 bytes의 mtime을 보존하되 변경 bytes에 과거 artifact보다 오래된 mtime을 승계하지 않는다. 전체 파일 touch로 대신하지 않는다.

## 보호와 제한

Root1844/library473/index-docs4/Media raw54 및 기존 보호 BIN48의 예상 밖 hash·mtime 변경0이다. S baseline2640 중 변경6은 승인된 기존 입력6이며 새 입력7과 함께 최종13 raw/S 해시를 확인했다. 메타데이터 감사의 잘못된 DateTime 문자열 비교는 UTC ticks 비교로 재수행했으며 처음 콘솔 결과를 drift 근거로 쓰지 않는다.

최종 receipt/원 프로세스 종료 및 슬롯 반환 상태는 gate `closeout.json`을 따른다. 슬롯 반환 전 false를 먼저 기록한다.

이는 제품 편집 화면 활성화, 실제 GUI 실행, 렌더링 또는 전체13 시각 승인 근거가 아니다. 기존 captured route는 read-only다. 구체 capture adapter의 ACK 실패 주입 seam은 아직 없으며 일반 plugin의 ACK 재시도 시험을 동등한 증거로 쓰지 않는다.

양성 진입 source10은 `runtime/development/ui-capture-positive-entry-r1/`에 별도 동결되어 있다. effective parent는 r1 EB62이며 현재 r2 부모와의 명시적 재기준 검토가 필요하다. 해당 후보의 configured store/guarded IPC7은 아직 컴파일·실행하지 않았다. 제품 Card 입력 callback·mount/frame/route 연결도 후속 범위다.
