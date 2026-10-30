---
id: adr:kinekino/20261029113540
title: 원 producer 완료 상태의 명시 표현
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 producer 완료 상태의 명시 표현

2026-10-26 중앙 PM. 적용 범위는 내부 timeline producer bundle v1의 역사 관측 import다.

원 audio/video observer receipt는 result=Ok(())와 terminal=true를 기록한다. 원 구현은 child status.success일 때만 Ok를 반환했지만 receipt에 숫자 exit_code 필드가 직접 있는 것은 아니다. 따라서 정규화 기록에 편의 숫자0이나 Default 상태를 만들지 않고 필수 ObservedCompletion::ObserverAccepted를 사용한다. 원 process를 파싱한 결과와 정규화된 명시 상태가 같아야 한다.

원 capture process에는 숫자 exit_code와 원 CLI SHA/FileID가 직접 있으므로 그 값을 읽어 사용한다. 해당 관측과 wrapper accepted를 섞지 않는다. 입장 지정은 신뢰된 조립 코드의 선택이고, 실제 관측 교차검증·source-content identity·파생byte일치·SDK playback readiness·사용자품질은 각각 별도다.

이 결정은 기존 host/job의 취소·종료 상태를 재정의하지 않고 공개 CLI wire도 바꾸지 않는다. 아직 실제 발행되지 않은 내부 v1 normalized fixture만 갱신한다. 역사 원 receipt·실패·파일식별은 그대로 보존한다.
