---
name: public-surface-scan
description: 공개 표면(README, docs, .spec, 커밋 메시지, --help 출력)을 점검·개선할 때. 비공개 흔적·낡은 문구·링크 오류를 찾는다. PR의 비공개 용어 체크와 로컬 사전 검사도 여기에 둔다.
---
# 공개 표면 점검
1. 비공개 흔적(일상 점검, 금지어 목록 사본이 있을 때만): `git grep -n -i -F -f <목록> -- . ':!.spec.local'` (`-F` 필수, 2026-11-03). 목록 사본 위치는 `.agents/local/`(추적 안 함). 결과에 용어를 그대로 옮겨 적지 않는다. 보조: 로컬 정밀 스캔 도구(`.agents/local/scan.md`, stdout만, 결과 파일 금지; tracked 파일명·내용).
2. `spec check` 0/0, README·docs의 플랫폼 상태 표기가 실제 구현과 맞는지 비교.
3. CLI `--help`·`version` 출력이 실제 명령과 맞는지 확인.
4. 발견마다: 위치 / 심각도(높음·중간·낮음) / 수정안. 수정은 worktree 안에서만.

## PR의 비공개 용어 체크 (2026-11-16, adr:kinekino/20261116065239)
관문은 PR 필수 체크 `Public surface`다. CI가 PR 범위(`base..head`)의 커밋 메시지·추가 줄·파일명과 PR 제목·본문을 시크릿 `KINEKINO_DENY_TERMS`로 검사하고, 목록이 없으면 실패한다. 로그에는 위치와 목록 번호만 나온다.
1. **로컬 사전 검사(선택)**: push 전에 `cargo xtask deny-terms --range origin/main..HEAD --terms-file <목록>`. 0건이 아니면 push 전에 고친다.
2. **이미지·바이너리**: 경로·파일명만 점검한다. 신규 바이너리 추가는 PR 본문에 적고, 내용 바이트 검사는 하지 않는다.
3. **적중 처리**: CI 체크가 실패하면 병합하지 않는다. 추가 줄·파일명은 새 커밋으로 고치고, 커밋 메시지 적중은 병합 전 자기 토픽 브랜치에서만 다시 쓴다. 오탐은 목록을 좁혀 해결하고 체크를 건너뛰지 않는다.
