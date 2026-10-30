---
id: evidence:kinekino/20261029114359
title: Jobs 실행 attempt 읽기 전용 identity 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Jobs 실행 attempt 읽기 전용 identity 검증

2026-10-21, root `c7fe45510d05ba6297240354abc149ae46cc6f66`. 원 start-CAS가 반환한 Running record에서 job/attempt/input SHA/source blob SHA/선택적 submission request SHA를 복사한다. API request ID를 새로 만들지 않는다. private 필드와 no Deserialize, 기존 ExecutionControl의 default None을 유지한다. identity는 출처 정보이며 실행 생존·취소·게시 권한이 아니다.

검증 source5 SHA256은 `298EE76BD9592E68EC4F9AB0775A4092DAF8A2A3C8446F027F2C87D4C5604226`이다. 원 root 및 공유 snapshot의 before5 일치 후 해당5만 합성했다. 기존 cache/manifest/lock 및 원 resource runner의 conservative2 제한으로 offline/locked/-j1을 적용했다.

media-jobs all-target strict clippy 및 package fmt가 통과했다. fresh 고정 libtest의 실제 등록31·실행31이 모두 통과했다. 새 identity5는 이31에 포함되며 중복 계수하지 않는다. 기본 feature의 root lib/player-cli/jobs test 소비자 strict clippy도 통과했다. root jobs는 컴파일만 수행했다.

고정 시험 BIN SHA256 `06E92420366BA15B8F8F00E1C6743F88DA05A0FDC55F467C82EF707D8F51B321`, 2440704 bytes. 원 receipt6 terminal0, 기록 PID12 현재 부재, pending exec0이다. root/공유 비입력 보호 해시 차이0·index 불변·원본/사본 BIN 일치를 확인하고 heavy/shared를 반환했다.

원 로그·명령 결과·등록 명단·receipt·보호 목록·closeout은 [검증 인계](../../../../runtime/verification/job-attempt-identity-p0-r1/HANDOFF.md)에 보존한다. 위 검증은 실제 외부 worker 종료·AI 취소 ACK·게시·모델 품질의 인수가 아니다. retained owner를 보존하는 typed outcome, 제한 control, settlement-bound publication 및 host/session/retry/shutdown 전체 전파는 별도 P1 잔여다.
