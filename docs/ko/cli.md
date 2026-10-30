---
title: "CLI 레퍼런스"
description: "사람, 스크립트, AI 에이전트를 위한 kinekino 명령줄."
ref: cli
nav_order: 20
---

현재 빌드에 있는 명령만 적었습니다. 기본 출력은 JSON입니다.

```sh
kinekino version              # 제품·API 버전
kinekino capabilities         # 이 빌드와 호스트가 할 수 있는 것
kinekino schema COMMAND       # 명령 하나의 요청 스키마
kinekino GROUP ACTION         # 예: subtitles convert (capabilities 참고)
kinekino --input request.json # 전체 요청 JSON(context, command, payload)
kinekino platform capabilities
kinekino platform config
```

공통 플래그: `--json|--human`, `--request-id ID`, `--timeout-ms MS`, `--expected-revision N`, `--idempotency-key KEY`, `--endpoint ENDPOINT`.

| 표면 | 상태 |
|---|---|
| 위 명령 | `status:available` |
| OS IPC 로컬 호스트 `--endpoint` | `status:partial` |
| `kinekino settings` | `status:planned` |
| `kinekino mcp serve` | `status:planned` |
