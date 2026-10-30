---
id: evidence:kinekino/20261029114738
title: Jobs owned attempt 계약 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Jobs owned attempt 계약 검증

2026-10-21 root94ca7a51fe4df1b93a75e62221d90aa546868e66, Meari2e0bf259 기준. source9 r2 SHA256 `9BF059B57B527FF80F41F353D3C09EAE0D6CC316B1455DEC7830309CD8F9FB9F`는 제한 scope·원 owner retained/settled·원 deadline·core-only CAS permit·bounded 최초/후속 오류 계약과 기존 shutdown 최초 오류 보존 수리다. 생산 binding issuer/외부 executor 등록은 포함하지 않는다.

원 r1은 fixture 함수와 지역 변수 이름 충돌로 strict E0618/101에 실패했다. 해당 변수명만 바꾼 r2를 별도 보존했고 나머지8 파일은 동일하다. 원 실패 로그를 지우거나 통과로 재분류하지 않았다.

최종 media-jobs all-target strict·fmt, fresh 고정 BIN의 실제 등록51/실행51, 문서 시험3이 통과했다. 문서 원문에서 E0599(게시 메서드 부재·Clone 부재)와 E0277(Deserialize 부재)을 확인했다. default root lib/player-cli/jobs 소비자 strict도 통과했으며 root jobs 실행은 하지 않았다. BIN SHA256 `C5733D77AF3C9D392F557001FF9569E1D92999180DF90C28350F2705E6E81E68`, 2567168 bytes다.

원 receipt8은 r1 실패101 하나와 최종0 일곱이며 모두 terminal이다. 기록 PID16 현재 부재·pending exec0, root1484/shared1444 비입력 보호 차이0/index·원 CLI/SDK/부모 후보 불변을 확인하고 heavy/shared를 반환했다. 원문·최종 소스·등록/실행 목록·receipt·보호 목록은 [검증 인계](../../../../runtime/verification/job-owned-attempt-union9-r1/HANDOFF.md)에 보존한다.

이 결과는 실제 WindowsFamily/외부 worker 종료·AI 취소 ACK·retry/session lease·생산 게시·모델 품질을 증명하지 않는다. 실제 WorkerHost→provider→workspace→CLI carrier와 원 start-CAS issuer·양 ACK/stop sweep·retry 전파는 별도 P1b 구현 및 검증 대상이다.
