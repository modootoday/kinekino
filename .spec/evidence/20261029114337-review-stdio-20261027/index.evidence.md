---
id: evidence:kinekino/20261029114337
title: Review host I/O 드라이버·실제 broken pipe
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Review host I/O 드라이버·실제 broken pipe

| 후조건 | 독립 읽기 | 결과 |
|---|---|---|
| 정상 EOF 후 원 join/물리완료 | 423의 두 host PROCESS/STOPPED·mpv 원 기록 | 실제1시험/23.33초 통과 |
| 실제 출력 pipe 오류 보존·오류 종료 | 424의 STDOUT-CLOSE/STOPPED/PROCESS | hostPID32844·BrokenPipe/ready_write/os232·exit8·succeeded=false |
| 오류 뒤 정상 물리 정리 | 같은 STOPPED와 원 media PROCESS | pipe/sdk Joined·owner1물리완료; mpvPID27784/creation134375238915095303·exit0/forced=false/Job Empty/IPC 종료 |
| 원 입력 보존 | 후속425 source 감사 | 2199파일 content/identity 동일·종료0/38879ms |

2026-10-27 중앙 PM. serve_stdio는 ready JSON 한 줄 write/flush→stdin EOF/read오류→begin_shutdown→원 pending 관측을 공통 Rust로 제공한다. 준비 출력 실패 뒤 stdin read를 건너뛰며 IO 오류가 cleanup 결과보다 먼저 유지된다. PendingReviewTransport::into_completion은 원 join/owner 물리 완료를 관측했을 때만 owner를 해제한다. cleanup관측10초는 session operation을 연장하거나 동기Read/Write를 hard-preempt하는 보장이 아니다. 미완료는 원 pending을 반환한다.

입력 runtime/verification/source375-review-stdio-input-r1/SOURCE.md SHA256 `DC5F6B85274DC670A71FB287C95E23388FB39F3B3CD3B4BA18D9B54A63DA0F02`; inventory `3DD594DD0631A47CC16D9F73955F47484C921D374FFD76A7254E6AC2C80DDDE2`,2199파일. preimage는 runtime/verification/review-stdio-preimage-r1이다. 419 미사용 import 최초 strict 실패·해당 source를 보존했다. 수정 후420 공통24/421 strict/422 no-run·fmt --check 통과. 원 stdout/stderr/launcher receipts는 source375-datalab-b0-r1-logs의 각 번호다.

423 결과 runtime/verification/review-stdio-eof-actual-datalab-r1/PROCESS-HOST-RETURN.json SHA256 `D9C7FA122EACC1A238DAF13E219A795E9C2C4A2FC583DB3B67C98DDE5C0F1D78`. 기존 별도OS2host EOF/restart가 새 공통 driver를 사용해 정상 종료·새scope/이전참조거절을 통과했다.

424 결과 runtime/verification/review-stdio-broken-actual-datalab-r1/PROCESS-HOST-IO-RETURN.json SHA256 `8D36188AA16BB208C4C3175F9CF0C876D8159F6D25265C440AD3AF0209E6E389`, 실제1시험/14.64초 통과. 부모는 실제 재생 연결을 CLI로 확인한 뒤 원 child stdout 읽기 handle을 닫고 stdin EOF를 보냈다. test-only GateWriter가 EOF까지 write를 지연하고 실제 OS stdout으로 쓰기 때문에 합성 BrokenPipe 객체를 반환한 시험이 아니다. stdout prefix는 의도적으로 보존하지 못했고 STDOUT-CLOSE.json에 그 범위를 기록했다. 원 host는 정리 후 의도한 exit8로 종료했다. Rusttest harness의 뒤따르는 broken stdout write를 피하기 위한 명시 exit이며 모든 원 runtime 정리는 그 전에 완료했다. 부모 fixture 전체는 예상 실패 상태를 검증해 종료0으로 통과했지만 host 실패를 정상 성공으로 바꾸지 않았다.

합성 unit3은 빈 host EOF, ready write/flush 실패 시 input미호출·최초오류 유지, input실패 뒤 물리정리 분리를 확인했다. 실제 OS 검증은 ready_write BrokenPipe 한 경로이며 stdin OS오류/ready_flush OS오류·crash/전원손실 전수는 아니다. 제품 CLI serve startup·trusted admission 공급·전체CLI/AI/GUI 인수도 남는다. 이전9F602A06의190/정상bins는 해당 입력 통합이며 이번 입력에서 재실행하지 않았다.

템플릿 이름만으로 실제 전송 내용을 알 수 없다. 원 SOURCE/producer/CLI SHA·request.json·actual pipe handle 폐쇄 시점·host/미디어 receipt를 보호된 runtime에 기록했다. PID/시간/GUID/스케줄링은 비재현 값이며 새 create-new 출력·원 case·고정 tool에서 의미상 후조건을 비교했다. 모델 추론/prompt는 N/A다. skill출처·본문/참조SHA와 producer 공급 공백 감사는 [후조건 감사](review-host-postconditions-20261027.md)를 따른다. 수동GUI·추론·다운로드·WARP·push0, 기존WIP/소유권/원증거·C캐시/locked/offline/jobs1/CPU2/BelowNormal/heavy1 유지.
