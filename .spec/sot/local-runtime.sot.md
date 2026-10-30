---
id: sot:kinekino/local-runtime
title: 공통 코어와 로컬 런타임 연결
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 공통 코어와 로컬 런타임 연결

상태: named pipe/peer identity·bounded codec·single SDK admission/원 worker 종료는 부분 구현했다. instance registry/heartbeat는 제품 인수 근거를 별도로 확정해야 하며, 원격 execution submit/query/cancel/terminal ACK는 현재 one-shot wire가 제공하지 않는다. 소스별 실행 근거와 계획을 구분한다. [Endpoint의 전송 전 취소 계약](endpoint-execution-control.md)을 따른다. [SDK](../../application/api/sdk-contract.md)의 adapter 책임이다. 외부 HTTP 서버·추가 상주 exe는 기본 요구가 아니다.

GUI는 UI 스레드의 WebView2/영상 HWND를 소유하고 application runtime의 명령 큐에 연결한다. UI 제어가 필요한 명령은 UI dispatcher 포트를 통해 해당 스레드에서 처리한다. 큐·모델·인코딩은 워커로 분리한다. CLI가 GUI와 연결할 때 Windows named pipe로 동일 facade에 접근한다. GUI가 없으면 CLI는 로컬 파일/프로젝트/제작용 in-process headless runtime을 조립할 수 있다. SDK 정책은 두 경로에서 동일하다.

headless CLI 단독 작업은 프로세스 수명에 귀속된다. detach는 소유할 durable host가 있을 때만 허용한다. 기본 CLI는 보이지 않는 GUI나 세 번째 daemon을 자동 실행하지 않는다. GUI 종료 시 실행 작업은 명시적인 종료 정책과 interrupted 기록을 남기고 재실행 시 중복 제출 없이 복구 확인한다. IPC 연결이 끊긴 것만으로 소유 작업을 취소하지 않는다.

로컬 instance registry는 사용자 SID·workspace/project store ID·opaque instance ID·protocol version·만료/heartbeat만 기록한다. PID 재사용을 확인하고 endpoint를 임의 경로로 받지 않는다. 복수 인스턴스에서 기본 선택이 모호하면 instance 목록을 반환하고 선택을 요구한다. 재생 SessionId와 작업 JobId는 host 범위를 가진다. 하나의 프로젝트 저장소에는 single-writer 잠금과 명시적 version 충돌을 적용해 GUI/단독 CLI가 동시에 쓰지 못하게 한다. 잠금 해제는 PID 숫자만으로 판정하지 않는다.

pipe ACL은 현재 사용자만, remote clients 거절, 클라이언트/서버 사용자와 protocol handshake를 확인한다. 이름만 맞는 가짜 endpoint를 신뢰하지 않는다. 명령 최대1MiB·응답 최대8MiB·이벤트 backlog1024개를 초기 제안 한도로 두고 파일/미디어/모델 대용량은 ArtifactId/검증된 저장 참조로 전달한다. 길이 prefix+UTF-8 JSON 프레이밍, 상한 초과·잘림·잘못된 JSON·타임아웃을 분리한다. 요청 URL이나 프로젝트 데이터가 임의 실행 파일·셸·스크립트 실행 경로가 되지 않는다.

세션 URL·쿠키·HF token은 호스트 메모리/credential store 포트 내부에서만 처리한다. IPC snapshot·capabilities·진단에 반환하지 않는다. named pipe는 SDK의 전송 구현이며 domain이나 GUI 코드에 프레임 파서를 넣지 않는다. 호스트 없이 가능한 명령과 UI thread 필요 명령은 capabilities에서 명시한다. 크기 한도·장애 복구·ACL·동시성 실측은 남은 인수 항목이다.
