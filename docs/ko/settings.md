---
title: "설정과 개인정보"
description: "하나의 설정 화면, kinekino settings로도 스크립트 가능. 설정 창은 Linux에서 확인했고 Windows·macOS는 아직 확인 전."
ref: settings
nav_order: 21
hero: { c: split, visual: player-mock, scene: settings }
sections:
  - { c: prose }
  - { c: prose }
  - { c: prose }
---

설정은 한 화면에 모이고 같은 키를 `kinekino settings`로 읽고 쓸 수 있습니다. `kinekino settings gui [YYYY-MM]`는 Meari가 그리는 네이티브 창을 엽니다. 모든 OS가 같은 코드이며 제품 디자인 토큰·아이콘과, `kinekino fonts fetch`가 내려받아 검증하는 고정 버전 Noto Sans 글꼴(라틴·한국어·일본어)을 씁니다. 창은 Linux(X11)에서 확인했고 Windows·macOS는 아직 확인하지 않았습니다. 변경은 전부 적용되거나 하나도 적용되지 않고, 지출 한도에 가까우면 한도 대화상자가 뜹니다. 스크립트용 `kinekino settings page|apply`도 그대로입니다.

## 추론

| 키 | 기본값 |
|---|---|
| 모드 | 클라우드, 또는 로컬 우선, 로컬만 |
| 비용 한도 전체, 호출당, 월간 | $5.00, $0.50, $10.00 |
| 확인 비율 | 80% |
| 동의 범위 | 프로젝트, 또는 작업, 항상(철회 가능) |
| 화자 분리 | 꺼짐, 또는 로컬 |
| 로컬 모델 팩 | 설치, 삭제, 검증을 작업으로 |

## 에이전트

| 키 | 기본값 |
|---|---|
| MCP 사용 | 꺼짐 |
| 허용 도구 | 읽기 전용 |
| 파괴적 호출 확인 | 켜짐 |

## 개인정보

- 작업에 필요한 구간이나 프레임만 올리고 영상 전체는 올리지 않습니다.
- 토큰은 지금은 현재 세션 동안 메모리에만 둡니다. OS 자격 증명 저장소 어댑터(Linux Secret Service부터)는 만들어 두었지만 아직 앱에 연결하지 않았습니다. 설정에는 설정 여부만 보입니다.
- 설정은 기본값, 파일, 환경 변수, 명령줄 순서로 읽습니다.

> [!TIP]
> 아무것도 컴퓨터 밖으로 나가면 안 된다면 로컬만을 고르세요. AI 기능은 로컬 모델 팩만 씁니다.
