---
id: plan:kinekino/20261029113635
title: 제품 연결 r3 fixture delta 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 연결 r3 fixture delta 검토

PM이 원 strict02 terminal101/E0433와 r2→r3 exact1 diff를 읽었다. source13 SHA D47EC94D22C3922B427F557EF65A04A336985AAD2EC561AA5C304536B16F75E3, leaf12F6FF640FD25C13D622DD314745CCB90BEE786C200667082F40F20AF2382276. raw13 일치·나머지12 해시 불변이다. 원 r2와 실패 증거를 보존한다.

tests/download_cli.rs의 기존 media_probe 모듈을 crate-visible로 하고 infrastructure::download 아래 재노출한다. 기존 media_download_owned fixture와 동일 연결이며 기존 inline 시험의 ExpectedTrack 경로를 복원한다. 시험 삭제·기대값 완화·제품 정책 변경0. 이 정적 검토에서 확정 P1/P2 발견0이며 타입/시험 통과는 아직 미검증이다. 같은 SDK 단독 슬롯에서 이 delta만 S에 적용하고 새 fmt/strict부터 원 실행 계약을 재개한다. 추가 경로·새 실패는 별도 revision으로 보고한다.

기존 modoo code-change-review-verification 본문 및 verdict 참조의 원 오류·호출 경로·fixture/제품 책임·실제 검증 분리 기준을 적용했다.
