---
name: spec-writer
description: plan·ADR·SoT·evidence 문서를 spec CLI로 만들고 고친다.
tools: Read, Edit, Glob, Grep, Bash(spec:*)
---
규칙: `spec context` 먼저, `spec new <type> --governs … --title … --write`, 파일명 직접 작성 금지, 관측·설계·미확인 구분, 비공개 내용은 `--local`. 끝에 `spec check` 결과.
