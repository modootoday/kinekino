---
id: evidence:kinekino/20261029113826
title: 명시적 captured 편집 진입의 guarded IPC 검증 — 2026-10-21
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 명시적 captured 편집 진입의 guarded IPC 검증 — 2026-10-21

## 관측 범위

Root `b8e20f83b78d4e802dd0c3c3d1d941985c558acc`, Meari `9c1b6a339885ce9e2f593c322bc11ee213e0a0bf`, lock E469 기준이다. 근거는 `runtime/quality-review/ui-capture-positive-entry-r1/`에 보존한다. 원 child10 `850A8DD60FB23D97729AF6946671B896A06349E1278BB52E16EED87488E337EB`, rebase union19 B932, 부모 final13 8376은 변경하지 않았다. 부모와 겹치는 child4, 부모 r2 fixture delta1과의 충돌0, 기존 입력7/새 입력3을 대조했다.

첫 전체 fmt는 path-overridden guarded_ipc.rs의 자식 fixture 상대 경로로 terminal1/1887ms였다. 원 실패를 보존하고 gate revisions/r1에서 명시적 `guarded_ipc/fixture.rs` 경로 한 줄만 추가했다. 최종 child10 SHA256은 `8E3590080B711F724D5F91C03B61411DD782AF2C1EDF57AE39BBF714AD862014`, 최종 union19는 `BDF13F8738CF3FDA65483D85471A86CBD0F46074AD2214ADABFB64B2C3D519D9`다. 퍼리프 skip_children 포맷 검사를 모듈 트리 유효성 증거로 쓰지 않는다.

## 실제 검증

fmt/default·Native all-target clippy `-D warnings` 및 최소 Native libtest no-run 빌드가 통과했다. 빌드 terminal0/110409ms다. 고정 실행 파일 `kinekino_player-694dc45cc9ec9e94.exe` SHA256은 `2322CCA32BEBD3F3CAA83DC6E63924379D29A4DC6E523CAD8B87C341B5A84963`, 169248256 bytes이며 gate artifacts/native-libtest.exe에 보존했다. 실제 등록966 tests 중 고유79건 통과/실패0이다. `actual-tests79.json`에 실제 통과 이름과 로그를 기록했다.

실제 configured ProposalFiles/ProjectFiles와 PlayerSdk, named-pipe LocalHost, EndpointClient, 원 ExecutionOwner/ViewerHost를 사용한 새 ignored7을 각각 독립 프로세스에서 실행했다.

- 명시적 새 origin callback → 편집2회 → accept → 원 payload 재시도. 결과 revision4 및 reused acceptance, 저장된 capture 불변을 확인했다.
- 외부 편집 후 stale capture의 RevisionConflict와 UI의 자동 새로고침·continuation 갱신 부재를 확인했다.
- 별도로 발행한 legacy originNone의 callback 거절과 SDK NeedsInput을 확인했다.
- 소비하지 않은 실제 성공 terminal이 원 host close를 막고 명시 소비·ACK 후 해제되는 것을 확인했다.
- 백엔드 처리 후 클라이언트 panic의 불확실성 보존, 무단 재제출 부재 및 이후 stale 거절을 확인했다. panic hook 출력은 의도한 worker fault이며 시험은 통과했다.
- 백엔드 성공 후 decode 실패에서 원 operation/raw/diagnostic을 보존하고 명시 discard·ACK하는 것을 확인했다.
- 새 origin이어도 기존 selection binding은 편집을 활성화하지 않는 것을 확인했다.

기존 직접 영향51/adapter7/worker9/별도 IPC5도 같은 최종 BIN으로 다시 실행했다. 과거72 통과를 승계하지 않았다. 일반 필터의 ignored15 중 새7+기존5를 별도 실행했으며 다른3건은 미실행이다.

원 request payload, SDK raw response, 원 terminal, 명시 소비/discard 및 ACK 증거의 동일성은 각 프로세스 내 assertion으로 확인했다. 모든 성공 RPC의 전체 JSON transcript를 별도 파일로 덤프한 것은 아니다. 원 시험 소스·stdout/stderr·receipt·남겨진 fixture 저장소가 현재의 보존 근거다.

새 IPC7 모두 원 endpoint pipe Joined / SDK Joined / dispatch Completed / worker_stopped=true / unacknowledged=0 / Complete를 기록했다. 기존 IPC5도 원 소유자의 Complete를 확인했으며, 기존 intentional assertion panic 시험의 primary payload는 보존됐다. close panic·close error는 모두0이다.

## 보호·인수 제한

root1851/library473/index-docs4/S2647/BIN51의 예상 밖 hash·mtime 차이0이다. S의 기존 입력7 변경과 새 입력3은 최종10 raw/S hash로 확인했다. canonical BIN의 4DD0→2322 교체1은 의도된 변경이며 이전 고정 사본은 보존했다. 원 child10·union19·부모13 raw mismatch0이다. 변경 bytes에 현재 mtime을 적용했고 동일 bytes를 무조건 touch하거나 전체 clean하지 않았다.

28 runner receipts 모두 terminal이며 원 fmt 실패1을 보존했다. 원 long sessions5는 terminal/EOF, 관측 launcher/child56 PID는 마지막 조회에서 부재였다. 최종 슬롯 반환은 gate closeout.json을 따른다. 별도 저장소의 외부 빌드는 조작하지 않았다. 잠시 관측된 별도 Player check의 소유권·target은 확인하지 못했으며 충돌 사실로 단정하지 않는다.

현재 제품의 기본 captured route는 read-only다. 본 검증은 opt-in entry와 실제 SDK 편집 계약의 증거이며 제품 Card mutation input callback·mount/frame/route 연결, GUI/렌더링·전체13 시각 승인 증거가 아니다. 구체 adapter ACK 실패 주입은 여전히 미검증이며 일반 plugin의 ACK 시험으로 대체하지 않는다. 후속 제품 입력 연결은 별도 source-only로 진행한다.
