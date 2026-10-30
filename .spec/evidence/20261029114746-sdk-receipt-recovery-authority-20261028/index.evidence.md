---
id: evidence:kinekino/20261029114746
title: SDK 원 operation receipt 복구 권한 — R56/R57
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SDK 원 operation receipt 복구 권한 — R56/R57

1458 최종 custody terminal/exit0, 입력2310 content/identity unchanged 확인. 첫1450 실패 및 원 입력/로그는 유지했다.

최종 구성 검사:1455 Native strict terminal0/38464ms,1456 기본 strict terminal0/28288ms,1457 fmt check terminal0/3640ms. fixed-r57/sdk-execution-tests.exe SHA256 54AAC094724276D66C044EE23F385E8A8ABF5F3551D02087AA3B4468E8A67C45 /192241152bytes 보존. 전체190/정상bins 및 frame 복구 연결은 이번 실행 범위가 아니다. 아래 작성 시 진행 중 표기는 이 후속 결과와 구분해 읽는다.

중앙 PM, 2026-10-28. [복구 계약](../../contracts/production/frame-receipt-recovery.md)의 원 SDK 실행 계층을 구현 중이다. 원 source375/root canonical에 적용하지 않는 private 후보이며 preimage-r55-sdk-recovery에 기존 구현과 첫 실패 시험 소스를 보존했다.

ExecutionHandle/ExecutionAttachment의 receipt_recovery 및 retry_receipt는 원 OperationId의 owner Arc/sequence와 consumer attachment 권한을 검사한다. Running entry만 대기 관측/요청을 허용하며, 토큰은 pending 세대별 새 Arc 권한을 포함한다. 토큰 필드는 외부에서 구성할 수 없고 Serialize를 제공하지 않는다. 다른 operation/이전 세대 토큰은 거절, 같은 토큰 중복 요청은 AlreadyRequested이며 원 worker의 take_retry는 한 번만 true를 반환한다.

entry당 한 개의 bounded pending 관측과 u8 세대를 보유하고 16세대 초과는 ResourceExhausted다. 새 세대는 이전 요청을 소비한 뒤에만 만들 수 있다. 저장 실패를 알린 adapter가 원 owner를 유지하는 계약이며, SDK는 보고 자체를 물리 종료 검증으로 승격하지 않는다. 실제 원 family 관측은 adapter 책임이다. 고정 경로나 PID를 복구 권한으로 사용하지 않는다.

ApplicationClient::execute_recoverable은 기본적으로 기존 execute_controlled로 위임한다. worker는 원 실행 제어와 분리된 ReceiptRecoveryControl을 같은 호출에 제공한다. 기존 legacy client의 응답/취소를 대체하지 않는다. 아직 PlayerSdk/FfmpegFrameCapture/Session으로 전달하지 않았고 capabilities·CLI/wire에 지원을 표시하지 않았다. 외부 라이브러리 기본 포트 도입은 제품 receipt 복구 기능 완료가 아니다.

첫1450 R56 시험은 컴파일 후 실행에서 PermissionDenied로 실패했다. attachment 활성 중 owner.poll_close를 사용한 시험의 권한 오류이며 원 raw/receipt를 보존했다. R57은 attachment.poll_close를 추가해 같은 consumer 권한으로 기존 observe_close를 호출한다. bare handle/owner의 기존 권한 검사는 완화하지 않았다. 수정 후 SDK execution 전체 선택 회귀1453과 입력 snapshot1454는 작성 시 진행 중이다.

새 시험은 2세대에서 자동 요청 부재·명시 요청 단일 소비·중복·foreign owner·queued·이전 세대·무권한 bare handle 거절, 원 entry capacity 보유, 종료 요청 중 원 worker pending과 후속 복구, 첫 EngineFailed 보존·terminal/ACK 후 거절을 확인하도록 작성했다. 이 시험은 synthetic recovering ApplicationClient이며 실제 파일 저장 실패/Windows 프로세스는 R55 별도 근거다. 세대 상한의 실제16회 시험·다른 operation token 행렬·프레임 연결·실제 SDK receipt fault는 아직 미검증이다.

후속1453은 terminal/exit0(86915ms), 제품 SDK execution 시험20 pass/2 ignored/1536 filtered를 확인했다. 새 권한 시험 및 기존 model 협력 취소2도 실제 통과했다. ignored2는 실제도구를 요구하는 frame 시험이며 실행하지 않았다. 다른 workspace에서 같은 이름 필터로 0개 실행한 결과는 추가 통과로 계상하지 않는다. 입력1454 snapshot은2310, SOURCE SHA256 50D11821654F3ACB6515B78F8B438174DDB90C21A0F31A71FBCCC151613D74F5, inventory ED76E0014859AA23C918AB24285A96FF9FBC9CB9003C6A65FB393BAE6BDD5B75다. Native strict1455는 실제 exit0, 기본 strict1456은 작성 시 진행 중이다.

원 R52 전체190/정상bins·R53 실제발행 취소·R55 actual receipt fault는 각 원 입력의 증거로 유지한다. 최신 후보에 과거 실행을 승계하지 않는다. 전체CLI/GUI/AI품질0/9·원WIP/소유권/실패/보류 유지. C-only cache·heavy1·CPU2/BelowNormal·locked/offline/-j1, 컴파일 중 읽기 snapshot만 병행. 새 GUI/추론/다운로드/WARP/push 없음.

modootoday/software-qa/test-fixture-and-mock-build SHA256 A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및 code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B 적용: 실행 계층의 합성 제어 시험과 실제 파일/프로세스 인수를 분리하고 첫 실패를 보존한다.
