---
title: "AI 네이티브: CLI와 MCP"
description: "사람은 앱으로, 스크립트와 AI 에이전트는 같은 명령을 지금은 CLI로, 나중에는 MCP 서버로 씁니다."
ref: agents
nav_order: 4
hero: { c: split, visual: player-mock, scene: agent }
sections:
  - { c: features, cols: 2 }
  - { c: split }
  - { c: prose }
---

kinekino는 GUI 앞의 사람과 터미널의 AI 에이전트가 같은 계약으로 같은 일을 하도록 만듭니다. CLI가 있는 이유가 이것이고, MCP는 같은 명령 위의 두 번째 표면일 뿐 별도 기능이 아닙니다.

## 표면

- **CLI**: `version`, `capabilities`, `schema`, `GROUP ACTION`. `status:available`
- **요청 JSON**: `--input FILE` 또는 `-`와 `--json`. `status:available`
- **로컬 호스트 엔드포인트**: OS IPC(Windows 네임드 파이프, Unix 소켓)로 `--endpoint`. `status:partial`
- **MCP 서버**: stdio로 `kinekino mcp serve`(로드맵 P9). `status:planned`
- **마크다운 트윈과 llms.txt**: 이 사이트의 모든 페이지를 마크다운으로. `status:available`

## 안전한 호출

모든 요청에 요청 ID, 기대 리비전, 멱등 키가 붙습니다. 오류는 코드와 `retryable` 표시와 함께 돌아와 에이전트가 언제 다시 시도할지 압니다.

```sh
kinekino --input request.json --json --request-id r-42 --idempotency-key k-42
```

## 계획된 MCP

MCP 서버는 `kinekino schema`와 같은 도구와 스키마를 내보내고 같은 로컬 호스트로 호출을 보냅니다. 기본은 꺼져 있습니다.

| 설정 | 기본값 |
|---|---|
| `settings.mcp.enabled` | 꺼짐 |
| `settings.mcp.allowed_tools` | 읽기 전용 |
| `settings.mcp.confirm_destructive` | 켜짐: 삭제나 유료 호출은 먼저 확인 |

> [!CAUTION]
> P9가 나오기 전에는 MCP 서버가 없습니다. 지금은 에이전트가 CLI를 쓰면 됩니다.

[CLI 레퍼런스](cli.md)
